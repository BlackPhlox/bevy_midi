use bevy::{pbr::AmbientLight, prelude::*};
use bevy_config_cam::ConfigCam;
use bevy_midi::{Midi, MidiRawData, MidiSettings, KEY_RANGE};
use crossbeam_channel::Receiver;

fn main() {
    App::build()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ConfigCam)
        .add_plugin(Midi)
        .insert_resource(MidiSettings {
            is_debug: false,
            ..Default::default()
        })
        .add_startup_system(setup.system())
        .add_system(handle_midi_input.system())
        .run();
}

#[derive(Debug)]
struct Key {
    key_val: String,
    y_reset: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let pos: Vec3 = Vec3::new(0., 0., 0.);

    let mut black_key: Handle<Scene> = asset_server.load("models/black_key.gltf#Scene0");
    let mut white_key_0: Handle<Scene> = asset_server.load("models/white_key_0.gltf#Scene0");
    let mut white_key_1: Handle<Scene> = asset_server.load("models/white_key_1.gltf#Scene0");
    let mut white_key_2: Handle<Scene> = asset_server.load("models/white_key_2.gltf#Scene0");

    //Create keyboard layout
    let bk_off = Vec3::new(0., 0.06, 0.);
    for i in 0..8 {
        spawn_note(&mut commands, 0.00, pos, &mut white_key_0, i, "C");
        spawn_note(&mut commands, 0.15, pos + bk_off, &mut black_key, i, "C#");
        spawn_note(&mut commands, 0.27, pos, &mut white_key_1, i, "D");
        spawn_note(&mut commands, 0.39, pos + bk_off, &mut black_key, i, "D#");
        spawn_note(&mut commands, 0.54, pos, &mut white_key_2, i, "E");
        spawn_note(&mut commands, 0.69, pos, &mut white_key_0, i, "F");
        spawn_note(&mut commands, 0.85, pos + bk_off, &mut black_key, i, "F#");
        spawn_note(&mut commands, 0.96, pos, &mut white_key_1, i, "G");
        spawn_note(&mut commands, 1.08, pos + bk_off, &mut black_key, i, "G#");
        spawn_note(&mut commands, 1.19, pos, &mut white_key_1, i, "A");
        spawn_note(&mut commands, 1.31, pos + bk_off, &mut black_key, i, "A#");
        spawn_note(&mut commands, 1.46, pos, &mut white_key_2, i, "B");
    }
}

fn spawn_note(
    commands: &mut Commands,
    offset_z: f32,
    pos: Vec3,
    asset: &mut Handle<Scene>,
    oct: i32,
    key: &str,
) {
    commands
        .spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - offset_z - (1.61 * oct as f32)),
                scale: Vec3::new(10., 10., 10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key {
            key_val: format!("{}{}", key, oct),
            y_reset: pos.y,
        })
        .with_children(|cell| {
            cell.spawn_scene(asset.clone());
        });
}

fn handle_midi_input(
    receiver: Res<Receiver<MidiRawData>>,
    mut query: Query<(&Key, &mut Transform)>,
    settings: Res<MidiSettings>,
) {
    if let Ok(data) = receiver.try_recv() {
        let [event, index, value] = data.message;
        let off = index % 12;
        let oct = index.overflowing_div(12).0;
        let key_str = KEY_RANGE.iter().nth(off.into()).unwrap();

        if event.eq(&settings.note_on) {
            for (key, mut transform) in query.iter_mut() {
                if key.key_val.eq(&format!("{}{}", key_str, oct).to_string()) {
                    if transform.translation.y > -0.1 {
                        transform.translation = Vec3::new(
                            transform.translation.x,
                            transform.translation.y - 0.1,
                            transform.translation.z,
                        );
                    }
                }
            }
        } else if event.eq(&settings.note_off) {
            for (key, mut transform) in query.iter_mut() {
                if key.key_val.eq(&format!("{}{}", key_str, oct).to_string()) {
                    transform.translation = Vec3::new(
                        transform.translation.x,
                        key.y_reset,
                        transform.translation.z,
                    );
                }
            }
        } else {
        }
    }
}
