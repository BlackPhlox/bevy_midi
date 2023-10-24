use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_midi::prelude::*;

const KEY_PORT_MAP: [(KeyCode, usize); 10] = [
    (KeyCode::Key0, 0),
    (KeyCode::Key1, 1),
    (KeyCode::Key2, 2),
    (KeyCode::Key3, 3),
    (KeyCode::Key4, 4),
    (KeyCode::Key5, 5),
    (KeyCode::Key6, 6),
    (KeyCode::Key7, 7),
    (KeyCode::Key8, 8),
    (KeyCode::Key9, 9),
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
        }))
        .add_plugins(MidiInputPlugin)
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
        .add_systems(Startup, setup)
        .run();
}

fn refresh_ports(keys: Res<Input<KeyCode>>, input: Res<MidiInput>) {
    if keys.just_pressed(KeyCode::R) {
        input.refresh_ports();
    }
}

fn connect(keys: Res<Input<KeyCode>>, input: Res<MidiInput>) {
    for (keycode, index) in &KEY_PORT_MAP {
        if keys.just_pressed(*keycode) {
            if let Some((_, port)) = input.ports().get(*index) {
                input.connect(port.clone());
            }
        }
    }
}

fn disconnect(keys: Res<Input<KeyCode>>, input: Res<MidiInput>) {
    if keys.just_pressed(KeyCode::Escape) {
        input.disconnect();
    }
}

#[derive(Component)]
pub struct Instructions;

fn show_ports(input: Res<MidiInput>, mut instructions: Query<&mut Text, With<Instructions>>) {
    if input.is_changed() {
        let text_section = &mut instructions.single_mut().sections[1];
        text_section.value = "Available input ports:\n\n".to_string();
        for (i, (name, _)) in input.ports().iter().enumerate() {
            text_section
                .value
                .push_str(format!("Port {:?}: {:?}\n", i, name).as_str());
        }
    }
}

fn show_connection(
    connection: Res<MidiInputConnection>,
    mut instructions: Query<&mut Text, With<Instructions>>,
) {
    if connection.is_changed() {
        let text_section = &mut instructions.single_mut().sections[2];
        if connection.is_connected() {
            text_section.value = "Connected\n".to_string();
            text_section.style.color = Color::GREEN;
        } else {
            text_section.value = "Disconnected\n".to_string();
            text_section.style.color = Color::RED;
        }
    }
}

fn show_last_message(
    mut midi_data: EventReader<MidiData>,
    mut instructions: Query<&mut Text, With<Instructions>>,
) {
    for data in midi_data.iter() {
        let text_section = &mut instructions.single_mut().sections[3];
        text_section.value = format!(
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
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TextBundle {
            text: Text {
                sections: vec![
                    TextSection::new(
                        "INSTRUCTIONS \n\
                        R - Refresh ports \n\
                        0 to 9 - Connect to port \n\
                        Escape - Disconnect from current port \n",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::BLACK,
                    }),
                    TextSection::new(
                        "Disconnected\n",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::RED,
                        },
                    ),
                    TextSection::new(
                        "Last Message:",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    ),
                ],
                ..Default::default()
            },
            ..default()
        },
        Instructions,
    ));
}
