use super::{MidiMessage, KEY_RANGE};
use bevy::prelude::Plugin;
use bevy::{prelude::*, tasks::IoTaskPool};
use crossbeam_channel::{Receiver, Sender};
use midir::ConnectErrorKind; // XXX: do we expose this?
pub use midir::{Ignore, MidiInputPort};
use std::error::Error;
use std::fmt::Display;
use MidiInputError::*;

pub struct MidiInputPlugin;

impl Plugin for MidiInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MidiInputSettings>()
            .init_resource::<MidiInputConnection>()
            .add_event::<MidiInputError>()
            .add_event::<MidiData>()
            .add_startup_system(setup)
            .add_system_to_stage(CoreStage::PreUpdate, reply)
            .add_system(debug);
    }
}

/// Settings for [`MidiInputPlugin`].
///
/// This resource must be added before [`MidiInputPlugin`] to take effect.
#[derive(Clone, Debug)]
pub struct MidiInputSettings {
    pub client_name: &'static str,
    pub port_name: &'static str,
    pub ignore: Ignore,
}

impl Default for MidiInputSettings {
    fn default() -> Self {
        Self {
            client_name: "bevy_midi", // XXX: change client name? Test examples?
            port_name: "bevy_midi",
            ignore: Ignore::None,
        }
    }
}

/// [`Resource`](bevy::ecs::system::Resource) for receiving midi messages.
///
/// Change detection will only fire when its input ports are refreshed.
pub struct MidiInput {
    receiver: Receiver<Reply>,
    sender: Sender<Message>,
    ports: Vec<(String, MidiInputPort)>,
}

impl MidiInput {
    /// Update the available input ports.
    ///
    /// This method temporarily disconnects from the current midi port, so
    /// some [`MidiData`] events may be missed.
    ///
    /// Change detection is fired when the ports are refreshed.
    pub fn refresh_ports(&self) {
        self.sender.send(Message::RefreshPorts).unwrap();
    }

    /// Connects to the given `port`.
    pub fn connect(&self, port: MidiInputPort) {
        self.sender.send(Message::ConnectToPort(port)).unwrap();
    }

    /// Disconnects from the current input port.
    pub fn disconnect(&self) {
        self.sender.send(Message::DisconnectFromPort).unwrap();
    }

    /// Get the current input ports, and their names.
    pub fn ports(&self) -> &Vec<(String, MidiInputPort)> {
        &self.ports
    }
}

/// [`Resource`](bevy::ecs::system::Resource) for checking whether [`MidiInput`] is
/// connected to any ports.
///
/// Change detection fires whenever the connection changes.
#[derive(Default)]
pub struct MidiInputConnection {
    connected: bool,
}

impl MidiInputConnection {
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

/// An [`Event`](bevy::ecs::event::Event) for incoming midi data.
///
/// This event fires from [`CoreStage::PreUpdate`].
pub struct MidiData {
    pub stamp: u64,
    pub message: MidiMessage,
}

/// The [`Error`] type for midi input operations, accessible as an [`Event`](bevy::ecs::event::Event).
#[derive(Clone, Debug)]
pub enum MidiInputError {
    ConnectionError(ConnectErrorKind),
    PortRefreshError,
}

impl Error for MidiInputError {}
impl Display for MidiInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ConnectionError(k) => match k {
                ConnectErrorKind::InvalidPort => {
                    write!(f, "Couldn't (re)connect to input port: invalid port")?
                }
                ConnectErrorKind::Other(s) => {
                    write!(f, "Couldn't (re)connect to input port: {}", s)?
                }
            },
            PortRefreshError => write!(f, "Couldn't refresh input ports")?,
        }
        Ok(())
    }
}

fn reply(
    mut input: ResMut<MidiInput>,
    mut conn: ResMut<MidiInputConnection>,
    mut err: EventWriter<MidiInputError>,
    mut midi: EventWriter<MidiData>,
) {
    while let Ok(msg) = input.receiver.try_recv() {
        match msg {
            Reply::AvailablePorts(ports) => {
                input.ports = ports;
            }
            Reply::Error(e) => {
                warn!("{}", e);
                err.send(e);
            }
            Reply::Connected => {
                conn.connected = true;
            }
            Reply::Disconnected => {
                conn.connected = false;
            }
            Reply::Midi(m) => {
                midi.send(m);
            }
        }
    }
}

fn setup(mut commands: Commands, settings: Res<MidiInputSettings>) {
    let (m_sender, m_receiver) = crossbeam_channel::unbounded::<Message>();
    let (r_sender, r_receiver) = crossbeam_channel::unbounded::<Reply>();

    let thread_pool = IoTaskPool::get();
    thread_pool
        .spawn(midi_input(m_receiver, r_sender, settings.clone()))
        .detach();

    commands.insert_resource(MidiInput {
        sender: m_sender,
        receiver: r_receiver,
        ports: Vec::new(),
    });
}

enum Message {
    RefreshPorts,
    ConnectToPort(MidiInputPort),
    DisconnectFromPort,
}

enum Reply {
    AvailablePorts(Vec<(String, MidiInputPort)>),
    Error(MidiInputError),
    Connected,
    Disconnected,
    Midi(MidiData),
}

async fn midi_input(
    receiver: Receiver<Message>,
    sender: Sender<Reply>,
    settings: MidiInputSettings,
) -> Result<(), crossbeam_channel::SendError<Reply>> {
    use Message::*;

    let input = midir::MidiInput::new(settings.client_name).unwrap();
    sender.send(get_available_ports(&input))?;

    // Invariant: exactly one of `input` or `connection` is Some
    let mut input: Option<midir::MidiInput> = Some(input);
    let mut connection: Option<(midir::MidiInputConnection<()>, MidiInputPort)> = None;

    while let Ok(msg) = receiver.recv() {
        match msg {
            ConnectToPort(port) => {
                let was_connected = input.is_none();
                let s = sender.clone();
                let i = input.unwrap_or_else(|| connection.unwrap().0.close().0);
                let conn = i.connect(
                    &port,
                    settings.port_name,
                    move |stamp, message, _| {
                        s.send(Reply::Midi(MidiData {
                            stamp,
                            message: [message[0], message[1], message[2]].into(),
                        }))
                        .unwrap()
                    },
                    (),
                );
                match conn {
                    Ok(conn) => {
                        sender.send(Reply::Connected)?;
                        connection = Some((conn, port));
                        input = None;
                    }
                    Err(conn_err) => {
                        sender.send(Reply::Error(ConnectionError(conn_err.kind())))?;
                        if was_connected {
                            sender.send(Reply::Disconnected)?;
                        }
                        connection = None;
                        input = Some(conn_err.into_inner());
                    }
                }
            }
            DisconnectFromPort => {
                if let Some((conn, _)) = connection {
                    input = Some(conn.close().0);
                    connection = None;
                    sender.send(Reply::Disconnected)?;
                }
            }
            RefreshPorts => match &input {
                Some(i) => {
                    sender.send(get_available_ports(i))?;
                }
                None => {
                    let (conn, port) = connection.unwrap();
                    let i = conn.close().0;

                    sender.send(get_available_ports(&i))?;

                    let s = sender.clone();
                    let conn = i.connect(
                        &port,
                        settings.port_name,
                        move |stamp, message, _| {
                            s.send(Reply::Midi(MidiData {
                                stamp,
                                message: [message[0], message[1], message[2]].into(),
                            }))
                            .unwrap()
                        },
                        (),
                    );
                    match conn {
                        Ok(conn) => {
                            connection = Some((conn, port));
                            input = None;
                        }
                        Err(conn_err) => {
                            sender.send(Reply::Error(ConnectionError(conn_err.kind())))?;
                            sender.send(Reply::Disconnected)?;
                            connection = None;
                            input = Some(conn_err.into_inner());
                        }
                    }
                }
            },
        }
    }
    Ok(())
}

// Helper for above.
//
// Returns either Reply::AvailablePorts or Reply::PortRefreshError
// If there's an error getting port names, it's because the available ports changed,
// so it tries again (up to 10 times)
fn get_available_ports(input: &midir::MidiInput) -> Reply {
    for _ in 0..10 {
        let ports = input.ports();
        let ports: Result<Vec<_>, _> = ports
            .into_iter()
            .map(|p| input.port_name(&p).map(|n| (n, p)))
            .collect();
        if let Ok(ports) = ports {
            return Reply::AvailablePorts(ports);
        }
    }
    Reply::Error(PortRefreshError)
}

// A system which debug prints note events
fn debug(mut midi: EventReader<MidiData>) {
    for data in midi.iter() {
        let pitch = data.message.msg[1];
        let octave = pitch / 12;
        let key = KEY_RANGE[pitch as usize % 12];

        if data.message.is_note_on() {
            debug!("NoteOn: {}{:?} - Raw: {:?}", key, octave, data.message.msg);
        } else if data.message.is_note_off() {
            debug!("NoteOff: {}{:?} - Raw: {:?}", key, octave, data.message.msg);
        } else {
            debug!("Other: {:?}", data.message.msg);
        }
    }
}
