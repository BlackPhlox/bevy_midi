use bevy::{
    color::palettes::basic::{GREEN, RED},
    log::{Level, LogPlugin},
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

fn main() {
    App::new()
        .insert_resource(MidiInputSettings {
            port_name: "input",
            client_name: "input",
            ..default()
        })
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::WARN,
            filter: "bevy_midi=debug".to_string(),
            ..default()
        }))
        .add_plugins(MidiInputPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                refresh_ports,
                connect,
                disconnect,
                show_ports,
                show_connection,
                show_last_message,
            ),
        )
        .run();
}

fn refresh_ports(keys: Res<ButtonInput<KeyCode>>, input: Res<MidiInput>) {
    if keys.just_pressed(KeyCode::KeyR) {
        input.refresh_ports();
    }
}

fn connect(keys: Res<ButtonInput<KeyCode>>, input: Res<MidiInput>) {
    for (keycode, index) in &KEY_PORT_MAP {
        if keys.just_pressed(*keycode) {
            if let Some((_, port)) = input.ports().get(*index) {
                input.connect(port.clone());
            }
        }
    }
}

fn disconnect(keys: Res<ButtonInput<KeyCode>>, input: Res<MidiInput>) {
    if keys.just_pressed(KeyCode::Escape) {
        input.disconnect();
    }
}

#[derive(Component)]
pub struct InputPorts;

#[derive(Component)]
pub struct ConnectStatus;

#[derive(Component)]
pub struct Messages;

fn show_ports(
    input: Res<MidiInput>,
    mut instructions: Query<&mut TextSpan, With<InputPorts>>,
) -> Result {
    if input.is_changed() {
        let text = &mut instructions.single_mut()?;
        text.0 = "Available input ports:\n\n".to_string();
        for (i, (name, _)) in input.ports().iter().enumerate() {
            text.0
                .push_str(format!("Port {:?}: {:?}\n", i, name).as_str());
        }
    }
    Ok(())
}

fn show_connection(
    connection: Res<MidiInputConnection>,
    mut instructions: Query<(&mut TextSpan, &mut TextColor), With<ConnectStatus>>,
) -> Result {
    if connection.is_changed() {
        let (text, color) = &mut instructions.single_mut()?;
        if connection.is_connected() {
            text.0 = "Connected\n".to_string();
            color.0 = GREEN.into();
        } else {
            text.0 = "Disconnected\n".to_string();
            color.0 = RED.into();
        }
    }
    Ok(())
}

fn show_last_message(
    mut midi_data: EventReader<MidiData>,
    mut instructions: Query<&mut TextSpan, With<Messages>>,
) -> Result {
    for data in midi_data.read() {
        let text = &mut instructions.single_mut()?;
        text.0 = format!(
            "Last Message: {} - {:?}",
            if data.message.is_note_on() {
                "NoteOn"
            } else if data.message.is_note_off() {
                "NoteOff"
            } else {
                "Other"
            },
            data.message.msg
        );
    }
    Ok(())
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
        ))
        .with_children(|commands| {
            commands.spawn((
                TextSpan::new(
                    "INSTRUCTIONS \n\
                                       R - Refresh ports \n\
                                       0 to 9 - Connect to port \n\
                                       Escape - Disconnect from current port \n",
                ),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            commands.spawn((
                TextSpan::default(),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                InputPorts,
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

            commands.spawn((
                TextSpan::new("Last Message:"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::BLACK),
                Messages,
            ));
        });
}
