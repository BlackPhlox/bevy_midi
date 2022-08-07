use bevy::{prelude::*, tasks::IoTaskPool};
use crossbeam_channel::{Receiver, Sender};
use midir::ConnectErrorKind;
pub use midir::MidiOutputPort;
use std::error::Error;
use std::fmt::Display;
use MidiOutputError::*;

pub struct MidiOutputPlugin;

impl Plugin for MidiOutputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MidiOutputSettings>()
            .insert_resource(MidiOutputConnection { connected: false })
            .add_event::<MidiOutputError>()
            .add_startup_system(setup)
            .add_system(on_reply);
    }
}

/// Settings for [`MidiOutputPlugin`].
#[derive(Clone, Debug)]
pub struct MidiOutputSettings {
    pub port_name: &'static str,
}

impl Default for MidiOutputSettings {
    fn default() -> Self {
        MidiOutputSettings {
            port_name: "bevy_midi",
        }
    }
}

/// [`Resource`](bevy::ecs::system::Resource) for sending midi events.
///
/// Change detection will only fire on this resource when its output ports are
/// refreshed.
pub struct MidiOutput {
    sender: Sender<Message>,
    receiver: Receiver<Reply>,
    ports: Vec<(String, MidiOutputPort)>,
}

impl MidiOutput {
    /// Update the available output ports.
    ///
    /// Change detection is fired when the ports are refreshed.
    pub fn refresh_ports(&self) {
        self.sender.send(Message::RefreshPorts).unwrap();
    }

    /// Connect to the given `port`.
    pub fn connect(&self, port: MidiOutputPort) {
        self.sender.send(Message::ConnectToPort(port)).unwrap();
    }

    /// Disconnect from the current midi port.
    pub fn disconnect(&self) {
        self.sender.send(Message::DisconnectFromPort).unwrap();
    }

    /// Send a midi message.
    pub fn send(&self, msg: [u8; 3]) {
        self.sender.send(Message::Midi(msg)).unwrap();
    }

    /// Get the current output ports.
    pub fn ports(&self) -> &Vec<(String, MidiOutputPort)> {
        &self.ports
    }
}

/// [`Resource`](bevy::ecs::system::Resource) for checking whether MidiOutput is
/// connected to any ports.
///
/// Change detection fires whenever the connection changes.
pub struct MidiOutputConnection {
    connected: bool,
}

impl MidiOutputConnection {
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

// XXX: give doc comment/implement Error trait
#[derive(Clone, Debug)]
pub enum MidiOutputError {
    ConnectionError(ConnectErrorKind),
    SendError(midir::SendError),
    SendDisconnectedError([u8; 3]),
    PortRefreshError,
}

impl Error for MidiOutputError {}
impl Display for MidiOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            SendError(e) => e.fmt(f)?,
            SendDisconnectedError(m) => write!(
                f,
                "Couldn't send midi message {:?}; output is disconnected",
                m
            )?,
            ConnectionError(k) => match k {
                ConnectErrorKind::InvalidPort => {
                    write!(f, "Couldn't (re)connect to output port: invalid port")?
                }
                ConnectErrorKind::Other(s) => {
                    write!(f, "Couldn't (re)connect to output port: {}", s)?
                }
            },
            PortRefreshError => write!(f, "Couldn't refresh output ports")?,
        }
        Ok(())
    }
}

fn setup(mut commands: Commands, settings: Res<MidiOutputSettings>) {
    let (m_sender, m_receiver) = crossbeam_channel::unbounded();
    let (r_sender, r_receiver) = crossbeam_channel::unbounded();

    let thread_pool = IoTaskPool::get();
    thread_pool
        .spawn(midi_output(m_receiver, r_sender, settings.port_name))
        .detach();

    commands.insert_resource(MidiOutput {
        sender: m_sender,
        receiver: r_receiver,
        ports: Vec::new(),
    });
}

fn on_reply(
    mut output: ResMut<MidiOutput>,
    mut conn: ResMut<MidiOutputConnection>,
    mut err: EventWriter<MidiOutputError>,
) {
    while let Ok(msg) = output.receiver.try_recv() {
        match msg {
            Reply::AvailablePorts(ports) => {
                output.ports = ports;
            }
            Reply::Error(e) => {
                err.send(e);
            }
            Reply::Connected => {
                conn.connected = true;
            }
            Reply::Disconnected => {
                conn.connected = false;
            }
        }
    }
}

enum Message {
    RefreshPorts,
    ConnectToPort(MidiOutputPort),
    DisconnectFromPort,
    Midi([u8; 3]),
}

enum Reply {
    AvailablePorts(Vec<(String, MidiOutputPort)>),
    Error(MidiOutputError),
    Connected,
    Disconnected,
}

async fn midi_output(
    receiver: Receiver<Message>,
    sender: Sender<Reply>,
    name: &str,
) -> Result<(), crossbeam_channel::SendError<Reply>> {
    use Message::*;

    let output = midir::MidiOutput::new(name).unwrap();
    sender.send(get_available_ports(&output))?;

    // Invariant: exactly one of `output` or `connection` is Some
    let mut output: Option<midir::MidiOutput> = Some(output);
    let mut connection: Option<(midir::MidiOutputConnection, MidiOutputPort)> = None;

    while let Ok(msg) = receiver.recv() {
        match msg {
            ConnectToPort(port) => {
                let start_connected = output.is_none();
                let out = output.unwrap_or_else(|| connection.unwrap().0.close());
                match out.connect(&port, name) {
                    Ok(conn) => {
                        connection = Some((conn, port));
                        output = None;
                        sender.send(Reply::Connected)?;
                    }
                    Err(conn_err) => {
                        sender.send(Reply::Error(ConnectionError(conn_err.kind())))?;
                        if start_connected {
                            sender.send(Reply::Disconnected)?;
                        }
                        connection = None;
                        output = Some(conn_err.into_inner());
                    }
                }
            }
            DisconnectFromPort => {
                if let Some((conn, _)) = connection {
                    output = Some(conn.close());
                    connection = None;
                    sender.send(Reply::Disconnected)?;
                }
            }
            RefreshPorts => match &output {
                Some(out) => {
                    sender.send(get_available_ports(out))?;
                }
                None => {
                    let (conn, port) = connection.unwrap();
                    let out = conn.close();

                    sender.send(get_available_ports(&out))?;

                    match out.connect(&port, name) {
                        Ok(conn) => {
                            connection = Some((conn, port));
                            output = None;
                        }
                        Err(conn_err) => {
                            sender.send(Reply::Error(ConnectionError(conn_err.kind())))?;
                            sender.send(Reply::Disconnected)?;
                            connection = None;
                            output = Some(conn_err.into_inner());
                        }
                    }
                }
            },
            Midi(msg) => {
                if let Some((conn, _)) = &mut connection {
                    if let Err(e) = conn.send(&msg) {
                        sender.send(Reply::Error(SendError(e)))?;
                    }
                } else {
                    sender.send(Reply::Error(SendDisconnectedError(msg)))?;
                }
            }
        }
    }
    Ok(())
}

// Helper for above.
//
// Returns either Reply::AvailablePorts or Reply::PortRefreshError
// If there's an error getting port names, it's because the available ports changed,
// so it tries again (up to 10 times)
fn get_available_ports(output: &midir::MidiOutput) -> Reply {
    for _ in 0..10 {
        let ports = output.ports();
        let ports: Result<Vec<_>, _> = ports
            .into_iter()
            .map(|p| output.port_name(&p).map(|n| (n, p)))
            .collect();
        if let Ok(ports) = ports {
            return Reply::AvailablePorts(ports);
        }
    }
    Reply::Error(PortRefreshError)
}
