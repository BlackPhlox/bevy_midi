use bevy::{
    log::{Level, LogPlugin},
    pbr::AmbientLight,
    prelude::*,
};
use bevy_midi::prelude::*;
use bevy_mod_picking::prelude::{DefaultPickingPlugins, *};

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::WARN,
            filter: "bevy_midi=debug".to_string(),
            ..default()
        }))
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(MidiInputPlugin)
        .init_resource::<MidiInputSettings>()
        .add_plugins(MidiOutputPlugin)
        .init_resource::<MidiOutputSettings>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_midi_input,
                connect_to_first_input_port,
                connect_to_first_output_port,
                display_press,
                display_release,
            ),
        )
        .run();
}

#[derive(Component, Debug)]
struct Key {
    key_val: String,
    y_reset: f32,
}

#[derive(Component)]
struct PressedKey;

#[rustfmt::skip]
fn setup(
    mut cmds: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mid = -6.3;

    // light
    cmds.spawn((
        PointLight::default(),
        Transform::from_xyz(0.0, 6.0, mid)
    ));

    //Camera
    cmds.spawn((
        Camera3d::default(),
        Msaa::Sample4,
        Transform::from_xyz(8., 5., mid).looking_at(Vec3::new(0., 0., mid), Vec3::Y)
    ));

    let pos: Vec3 = Vec3::new(0., 0., 0.);

    let mut black_key: Handle<Mesh> = asset_server.load("models/black_key.gltf#Mesh0/Primitive0");
    let mut white_key_0: Handle<Mesh> = asset_server.load("models/white_key_0.gltf#Mesh0/Primitive0");
    let mut white_key_1: Handle<Mesh> = asset_server.load("models/white_key_1.gltf#Mesh0/Primitive0");
    let mut white_key_2: Handle<Mesh> = asset_server.load("models/white_key_2.gltf#Mesh0/Primitive0");
    let b_mat = materials.add(Color::rgb(0.1, 0.1, 0.1));
    let w_mat = materials.add(Color::rgb(1.0, 1.0, 1.0));

    //Create keyboard layout
    let pos_black = pos + Vec3::new(0., 0.06, 0.);

    for i in 0..8 {
        spawn_note(&mut cmds, &w_mat, 0.00, pos, &mut white_key_0, i, "C");
        spawn_note(&mut cmds, &b_mat, 0.15, pos_black, &mut black_key, i, "C#/Db");
        spawn_note(&mut cmds, &w_mat, 0.27, pos, &mut white_key_1, i, "D");
        spawn_note(&mut cmds, &b_mat, 0.39, pos_black, &mut black_key, i, "D#/Eb");
        spawn_note(&mut cmds, &w_mat, 0.54, pos, &mut white_key_2, i, "E");
        spawn_note(&mut cmds, &w_mat, 0.69, pos, &mut white_key_0, i, "F");
        spawn_note(&mut cmds, &b_mat, 0.85, pos_black, &mut black_key, i, "F#/Gb");
        spawn_note(&mut cmds, &w_mat, 0.96, pos, &mut white_key_1, i, "G");
        spawn_note(&mut cmds, &b_mat, 1.08, pos_black, &mut black_key, i, "G#/Ab");
        spawn_note(&mut cmds, &w_mat, 1.19, pos, &mut white_key_1, i, "A");
        spawn_note(&mut cmds, &b_mat, 1.31, pos_black, &mut black_key, i, "A#/Bb");
        spawn_note(&mut cmds, &w_mat, 1.46, pos, &mut white_key_2, i, "B");
    }
}

fn spawn_note(
    commands: &mut Commands,
    mat: &Handle<StandardMaterial>,
    offset_z: f32,
    pos: Vec3,
    asset: &mut Handle<Mesh>,
    oct: i32,
    key: &str,
) {
    commands.spawn((
        Mesh3d(asset.clone()),
        MeshMaterial3d(mat.clone()),
        Transform {
            translation: Vec3::new(pos.x, pos.y, pos.z - offset_z - (1.61 * oct as f32)),
            scale: Vec3::new(10., 10., 10.),
            ..Default::default()
        },
        Key {
            key_val: format!("{}{}", key, oct),
            y_reset: pos.y,
        },
        PickableBundle::default(),
        On::<Pointer<Down>>::target_commands_mut(|_click, entity_commands| {
            entity_commands.insert(PressedKey);
        }),
        On::<Pointer<Up>>::target_commands_mut(|_click, entity_commands| {
            entity_commands.remove::<PressedKey>();
        }),
    ));
}

fn display_press(mut query: Query<&mut Transform, With<PressedKey>>) {
    for mut t in &mut query {
        t.translation.y = -0.05;
    }
}

fn display_release(mut query: Query<(&mut Transform, &Key), Without<PressedKey>>) {
    for (mut t, k) in &mut query {
        t.translation.y = k.y_reset;
    }
}

fn handle_midi_input(
    mut commands: Commands,
    mut midi_events: EventReader<MidiData>,
    query: Query<(Entity, &Key)>,
) {
    for data in midi_events.read() {
        let [_, index, _value] = data.message.msg;
        let off = index % 12;
        let oct = index.overflowing_div(12).0;
        let key_str = KEY_RANGE.iter().nth(off.into()).unwrap();

        if data.message.is_note_on() {
            for (entity, key) in query.iter() {
                if key.key_val.eq(&format!("{}{}", key_str, oct).to_string()) {
                    commands.entity(entity).insert(PressedKey);
                }
            }
        } else if data.message.is_note_off() {
            for (entity, key) in query.iter() {
                if key.key_val.eq(&format!("{}{}", key_str, oct).to_string()) {
                    commands.entity(entity).remove::<PressedKey>();
                }
            }
        } else {
        }
    }
}

fn connect_to_first_input_port(input: Res<MidiInput>) {
    if input.is_changed() {
        if let Some((_, port)) = input.ports().get(0) {
            input.connect(port.clone());
        }
    }
}

fn connect_to_first_output_port(input: Res<MidiOutput>) {
    if input.is_changed() {
        if let Some((_, port)) = input.ports().get(0) {
            input.connect(port.clone());
        }
    }
}
