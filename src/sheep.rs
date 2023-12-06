use std::f32::consts::PI;

use bevy::{
    gltf::GltfMesh,
    pbr::ExtendedMaterial,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use rand::Rng;

use crate::{
    get_sprite_rotation,
    physics::Velocity,
    player::Bark,
    sprite_material::{create_plane_mesh, SpriteExtension},
    test_level::TEST_LEVEL_SIZE,
};

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
    // mut _sprite_material: ResMut<Assets<ExtendedMaterial<StandardMaterial, SpriteExtension>>>,
) {
    let square = meshes.add(create_plane_mesh());
    let sheep_texture: Handle<Image> = asset_server.load(SHEEP_PATH);

    let sheep_material = materials.add(StandardMaterial {
        base_color_texture: Some(sheep_texture.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    //spawn sheeps
    let r = TEST_LEVEL_SIZE / 2.0;
    let mut rng = rand::thread_rng();
    let sheep_count = 100;

    for _ in 0..sheep_count {
        let x = rng.gen_range(-r..r);
        let y = 0.0;
        let z = rng.gen_range(-r..r);

        let pos = Vec3::new(x, y, z);
        if pos.length() > r {
            continue;
        }

        commands.spawn((
            PbrBundle {
                mesh: square.clone(),
                material: sheep_material.clone(),
                transform: Transform::from_xyz(pos.x, pos.y, pos.z)
                    .with_rotation(get_sprite_rotation())
                    .with_scale(Vec3::splat(1.0)),
                ..default()
            },
            Sheep,
            Decision::Idle,
            Velocity::default(),
            IsScared(false, 0.),
        ));
    }
}
