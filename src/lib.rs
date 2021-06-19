use std::{error::{Error}, io::{Write, stdin, stdout}, sync::{Arc, Mutex}, thread};

use bevy::prelude::{AppBuilder, Commands, EventWriter, IntoSystem, Plugin, Res, ResMut};
use midir::{Ignore, MidiInput};

pub struct Midi;
impl Plugin for Midi {
    fn build(&self, app: &mut AppBuilder) {
        app
        .init_resource::<MidiLog>()
        .add_startup_system(midi_setup.system())
        .add_event::<MidiEvent>()
        .add_system(read_midi.system());
    }
}

pub struct MidiLog {
    data: Arc<Mutex<[u8]>>
}

impl Default for MidiLog {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new([0]))
        }
    }
}

fn midi_setup(mut commands: Commands, mut midi_events: EventWriter<MidiEvent>, mut log: ResMut<MidiLog>) {
    let thread_data = log.data.clone();

    thread::spawn(|| {
        {
            let mut input = String::new();
    
            let mut midi_in : MidiInput = MidiInput::new("midir reading input").unwrap();
            midi_in.ignore(Ignore::None);
            
            // Get an input port (read from console if multiple are available)
            let in_ports = midi_in.ports();
            let in_port = match in_ports.len() {
                //0 => return Err("no input port found".into()),
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
                    stdout().flush().unwrap();
                    let mut input = String::new();
                    stdin().read_line(&mut input).unwrap();
                    in_ports.get(input.trim().parse::<usize>().unwrap())
                            .ok_or("invalid input port selected").unwrap()
                }
            };
            
            println!("\nOpening connection");
            let in_port_name = midi_in.port_name(in_port).unwrap();

            
            
            // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
            let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, message, _ | {
                //println!("{}: {:?} (len = {})", stamp, message, message.len());
                translate(stamp, message);
                
                let mut data = thread_data.lock().unwrap();
                data[0] = message[0];
                /*midi_events.send(MidiEvent {
                    stamp,
                    message: Box::new(*message),
                });*/
                
            }, ()).unwrap();

            println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

            input.clear();
            stdin().read_line(&mut input).unwrap(); // wait for next enter key press

            println!("Closing connection");
        }
    });
}

fn read_midi(mut log: ResMut<MidiLog>){
    println!("{:?}", log.data);
}

pub struct MidiEvent {
    pub stamp: u64,
    pub message: Box<[u8]>,
}

const KEY_RANGE: [&str; 12] = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];

fn translate(stamp: u64, message: &[u8]){
    let msg = message[1];
    let off = msg % 12;
    let oct = msg.overflowing_div(12).0;

    let k = KEY_RANGE.iter().nth(off.into()).unwrap();
    println!("{}{:?}", k, oct);
    println!("{}: {:?} (len = {})", stamp, message, message.len());
}

/*
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