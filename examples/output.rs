use bevy::{
    color::palettes::basic::{GREEN, RED},
    prelude::*,
};
use bevy_midi::prelude::*;

const KEY_PORT_MAP: [(KeyCode, usize); 10] = [
    (KeyCode::Digit0, 0),
    (KeyCode::Digit1, 1),
    (KeyCode::Digit2, 2),
    (KeyCode::Digit3, 3),
    (KeyCode::Digit4, 4),
    (KeyCode::Digit5, 5),
    (KeyCode::Digit6, 6),
    (KeyCode::Digit7, 7),
    (KeyCode::Digit8, 8),
    (KeyCode::Digit9, 9),
];

const KEY_NOTE_MAP: [(KeyCode, u8); 7] = [
    (KeyCode::KeyA, 57),
    (KeyCode::KeyB, 59),
    (KeyCode::KeyC, 60),
    (KeyCode::KeyD, 62),
    (KeyCode::KeyE, 64),
    (KeyCode::KeyF, 65),
    (KeyCode::KeyG, 67),
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(MidiOutputSettings {
            port_name: "output",
        })
        .add_plugins(MidiOutputPlugin)
        .add_systems(
            Update,
            (
                refresh_ports,
                connect,
                disconnect,
                play_notes,
                show_ports,
                show_connection,
            ),
        )
        .add_systems(Startup, setup)
        .run();
}

fn refresh_ports(input: Res<ButtonInput<KeyCode>>, output: Res<MidiOutput>) {
    if input.just_pressed(KeyCode::KeyR) {
        output.refresh_ports();
    }
}

fn connect(input: Res<ButtonInput<KeyCode>>, output: Res<MidiOutput>) {
    for (keycode, index) in &KEY_PORT_MAP {
        if input.just_pressed(*keycode) {
            if let Some((_, port)) = output.ports().get(*index) {
                output.connect(port.clone());
            }
        }
    }
}

fn disconnect(input: Res<ButtonInput<KeyCode>>, output: Res<MidiOutput>) {
    if input.just_pressed(KeyCode::Escape) {
        output.disconnect();
    }
}

fn play_notes(input: Res<ButtonInput<KeyCode>>, output: Res<MidiOutput>) {
    for (keycode, note) in &KEY_NOTE_MAP {
        if input.just_pressed(*keycode) {
            output.send([0b1001_0000, *note, 127].into()); // Note on, channel 1, max velocity
        }
        if input.just_released(*keycode) {
            output.send([0b1000_0000, *note, 127].into()); // Note on, channel 1, max velocity
        }
    }
}

#[derive(Component)]
pub struct Instructions;

#[derive(Component)]
pub struct ConnectStatus;

fn show_ports(output: Res<MidiOutput>, mut instructions: Query<&mut TextSpan, With<Instructions>>) {
    if output.is_changed() {
        let text = &mut instructions.single_mut();
        text.0 = "Available output ports:\n\n".to_string();
        for (i, (name, _)) in output.ports().iter().enumerate() {
            text.0
                .push_str(format!("Port {:?}: {:?}\n", i, name).as_str());
        }
    }
}

fn show_connection(
    connection: Res<MidiOutputConnection>,
    mut instructions: Query<(&mut Text, &mut TextColor), With<ConnectStatus>>,
) {
    if connection.is_changed() {
        let (text, color) = &mut instructions.single_mut();
        if connection.is_connected() {
            text.0 = "Connected".to_string();
            color.0 = GREEN.into();
        } else {
            text.0 = "Disconnected".to_string();
            color.0 = RED.into();
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Text::default(),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 30.0,
                ..default()
            },
            Instructions,
        ))
        .with_children(|commands| {
            commands.spawn((
                TextSpan::new(
                    "INSTRUCTIONS \n\
                    R - Refresh ports \n\
                    A to G - Play note\n\
                    0 to 9 - Connect to port \n\
                    Escape - Disconnect from current port \n",
                ),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Instructions,
            ));
            commands.spawn((
                TextSpan::new("Disconnected\n"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::linear_rgb(1.0, 0., 0.)),
                ConnectStatus,
            ));
        });
}
