use bevy::prelude::*;
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

const KEY_NOTE_MAP: [(KeyCode, u8); 7] = [
    (KeyCode::A, 57),
    (KeyCode::B, 59),
    (KeyCode::C, 60),
    (KeyCode::D, 62),
    (KeyCode::E, 64),
    (KeyCode::F, 65),
    (KeyCode::G, 67),
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

fn refresh_ports(input: Res<Input<KeyCode>>, output: Res<MidiOutput>) {
    if input.just_pressed(KeyCode::R) {
        output.refresh_ports();
    }
}

fn connect(input: Res<Input<KeyCode>>, output: Res<MidiOutput>) {
    for (keycode, index) in &KEY_PORT_MAP {
        if input.just_pressed(*keycode) {
            if let Some((_, port)) = output.ports().get(*index) {
                output.connect(port.clone());
            }
        }
    }
}

fn disconnect(input: Res<Input<KeyCode>>, output: Res<MidiOutput>) {
    if input.just_pressed(KeyCode::Escape) {
        output.disconnect();
    }
}

fn play_notes(input: Res<Input<KeyCode>>, output: Res<MidiOutput>) {
    for (keycode, note) in &KEY_NOTE_MAP {
        let key: num::u7 = (*note).into();
        let vel: num::u7 = 127.into();
        if input.just_pressed(*keycode) {
            output.send(LiveEvent::Midi {
                channel: 0.into(),
                message: midly::MidiMessage::NoteOn { key, vel },
            }); // Note on, channel 1, max velocity
        }
        if input.just_released(*keycode) {
            output.send(LiveEvent::Midi {
                channel: 0.into(),
                message: midly::MidiMessage::NoteOff { key, vel },
            }); // Note on, channel 1, max velocity
        }
    }
}

#[derive(Component)]
pub struct Instructions;

fn show_ports(output: Res<MidiOutput>, mut instructions: Query<&mut Text, With<Instructions>>) {
    if output.is_changed() {
        let text_section = &mut instructions.single_mut().sections[1];
        text_section.value = "Available output ports:\n\n".to_string();
        for (i, (name, _)) in output.ports().iter().enumerate() {
            text_section
                .value
                .push_str(format!("Port {:?}: {:?}\n", i, name).as_str());
        }
    }
}

fn show_connection(
    connection: Res<MidiOutputConnection>,
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
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TextBundle {
            text: Text {
                sections: vec![
                    TextSection::new(
                        "INSTRUCTIONS \n\
                    R - Refresh ports \n\
                    A to G - Play note \n\
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
                ..Default::default()
            },
            ..default()
        },
        Instructions,
    ));
}
