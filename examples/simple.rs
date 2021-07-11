use bevy::prelude::*;
use bevy_midi::{translate, Midi, MidiEvent, MidiSettings};

fn main() {
    App::build()
        .add_plugins(MinimalPlugins)
        .add_plugin(Midi)
        .insert_resource(MidiSettings { 
            is_debug: false, 
            ..Default::default() 
        })
        .add_system(midi_listener.system())
        .run();
}

fn midi_listener(mut events: EventReader<MidiEvent>, settings: Res<MidiSettings>) {
    for midi_event in events.iter() {
        translate(&midi_event.message, *settings);
    }
}
