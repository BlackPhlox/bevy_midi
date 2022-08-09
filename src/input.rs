use super::{MidiMessage, KEY_RANGE};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::Plugin;
use bevy::{prelude::*, tasks::IoTaskPool};
use crossbeam_channel::{unbounded, Receiver, Sender};
use midir::{Ignore, MidiInput};
use std::io::{stdin, stdout, Write};

pub struct MidiInputPlugin;

impl Plugin for MidiInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MidiSettings>()
            .add_startup_system(setup)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_debug)
                    .with_system(debug_midi),
            );
    }
}

fn setup(mut commands: Commands) {
    let (sender, receiver) = unbounded::<MidiRawData>();
    let thread_pool = IoTaskPool::get();
    thread_pool.spawn(handshake(sender)).detach();
    commands.insert_resource(receiver);
}

#[derive(Clone, Copy)]
pub struct MidiSettings {
    pub is_debug: bool,
}

impl Default for MidiSettings {
    fn default() -> Self {
        Self { is_debug: true }
    }
}

fn run_if_debug(settings: Res<MidiSettings>) -> ShouldRun {
    if settings.is_debug {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub struct MidiRawData {
    pub stamp: u64,
    pub message: MidiMessage,
}

#[allow(clippy::unused_async)]
async fn handshake(sender: Sender<MidiRawData>) {
    let mut input = String::new();
    let mut midi_in: MidiInput = MidiInput::new("midir reading input").unwrap();
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        //0 => return Err("No input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush().unwrap();
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            in_ports
                .get(input.trim().parse::<usize>().unwrap())
                .ok_or("Invalid input port selected")
                .unwrap()
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port).unwrap();

    let sender = sender;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in
        .connect(
            in_port,
            "midir-read-input",
            move |stamp, message, _| {
                sender
                    .send(MidiRawData {
                        stamp,
                        message: [message[0], message[1], message[2]].into(),
                    })
                    .unwrap();
            },
            (),
        )
        .unwrap();

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    input.clear();
    stdin().read_line(&mut input).unwrap(); // wait for next enter key press

    println!("Closing connection");
}

fn debug_midi(receiver: Res<Receiver<MidiRawData>>) {
    if let Ok(data) = receiver.try_recv() {
        //info!("received message: {:?}", data.message);
        translate(data.message);
    }
}

pub fn translate(message: MidiMessage) -> (u8, String) {
    let msg = message.msg[1];
    let off = msg % 12;
    let oct = msg.overflowing_div(12).0;

    let midi_type = if message.is_note_on() {
        "NoteOn"
    } else if message.is_note_off() {
        "NoteOff"
    } else {
        "Other"
    };

    let k = KEY_RANGE.iter().nth(off.into()).unwrap();
    println!("{}:{}{:?} - Raw: {:?}", midi_type, k, oct, message.msg,);
    (message.msg[0], format!("{}{:?}", k, oct))
}
