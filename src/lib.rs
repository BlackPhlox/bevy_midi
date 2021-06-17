use bevy::prelude::{AppBuilder, Commands, IntoSystem, Plugin};

pub struct Midi;
impl Plugin for Midi {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(osc_setup.system());
    }
}

fn osc_setup(mut commands: Commands) {

}

