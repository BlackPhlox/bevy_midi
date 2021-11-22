#![warn(clippy::pedantic)]

use bevy::prelude::*;
use bevy_midi::Midi;

fn main() {
    App::build()
        .add_plugins(MinimalPlugins)
        .add_plugin(Midi)
        .run();
}
