use std::{
    io::{stdin, stdout, Write},
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

use bevy::{ecs::schedule::ShouldRun, prelude::{AppBuilder, EventReader, EventWriter, IntoSystem, Plugin, Res, ResMut, SystemSet}};
use midir::{Ignore, MidiInput};

pub struct Midi;
impl Plugin for Midi {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<MidiSettings>()
            .init_resource::<MidiLog>()
            .add_event::<MidiEvent>()
            .insert_resource(MidiStamp { stamp: 0_u64 })
            .add_startup_system(midi_setup.system())
            .add_system(midi_sender.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_debug.system())
                    .before("input")
                    .with_system(midi_listener.system())
            );
    }
}

pub struct MidiSettings {
    pub is_debug: bool,
}

impl Default for MidiSettings {
    fn default() -> Self {
        Self { is_debug: true }
    }
}

pub struct MidiLog {
    stamp: Arc<Mutex<[u64]>>,
    message: Arc<Mutex<[u8]>>,
}

impl MidiLog {
    pub fn get(&mut self) -> (MutexGuard<[u64]>, MutexGuard<[u8]>) {
        let stmp = self.stamp.lock().unwrap();
        let msg = self.message.lock().unwrap();
        (stmp, msg)
    }
}

impl Default for MidiLog {
    fn default() -> Self {
        Self {
            stamp: Arc::new(Mutex::new([0_u64])),
            message: Arc::new(Mutex::new([0, 0, 0])),
        }
    }
}

fn midi_setup(log: Res<MidiLog>) {
    let thread_stamp = log.stamp.clone();
    let thread_msg = log.message.clone();

    let _t = thread::Builder::new().name("Midi Input".into()).spawn(|| {
        {
            let mut input = String::new();

            let mut midi_in: MidiInput = MidiInput::new("midir reading input").unwrap();
            midi_in.ignore(Ignore::None);

            // Get an input port (read from console if multiple are available)
            let in_ports = midi_in.ports();
            let in_port = match in_ports.len() {
                //0 => return Err("no input port found".into()),
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
                        .ok_or("invalid input port selected")
                        .unwrap()
                }
            };

            println!("\nOpening connection");
            let in_port_name = midi_in.port_name(in_port).unwrap();

            // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
            let _conn_in = midi_in
                .connect(
                    in_port,
                    "midir-read-input",
                    move |stamp, message, _| {

                        let mut data = thread_msg.lock().unwrap();
                        let mut stmp = thread_stamp.lock().unwrap();

                        for (i, m) in message.iter().enumerate() {
                            data[i] = *m;
                        }

                        stmp[0] = stamp;
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
    });
}

struct MidiStamp {
    stamp: u64,
}

pub struct MidiEvent {
    pub message: [u8; 3],
}

fn midi_sender(
    mut log: ResMut<MidiLog>,
    mut rs_log: ResMut<MidiStamp>,
    mut midi_events: EventWriter<MidiEvent>,
) {
    let (stamps, msg) = log.get();
    let stamp = stamps[0];

    if !stamp.eq(&rs_log.stamp) {
        midi_events.send(MidiEvent {
            message: [msg[0], msg[1], msg[2]],
        });

        rs_log.stamp = stamp;
    }
}

fn run_if_debug(
    settings: Res<MidiSettings>
) -> ShouldRun
{
    if settings.is_debug {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn midi_listener(mut events: EventReader<MidiEvent>) {
    for midi_event in events.iter() {
        translate(&midi_event.message);
    }
}

const KEY_RANGE: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub fn translate(message: &[u8]) -> (u8, String) {
    let msg = message[1];
    let off = msg % 12;
    let oct = msg.overflowing_div(12).0;

    let midi_type = if message[0].eq(&144) {
        "NoteOn"
    } else if message[0].eq(&128) {
        "NoteOff"
    } else {
        "Other"
    };

    let k = KEY_RANGE.iter().nth(off.into()).unwrap();
    println!(
        "{}:{}{:?} - Raw: {}",
        midi_type,
        k,
        oct,
        format!("{:?} (len = {})", message, message.len())
    );
    (message[0], format!("{}{:?}", k, oct))
}

/*
TODO: Re-implement error handling/aggregation

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!("Choosing the only available input port: {}", midi_in.port_name(&in_ports[0]).unwrap());
            &in_ports[0]
        },
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports.get(input.trim().parse::<usize>()?)
            .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, message, _| {
        //println!("{}: {:?} (len = {})", stamp, message, message.len());
        translate(stamp, message);
    }, ())?;

    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}
*/
