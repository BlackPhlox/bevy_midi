use bevy::prelude::*;
use bevy_midi::MidiInputPlugin;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(MidiInputPlugin)
        .run();
}
