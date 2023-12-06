use std::f32::consts::PI;

use bevy::{pbr::ExtendedMaterial, prelude::*};
use rand::Rng;

use crate::{physics::Velocity, player::Bark, sprite_material::SpriteExtension};

const SHEEP_PATH: &str = "test/sheep.png";

#[derive(Default, PartialEq, Eq, Debug, Clone, Component, Reflect)]
pub struct Sheep;

#[derive(Default, PartialEq, Debug, Clone, Component, Reflect)]
pub struct IsScared(bool, f32);

#[derive(Default, PartialEq, Eq, Debug, Clone, Component, Reflect)]
#[reflect(Component, Default)]
pub enum Decision {
    #[default]
    Idle,
    Feed,
    MoveToSafeZone,
    MoveOutSafeZone,
}

#[derive(PartialEq, Debug, Clone, Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct StateChance {
    next_state: Vec<(f32, Decision)>,
}

impl Default for StateChance {
    fn default() -> Self {
        Self {
            next_state: vec![
                (0.25, Decision::Idle),
                (0.5, Decision::Feed),
                (0.75, Decision::MoveToSafeZone),
                (1.0, Decision::MoveOutSafeZone),
            ],
        }
    }
}

pub fn scared_sheeps(
    mut event_reader: EventReader<Bark>,
    mut sheeps: Query<(&Sheep, &Transform, &mut Velocity, &mut IsScared)>,
) {
    if let Some(bark) = event_reader.read().next() {
        let bark_origin = bark.position;
        for mut sheep in &mut sheeps {
            if sheep.1.translation.distance(bark_origin) <= bark.radius {
                sheep.3 .0 = true;
                sheep.2 .0 = sheep.1.translation - bark_origin;
                sheep.2 .0.y = 0.0; //sheep must not fly and be in fixed height
            }
        }
    }
    event_reader.clear();
}

pub fn sheep_state(
    _next_state: Res<StateChance>,
    mut sheeps: Query<(&Sheep, &mut Velocity, &mut Decision, &IsScared)>,
) {
    for mut sheep in &mut sheeps.iter_mut().filter(|query| !query.3 .0) {
        *sheep.2 = Decision::Feed;
        *sheep.1 = Velocity(Vec3 {
            x: 3.,
            y: 3.,
            z: 3.,
        });
    }
}

pub fn update_scared_sheeps(
    time: Res<Time>,
    mut sheeps: Query<(&Sheep, &mut Velocity, &mut Decision, &mut IsScared)>,
) {
    for mut sheep in sheeps.iter_mut().filter(|q| q.3 .0) {
        if sheep.3 .1 > 2. {
            *sheep.2 = Decision::Idle;
            *sheep.1 = Velocity::default();
            *sheep.3 = IsScared::default();
        } else {
            sheep.3 .1 += time.delta_seconds();
        }
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut sprite_material: ResMut<Assets<ExtendedMaterial<StandardMaterial, SpriteExtension>>>,
) {
    let square = meshes.add(
        shape::Plane {
            size: 1.0,
            ..default()
        }
        .into(),
    );
    let sheep_texture: Handle<Image> = asset_server.load(SHEEP_PATH);

    let sheep_material = materials.add(StandardMaterial {
        base_color_texture: Some(sheep_texture.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let sheep_sprite_material = sprite_material.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color_texture: Some(sheep_texture.clone()),
            alpha_mode: AlphaMode::Opaque,
            ..default()
        },
        extension: SpriteExtension {
            base_teture: Some(sheep_texture.clone()),
        },
    });

    //spawn sheeps
    let r = 50.0;
    let mut rng = rand::thread_rng();
    let sheep_count = 10;

    for _ in 0..sheep_count {
        let x = rng.gen_range(-r..r);
        let y = 0.0;
        let z = rng.gen_range(-r..r);

        let pos = Vec3::new(x, y, z);
        if pos.length() > r {
            continue;
        }

        commands.spawn((
            MaterialMeshBundle {
                mesh: square.clone(),
                material: sheep_sprite_material.clone(),
                transform: Transform::from_xyz(pos.x, pos.y + 3.0, pos.z)
                    .with_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        PI / 2.0 - PI / 4.0,
                        0.0,
                        0.0,
                    ))
                    .with_scale(Vec3::splat(10.0)),
                ..default()
            },
            Sheep,
            Decision::Idle,
            Velocity::default(),
            IsScared(false, 0.),
        ));
    }
}
