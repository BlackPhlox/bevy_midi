use bevy::prelude::*;
use bevy_midi::Midi;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(Midi)
        .run();
}
