use bevy::prelude::*;
use bevy_midi::Midi;

// Uses default behavior, logs the 5 latest osc messages
fn main() {
    App::build()
        .add_plugins(MinimalPlugins)
        .add_plugin(Midi)
        .run();
}
