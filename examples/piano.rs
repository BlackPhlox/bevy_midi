use bevy::{
    log::{Level, LogPlugin},
    pbr::AmbientLight,
    prelude::*,
    window::WindowResolution,
};
use bevy_midi::{
    input::*,
    output::{MidiOutput, MidiOutputPlugin, MidiOutputSettings},
    KEY_RANGE,
};
use bevy_mod_picking::{
    DefaultPickingPlugins, HoverEvent, PickableBundle, PickingCameraBundle, PickingEvent,
    SelectionEvent,
};

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::WARN,
            filter: "bevy_midi=debug".to_string(),
        }))
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(MidiInputPlugin)
        .init_resource::<MidiInputSettings>()
        .add_plugin(MidiOutputPlugin)
        .init_resource::<MidiOutputSettings>()
        .add_startup_system(setup)
        .add_system(handle_midi_input)
        .add_system(connect_to_first_input_port)
        .add_system(connect_to_first_output_port)
        .add_system(print_events.in_base_set(CoreSet::PostUpdate))
        .add_system(display_press)
        .add_system(display_release)
        .run();
}

#[derive(Component, Debug)]
struct Key {
    key_val: String,
    y_reset: f32,
}

pub fn print_events(
    mut events: EventReader<PickingEvent>,
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    for event in events.iter() {
        let entity = match event {
            PickingEvent::Selection(SelectionEvent::JustSelected(e)) => e,
            PickingEvent::Selection(SelectionEvent::JustDeselected(e)) => e,
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => e,
            PickingEvent::Hover(HoverEvent::JustLeft(e)) => e,
            PickingEvent::Clicked(e) => e,
        };

        if mouse_button_input.pressed(MouseButton::Left) {
            commands.entity(*entity).insert(PressedKey);
        } else {
            commands.entity(*entity).remove::<PressedKey>();
        }
    }
}

#[derive(Component)]
struct PressedKey;

#[rustfmt::skip]
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mid = -6.3;

    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 6.0, mid),
        ..Default::default()
    });

    //Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(8., 5., mid).looking_at(Vec3::new(0., 0., mid), Vec3::Y),
            ..Default::default()
        },
        PickingCameraBundle::default(),
    ));

    let pos: Vec3 = Vec3::new(0., 0., 0.);

    let mut black_key: Handle<Mesh> = asset_server.load("models/black_key.gltf#Mesh0/Primitive0");
    let mut white_key_0: Handle<Mesh> = asset_server.load("models/white_key_0.gltf#Mesh0/Primitive0");
    let mut white_key_1: Handle<Mesh> = asset_server.load("models/white_key_1.gltf#Mesh0/Primitive0");
    let mut white_key_2: Handle<Mesh> = asset_server.load("models/white_key_2.gltf#Mesh0/Primitive0");
    let b_mat = materials.add(Color::rgb(0.1, 0.1, 0.1).into());
    let w_mat = materials.add(Color::rgb(1.0, 1.0, 1.0).into());

    //Create keyboard layout
    let pos_black = pos + Vec3::new(0., 0.06, 0.);
    
    for i in 0..8 {
        spawn_note(&mut commands, &w_mat, 0.00, pos, &mut white_key_0, i, "C");
        spawn_note(&mut commands, &b_mat, 0.15, pos_black, &mut black_key, i, "C#");
        spawn_note(&mut commands, &w_mat, 0.27, pos, &mut white_key_1, i, "D");
        spawn_note(&mut commands, &b_mat, 0.39, pos_black, &mut black_key, i, "D#");
        spawn_note(&mut commands, &w_mat, 0.54, pos, &mut white_key_2, i, "E");
        spawn_note(&mut commands, &w_mat, 0.69, pos, &mut white_key_0, i, "F");
        spawn_note(&mut commands, &b_mat, 0.85, pos_black, &mut black_key, i, "F#");
        spawn_note(&mut commands, &w_mat, 0.96, pos, &mut white_key_1, i, "G");
        spawn_note(&mut commands, &b_mat, 1.08, pos_black, &mut black_key, i, "G#");
        spawn_note(&mut commands, &w_mat, 1.19, pos, &mut white_key_1, i, "A");
        spawn_note(&mut commands, &b_mat, 1.31, pos_black, &mut black_key, i, "A#");
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
    commands.spawn((
        PbrBundle {
            mesh: asset.clone(),
            material: mat.clone(),
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - offset_z - (1.61 * oct as f32)),
                scale: Vec3::new(10., 10., 10.),
                ..Default::default()
            },
            ..Default::default()
        },
        Key {
            key_val: format!("{}{}", key, oct),
            y_reset: pos.y,
        },
        PickableBundle::default(),
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
