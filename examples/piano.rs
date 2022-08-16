use bevy::{
    log::{Level, LogSettings},
    pbr::AmbientLight,
    prelude::*,
};
use bevy_midi::{input::*, KEY_RANGE};
use bevy_mod_picking::{PickingCameraBundle, DefaultPickingPlugins, PickableBundle, PickingEvent, HoverEvent, SelectionEvent};

fn main() {
    App::new()
        .insert_resource(LogSettings {
            filter: "bevy_midi=debug".to_string(),
            level: Level::WARN,
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(MidiInputPlugin)
        .insert_resource(MidiInputSettings {
            port_name: "piano_example",
            ..default()
        })
        .add_startup_system(setup)
        .add_system(handle_midi_input)
        .add_system(connect_to_first_port)
        .add_system_to_stage(CoreStage::PostUpdate, print_events)
        .add_system(display_press)
        .add_system(display_release)
        .run();
}

#[derive(Component, Debug)]
struct Key {
    key_val: String,
    y_reset: f32,
}

pub fn print_events(mut events: EventReader<PickingEvent>, mut commands: Commands) {
    for event in events.iter() {
        let entity = match event {
            PickingEvent::Selection(SelectionEvent::JustSelected(e)) => e,
            PickingEvent::Selection(SelectionEvent::JustDeselected(e)) => e,
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => e,
            PickingEvent::Hover(HoverEvent::JustLeft(e)) => e,
            PickingEvent::Clicked(e) => e,
        };
        commands.entity(*entity).insert(PressedKey);
    }
}

#[derive(Component)]
struct PressedKey;

fn setup(mut commands: Commands,mut materials: ResMut<Assets<StandardMaterial>>, asset_server: Res<AssetServer>) {
    let mid = -6.3;

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(0.0, 6.0, mid),
        ..Default::default()
    });

    //Camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(8., 5., mid).looking_at(Vec3::new(0., 0., mid), Vec3::Y),
        ..Default::default()
    }).insert_bundle(PickingCameraBundle::default());

    let pos: Vec3 = Vec3::new(0., 0., 0.);

    let mut black_key: Handle<Mesh> = asset_server.load("models/black_key.gltf#Mesh0/Primitive0");
    let mut white_key_0: Handle<Mesh> = asset_server.load("models/white_key_0.gltf#Mesh0/Primitive0");
    let mut white_key_1: Handle<Mesh> = asset_server.load("models/white_key_1.gltf#Mesh0/Primitive0");
    let mut white_key_2: Handle<Mesh> = asset_server.load("models/white_key_2.gltf#Mesh0/Primitive0");
    let b_mat = materials.add(Color::rgb(0.1, 0.1, 0.1).into());
    let w_mat = materials.add(Color::rgb(1.0, 1.0, 1.0).into());

    //Create keyboard layout
    let bk_off = Vec3::new(0., 0.06, 0.);
    for i in 0..8 {
        spawn_note(&mut commands, &w_mat, 0.00, pos, &mut white_key_0, i, "C");
        spawn_note(&mut commands, &b_mat, 0.15, pos + bk_off, &mut black_key, i, "C#");
        spawn_note(&mut commands, &w_mat, 0.27, pos, &mut white_key_1, i, "D");
        spawn_note(&mut commands, &b_mat, 0.39, pos + bk_off, &mut black_key, i, "D#");
        spawn_note(&mut commands, &w_mat, 0.54, pos, &mut white_key_2, i, "E");
        spawn_note(&mut commands, &w_mat, 0.69, pos, &mut white_key_0, i, "F");
        spawn_note(&mut commands, &b_mat, 0.85, pos + bk_off, &mut black_key, i, "F#");
        spawn_note(&mut commands, &w_mat, 0.96, pos, &mut white_key_1, i, "G");
        spawn_note(&mut commands, &b_mat, 1.08, pos + bk_off, &mut black_key, i, "G#");
        spawn_note(&mut commands, &w_mat, 1.19, pos, &mut white_key_1, i, "A");
        spawn_note(&mut commands, &b_mat, 1.31, pos + bk_off, &mut black_key, i, "A#");
        spawn_note(&mut commands, &w_mat, 1.46, pos, &mut white_key_2, i, "B");
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
    commands
        .spawn_bundle(PbrBundle {
            mesh: asset.clone(),
            material: mat.clone(),
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - offset_z - (1.61 * oct as f32)),
                scale: Vec3::new(10., 10., 10.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Key {
            key_val: format!("{}{}", key, oct),
            y_reset: pos.y,
        })
        .insert_bundle(PickableBundle::default());
}

fn display_press(
    mut query: Query<&mut Transform, With<PressedKey>>
){
    for mut t in &mut query {
        t.translation.y = -0.05;
    }
}

fn display_release(
    mut query: Query<(&mut Transform, &Key), Without<PressedKey>>
){
    for (mut t, k) in &mut query {
        t.translation.y = k.y_reset;
    }
}


fn handle_midi_input(
    mut commands: Commands,
    mut midi_events: EventReader<MidiData>,
    query: Query<(Entity, &Key)>,
) {
    for data in midi_events.iter() {
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

fn connect_to_first_port(input: Res<MidiInput>) {
    if input.is_changed() {
        if let Some((_, port)) = input.ports().get(0) {
            input.connect(port.clone());
        }
    }
}
