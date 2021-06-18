  
use bevy::{pbr::AmbientLight, prelude::*};
use bevy_config_cam::ConfigCam;
use bevy_midi::Midi;

#[derive(Debug)]
struct Key(String);

fn main() {
    App::build()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(ConfigCam)
        .add_plugin(Midi)
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_system(setup_octave.system())
        .add_system(rotator_system.system())
        .add_system(key_system.system())
        //.add_system(key_bow_system.system())
        .run();
}

fn setup_octave(mut commands: Commands, asset_server: Res<AssetServer>) {
    let pos: Vec3 = Vec3::new(0.,0.,0.);

    let black_key   : Handle<Scene> = asset_server.load("models/black_key.gltf#Scene0");
    let white_key_0 : Handle<Scene> = asset_server.load("models/white_key_0.gltf#Scene0");
    let white_key_1 : Handle<Scene> = asset_server.load("models/white_key_1.gltf#Scene0");
    let white_key_2 : Handle<Scene> = asset_server.load("models/white_key_2.gltf#Scene0");

    
    for i in 0..7 {
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("C{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(white_key_0.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y + 0.06, pos.z - 0.15 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("C#{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(black_key.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - 0.27 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("D{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(white_key_1.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y + 0.06, pos.z - 0.39 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("D#{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(black_key.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - 0.54 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("E{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(white_key_2.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - 0.69 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("F{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(white_key_0.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y + 0.06, pos.z - 0.85 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("F#{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(black_key.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - 0.96 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("G{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(white_key_1.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y + 0.06, pos.z - 1.08 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("G#{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(black_key.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - 1.19 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("A{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(white_key_1.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y + 0.06, pos.z - 1.31 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("A#{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(black_key.clone());
        });
    
        commands.spawn_bundle((
            Transform {
                translation: Vec3::new(pos.x, pos.y, pos.z - 1.46 - (1.61 * i as f32)),
                scale: Vec3::new(10.,10.,10.),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .insert(Key(format!("B{}",i)))
        .with_children(|cell| {
            cell.spawn_scene(white_key_2.clone());
        });
    }

    
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..Default::default()
    });
    commands
        .spawn_bundle(LightBundle {
            transform: Transform::from_xyz(3.0, 5.0, 3.0),
            ..Default::default()
        })
        .insert(Rotates);
}

/// this component indicates what entities should rotate
struct Rotates;

fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    for mut transform in query.iter_mut() {
        *transform = Transform::from_rotation(Quat::from_rotation_y(
            (4.0 * std::f32::consts::PI / 20.0) * time.delta_seconds(),
        )) * *transform;
    }
}

fn key_system(time: Res<Time>, keys: Res<Input<KeyCode>>, mut query: Query<(&Key, &mut Transform)>) {
    if keys.just_released(KeyCode::F){
        for (key, mut transform) in query.iter_mut() {
            if key.0 == "C4" {
                transform.translation = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
            }
        }
    }
    for key in keys.get_pressed() {
        if key == &KeyCode::F{
            for (key, mut transform) in query.iter_mut() {
                if key.0 == "C4" {
                    if transform.translation.y > -0.1 {
                        transform.translation = Vec3::new(transform.translation.x ,transform.translation.y - 0.1, transform.translation.z);
                    }
                }
            }
        }
    }
}

fn key_bow_system(time: Res<Time>, keys: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Key>>) {
    let mut i = 0;

    for mut transform in query.iter_mut() {
        *transform = Transform::from_translation(
            Vec3::new(0.,(time.seconds_since_startup() as f32 * 3. + i as f32/8.).sin() * -0.02, 0.)
        ) * *transform;
        i += 1;
    }
}