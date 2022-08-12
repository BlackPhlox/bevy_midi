use bevy::{prelude::*, log::*};
use bevy_midi::input::MidiInputPlugin;

fn main() {
    App::new()
        .insert_resource(LogSettings { level: Level::DEBUG, ..default() })
        .add_plugin(LogPlugin)
        .add_plugins(MinimalPlugins)
        .add_plugin(MidiInputPlugin)
        .run();
}
