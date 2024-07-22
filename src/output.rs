use super::MidiMessage;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use crossbeam_channel::{Receiver, Sender};
use midir::ConnectErrorKind;
pub use midir::MidiOutputPort;
use std::fmt::Display;
use std::{error::Error, future::Future};
use MidiOutputError::{ConnectionError, PortRefreshError, SendDisconnectedError, SendError};

pub struct MidiOutputPlugin;

impl Plugin for MidiOutputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MidiOutputSettings>()
            .init_resource::<MidiOutputConnection>()
            .add_event::<MidiOutputError>()
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, reply);
    }
}

/// Settings for [`MidiOutputPlugin`].
///
/// This resource must be added before [`MidiOutputPlugin`] to take effect.
#[derive(Resource, Clone, Debug)]
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

/// [`Resource`](bevy::ecs::system::Resource) for sending midi messages.
///
/// Change detection will only fire when its input ports are refreshed.
#[derive(Resource)]
pub struct MidiOutput {
    sender: Sender<Message>,
    receiver: Receiver<Reply>,
    ports: Vec<(String, MidiOutputPort)>,
}

impl MidiOutput {
    /// Update the available output ports.
    pub fn refresh_ports(&self) {
        self.sender
            .send(Message::RefreshPorts)
            .expect("Couldn't refresh output ports");
    }

    /// Connect to the given `port`.
    pub fn connect(&self, port: MidiOutputPort) {
        self.sender
            .send(Message::ConnectToPort(port))
            .expect("Failed to connect to port");
    }

    /// Disconnect from the current output port.
    pub fn disconnect(&self) {
        self.sender
            .send(Message::DisconnectFromPort)
            .expect("Failed to disconnect from port");
    }

    /// Send a midi message.
    pub fn send(&self, msg: MidiMessage) {
        self.sender
            .send(Message::Midi(msg))
            .expect("Couldn't send MIDI message");
    }

    /// Get the current output ports, and their names.
    #[must_use]
    pub fn ports(&self) -> &Vec<(String, MidiOutputPort)> {
        &self.ports
    }
}

/// [`Resource`](bevy::ecs::system::Resource) for checking whether [`MidiOutput`] is
/// connected to any ports.
///
/// Change detection fires whenever the connection changes.
#[derive(Resource, Default)]
pub struct MidiOutputConnection {
    connected: bool,
}

impl MidiOutputConnection {
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

/// The [`Error`] type for midi output operations, accessible as an [`Event`](bevy::ecs::event::Event)
#[derive(Clone, Debug, Event)]
pub enum MidiOutputError {
    ConnectionError(ConnectErrorKind),
    SendError(midir::SendError),
    SendDisconnectedError(MidiMessage),
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
                    write!(f, "Couldn't (re)connect to output port: invalid port")?;
                }
                ConnectErrorKind::Other(s) => {
                    write!(f, "Couldn't (re)connect to output port: {}", s)?;
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
        .spawn(MidiOutputTask {
            receiver: m_receiver,
            sender: r_sender,
            settings: settings.clone(),
            output: None,
            connection: None,
        })
        .detach();

    commands.insert_resource(MidiOutput {
        sender: m_sender,
        receiver: r_receiver,
        ports: Vec::new(),
    });
}

fn reply(
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
                warn!("{}", e);
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
    Midi(MidiMessage),
}

enum Reply {
    AvailablePorts(Vec<(String, MidiOutputPort)>),
    Error(MidiOutputError),
    Connected,
    Disconnected,
}

struct MidiOutputTask {
    receiver: Receiver<Message>,
    sender: Sender<Reply>,
    settings: MidiOutputSettings,

    // Invariant: exactly one of `output` or `connection` is Some
    output: Option<midir::MidiOutput>,
    connection: Option<(midir::MidiOutputConnection, MidiOutputPort)>,
}

impl Future for MidiOutputTask {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.output.is_none() && self.connection.is_none() {
            self.output = midir::MidiOutput::new(self.settings.port_name).ok();
            self.sender
                .send(get_available_ports(self.output.as_ref().unwrap()))
                .unwrap();
        }

        if let Ok(msg) = self.receiver.recv() {
            use Message::{ConnectToPort, DisconnectFromPort, Midi, RefreshPorts};

            match msg {
                ConnectToPort(port) => {
                    let was_connected = self.output.is_none();
                    let out = self
                        .output
                        .take()
                        .unwrap_or_else(|| self.connection.take().unwrap().0.close());
                    match out.connect(&port, self.settings.port_name) {
                        Ok(conn) => {
                            self.connection = Some((conn, port));
                            self.output = None;
                            self.sender.send(Reply::Connected).unwrap();
                        }
                        Err(conn_err) => {
                            self.sender
                                .send(Reply::Error(ConnectionError(conn_err.kind())))
                                .unwrap();
                            if was_connected {
                                self.sender.send(Reply::Disconnected).unwrap();
                            }
                            self.connection = None;
                            self.output = Some(conn_err.into_inner());
                        }
                    }
                }
                DisconnectFromPort => {
                    if let Some((conn, _)) = self.connection.take() {
                        self.output = Some(conn.close());
                        self.connection = None;
                        self.sender.send(Reply::Disconnected).unwrap();
                    }
                }
                RefreshPorts => match &self.output {
                    Some(out) => {
                        self.sender.send(get_available_ports(out)).unwrap();
                    }
                    None => {
                        let (conn, port) = self.connection.take().unwrap();
                        let out = conn.close();

                        self.sender.send(get_available_ports(&out)).unwrap();

                        match out.connect(&port, self.settings.port_name) {
                            Ok(conn) => {
                                self.connection = Some((conn, port));
                                self.output = None;
                            }
                            Err(conn_err) => {
                                self.sender
                                    .send(Reply::Error(ConnectionError(conn_err.kind())))
                                    .unwrap();
                                self.sender.send(Reply::Disconnected).unwrap();
                                self.connection = None;
                                self.output = Some(conn_err.into_inner());
                            }
                        }
                    }
                },
                Midi(message) => {
                    if let Some((conn, _)) = &mut self.connection {
                        if let Err(e) = conn.send(&message.msg) {
                            self.sender.send(Reply::Error(SendError(e))).unwrap();
                        }
                    } else {
                        self.sender
                            .send(Reply::Error(SendDisconnectedError(message)))
                            .unwrap();
                    }
                }
            }
        }

        cx.waker().wake_by_ref();
        std::task::Poll::Pending
    }
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
