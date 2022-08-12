use super::{MidiMessage, KEY_RANGE};
use bevy::prelude::Plugin;
use bevy::{prelude::*, tasks::IoTaskPool};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::io::{stdin, stdout, Write};
pub use midir::Ignore;

pub struct MidiInputPlugin;

impl Plugin for MidiInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MidiInputSettings>()
            .add_startup_system(setup)
            .add_system(debug);
    }
}

fn setup(mut commands: Commands, settings: Res<MidiInputSettings>) {
    let (sender, receiver) = unbounded::<MidiRawData>();
    let thread_pool = IoTaskPool::get();
    thread_pool.spawn(handshake(sender, settings.clone())).detach();
    commands.insert_resource(MidiInput { receiver });
}

#[derive(Clone, Debug)]
pub struct MidiInputSettings {
    pub port_name: &'static str,
    pub ignore: Ignore
}

impl Default for MidiInputSettings {
    fn default() -> Self {
        Self { 
            port_name: "bevy_midi",
            ignore: Ignore::None,
        }
    }
}

pub struct MidiRawData {
    pub stamp: u64,
    pub message: MidiMessage,
}

#[allow(clippy::unused_async)]
async fn handshake(sender: Sender<MidiRawData>, settings: MidiInputSettings) {
    let mut input = String::new();
    let mut midi_in = midir::MidiInput::new("midir reading input").unwrap();
    midi_in.ignore(settings.ignore);

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
            settings.port_name,
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

pub struct MidiInput {
    pub receiver: Receiver<MidiRawData>,
}

fn debug(input: Res<MidiInput>) {
    if let Ok(data) = input.receiver.try_recv() {
        let pitch = data.message.msg[1];
        let octave = pitch / 12;
        let key = KEY_RANGE[pitch as usize % 12];

        if data.message.is_note_on() {
            debug!("NoteOn: {}{:?} - Raw: {:?}", key, octave, data.message.msg);
        }
        else if data.message.is_note_off() {
            debug!("NoteOff: {}{:?} - Raw: {:?}", key, octave, data.message.msg);
        }
        else {
            debug!("Other: {:?}", data.message.msg);
        }
    }
}
