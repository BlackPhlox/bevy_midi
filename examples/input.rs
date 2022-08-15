use bevy::{log::*, prelude::*};
use bevy_midi::input::*;

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
        .insert_resource(LogSettings {
            filter: "bevy_midi=debug".to_string(),
            level: Level::WARN,
        })
        .insert_resource(MidiInputSettings {
            port_name: "input",
            client_name: "input",
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(MidiInputPlugin)
        .add_system(refresh_ports)
        .add_system(connect)
        .add_system(disconnect)
        .add_system(show_ports)
        .add_system(show_connection)
        .add_startup_system(setup)
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
            text_section.value = "Connected".to_string();
            text_section.style.color = Color::GREEN;
        } else {
            text_section.value = "Disconnected".to_string();
            text_section.style.color = Color::RED;
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn_bundle(TextBundle {
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
                        "Disconnected",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::RED,
                        },
                    ),
                ],
                alignment: TextAlignment::TOP_LEFT,
            },
            ..default()
        })
        .insert(Instructions);
}
