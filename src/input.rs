use super::{KEY_RANGE, MidiMessage};
use crate::safe_wrappers::MidiInputPort;
use MidiInputError::{ConnectionError, PortRefreshError};
use bevy::prelude::Plugin;
use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use midir::ConnectErrorKind; // XXX: do we expose this?
pub use midir::Ignore;
use std::error::Error;
use std::fmt::Display;

#[cfg(not(target_arch = "wasm32"))]
use bevy::tasks::IoTaskPool;

#[cfg(not(target_arch = "wasm32"))]
use std::future::Future;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;

pub struct MidiInputPlugin;

impl Plugin for MidiInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MidiInputSettings>()
            .init_resource::<MidiInputConnection>()
            .add_message::<MidiInputError>()
            .add_message::<MidiData>()
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, reply)
            .add_systems(Update, debug);
    }
}

/// Settings for [`MidiInputPlugin`].
///
/// This resource must be added before [`MidiInputPlugin`] to take effect.
#[derive(Resource, Clone, Debug)]
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

#[derive(Resource)]
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
        self.sender
            .send(Message::RefreshPorts)
            .expect("Couldn't refresh input ports");
    }

    /// Connects to the given `port`.
    pub fn connect(&self, port: MidiInputPort) {
        self.sender
            .send(Message::ConnectToPort(port))
            .expect("Failed to connect to port");
    }

    /// Disconnects from the current input port.
    pub fn disconnect(&self) {
        self.sender
            .send(Message::DisconnectFromPort)
            .expect("Failed to disconnect from port");
    }

    /// Get the current input ports, and their names.
    #[must_use]
    pub fn ports(&self) -> &Vec<(String, MidiInputPort)> {
        &self.ports
    }
}

/// [`Resource`](bevy::ecs::system::Resource) for checking whether [`MidiInput`] is
/// connected to any ports.
///
/// Change detection fires whenever the connection changes.
#[derive(Resource, Default)]
pub struct MidiInputConnection {
    connected: bool,
}

impl MidiInputConnection {
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

/// A [`Message`](bevy::ecs::message::Message) for incoming midi data.
///
/// This event fires from [`CoreStage::PreUpdate`].
#[derive(Resource, Message)]
pub struct MidiData {
    pub stamp: u64,
    pub message: MidiMessage,
}

/// The [`Error`] type for midi input operations, accessible as a [`Message`](bevy::ecs::message::Message).
#[derive(Clone, Debug, Message)]
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
                    write!(f, "Couldn't (re)connect to input port: invalid port")?;
                }
                ConnectErrorKind::Other(s) => {
                    write!(f, "Couldn't (re)connect to input port: {}", s)?;
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
    mut err: MessageWriter<MidiInputError>,
    mut midi: MessageWriter<MidiData>,
) {
    while let Ok(msg) = input.receiver.try_recv() {
        match msg {
            Reply::AvailablePorts(ports) => {
                input.ports = ports;
            }
            Reply::Error(e) => {
                warn!("{}", e);
                err.write(e);
            }
            Reply::Connected => {
                conn.connected = true;
            }
            Reply::Disconnected => {
                conn.connected = false;
            }
            Reply::Midi(m) => {
                midi.write(m);
            }
        }
    }
}

fn setup(mut commands: Commands, settings: Res<MidiInputSettings>) {
    let (m_sender, m_receiver) = crossbeam_channel::unbounded::<Message>();
    let (r_sender, r_receiver) = crossbeam_channel::unbounded::<Reply>();

    let settings_clone = settings.clone();

    let task = MidiInputTask {
        receiver: m_receiver,
        sender: r_sender,
        settings: settings_clone,
        input: None,
        connection: None,
    };

    // Platform-specific task spawning
    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get().spawn(task).detach();

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        task.run_wasm().await;
    });

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

struct MidiInputTask {
    receiver: Receiver<Message>,
    sender: Sender<Reply>,
    settings: MidiInputSettings,

    // Invariant: exactly one of `input` or `connection` is Some
    input: Option<midir::MidiInput>,
    connection: Option<(midir::MidiInputConnection<()>, MidiInputPort)>,
}

impl MidiInputTask {
    /// Handle connecting to a MIDI port (shared between native and WASM)
    fn handle_connect_to_port(&mut self, port: MidiInputPort) -> Vec<Reply> {
        let mut replies = Vec::new();
        let was_connected = self.input.is_none();
        let s = self.sender.clone();
        let i = self
            .input
            .take()
            .unwrap_or_else(|| self.connection.take().unwrap().0.close().0);

        // Connect to the port (same API on all platforms)
        let conn = i.connect(
            &port,
            self.settings.port_name,
            move |stamp, message, _| {
                if message.len() != 3 {
                    return;
                }
                let _ = s.send(Reply::Midi(MidiData {
                    stamp,
                    message: [
                        message[0],
                        message.get(1).cloned().unwrap_or_default(),
                        message.get(2).cloned().unwrap_or_default(),
                    ]
                    .into(),
                }));
            },
            (),
        );

        match conn {
            Ok(conn) => {
                replies.push(Reply::Connected);
                self.connection = Some((conn, port));
                self.input = None;
            }
            Err(conn_err) => {
                replies.push(Reply::Error(ConnectionError(conn_err.kind())));
                if was_connected {
                    replies.push(Reply::Disconnected);
                }
                self.connection = None;
                self.input = Some(conn_err.into_inner());
            }
        }
        replies
    }

    /// Handle disconnecting from current MIDI port (shared between native and WASM)
    fn handle_disconnect_from_port(&mut self) -> Vec<Reply> {
        if let Some((conn, _)) = self.connection.take() {
            self.input = Some(conn.close().0);
            self.connection = None;
            vec![Reply::Disconnected]
        } else {
            Vec::new()
        }
    }

    /// Handle refreshing MIDI ports (shared between native and WASM)
    fn handle_refresh_ports(&mut self) -> Vec<Reply> {
        match &self.input {
            Some(i) => vec![get_available_ports(i)],
            None => {
                if let Some((conn, port)) = self.connection.take() {
                    let i = conn.close().0;
                    let mut replies = vec![get_available_ports(&i)];

                    let s = self.sender.clone();

                    // Reconnect to the port (same API on all platforms)
                    let conn = i.connect(
                        &port,
                        self.settings.port_name,
                        move |stamp, message, _| {
                            let _ = s.send(Reply::Midi(MidiData {
                                stamp,
                                message: [message[0], message[1], message[2]].into(),
                            }));
                        },
                        (),
                    );

                    match conn {
                        Ok(conn) => {
                            self.connection = Some((conn, port));
                            self.input = None;
                        }
                        Err(conn_err) => {
                            replies.push(Reply::Error(ConnectionError(conn_err.kind())));
                            replies.push(Reply::Disconnected);
                            self.connection = None;
                            self.input = Some(conn_err.into_inner());
                        }
                    }
                    replies
                } else {
                    Vec::new()
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn run_wasm(mut self) {
        // Initialize the input if not already done
        if self.input.is_none() && self.connection.is_none() {
            self.input = midir::MidiInput::new(self.settings.client_name).ok();
            if let Some(ref input) = self.input {
                info!("MIDI input initialized for WASM");
                let _ = self.sender.send(get_available_ports(input));
            } else {
                warn!("Failed to create MIDI input");
            }
        }

        // Main message processing loop for WASM
        loop {
            // Process messages non-blockingly
            while let Ok(msg) = self.receiver.try_recv() {
                self.handle_message(msg).await;
            }

            // Use requestAnimationFrame to yield control properly to the browser
            // This is much better than tight loops or immediate promises
            self.next_animation_frame().await;
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn next_animation_frame(&self) {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::prelude::*;

        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let window = web_sys::window().unwrap();
            let closure = Closure::once(move || {
                resolve.call0(&JsValue::UNDEFINED).unwrap();
            });
            window
                .request_animation_frame(closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        });
        wasm_bindgen_futures::JsFuture::from(promise).await.ok();
    }

    #[cfg(target_arch = "wasm32")]
    async fn handle_message(&mut self, msg: Message) {
        use Message::{ConnectToPort, DisconnectFromPort, RefreshPorts};

        let replies = match msg {
            ConnectToPort(port) => self.handle_connect_to_port(port),
            DisconnectFromPort => self.handle_disconnect_from_port(),
            RefreshPorts => self.handle_refresh_ports(),
        };

        for reply in replies {
            let _ = self.sender.send(reply);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Future for MidiInputTask {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.input.is_none() && self.connection.is_none() {
            self.input = midir::MidiInput::new(self.settings.client_name).ok();
            self.sender
                .send(get_available_ports(self.input.as_ref().unwrap()))
                .unwrap();
        }

        if let Ok(msg) = self.receiver.recv() {
            use Message::{ConnectToPort, DisconnectFromPort, RefreshPorts};

            let replies = match msg {
                ConnectToPort(port) => self.handle_connect_to_port(port),
                DisconnectFromPort => self.handle_disconnect_from_port(),
                RefreshPorts => self.handle_refresh_ports(),
            };

            for reply in replies {
                self.sender.send(reply).unwrap();
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
fn get_available_ports(input: &midir::MidiInput) -> Reply {
    for _ in 0..10 {
        let ports = input.ports();
        let ports: Result<Vec<_>, _> = ports
            .into_iter()
            .map(|p| input.port_name(&p).map(|n| (n, MidiInputPort::new(p))))
            .collect();
        if let Ok(ports) = ports {
            return Reply::AvailablePorts(ports);
        }
    }
    Reply::Error(PortRefreshError)
}

// A system which debug prints note messages
fn debug(mut midi: MessageReader<MidiData>) {
    for data in midi.read() {
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
