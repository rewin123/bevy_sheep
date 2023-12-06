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
    physics::{Velocity, WalkController},
    player::{Bark, DOG_SPEED, DOG_ACCELERATION},
    sprite_material::{create_plane_mesh, SpriteExtension},
    test_level::TEST_LEVEL_SIZE,
};

const SHEEP_PATH: &str = "test/sheep.png";

const SHEEP_SPEED : f32 = DOG_SPEED * 0.3;
const SHEEP_ACCELERATION : f32 = SHEEP_SPEED * 3.0;

const RANDOM_WALK_RANGE : f32 = 5.0;
const RANDOM_WALK_ACCEPT_RADIUS : f32 = 0.5;

const IDLE_FEEDING_TIME : f32 = 1.0;
const IDLE_FEEDING_TIME_RANGE : f32 = 0.5;

pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {

        app.init_resource::<StateChance>();

        app.add_systems(
            Update,
            (
                scared_sheeps,
                update_scared_sheeps,
            ),
        );

        app.add_systems(Update, (
            sheep_state,
        ));

        //random walk
        app.add_event::<InitRandomWalk>()
            .add_systems(Update, (
                init_random_walk,
                random_walk_system,
            ));

        //idle feeding
        app.add_systems(Update, idle_feeding_system);
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Component, Reflect)]
pub struct Sheep;

#[derive(Default, PartialEq, Debug, Clone, Component, Reflect)]
pub struct IsScared{
    time : f32
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Component, Reflect, Copy)]
#[reflect(Component, Default)]
pub enum Decision {
    #[default]
    Idle,
    Feed,
    IdleFeeding,
    RandomWalk,
    MoveToSafeZone,
    MoveOutSafeZone,
    Scared //Mark that ship will not be able to another decision
}

#[derive(PartialEq, Debug, Clone, Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct StateChance {
    pub next_state: Vec<(f32, Decision)>,
}

impl Default for StateChance {
    fn default() -> Self {
        let mut res = Self {
            //set weights
            next_state: vec![
                (0.0, Decision::Idle),
                (0.0, Decision::Feed), //zero values for unimplemented things
                (1.0, Decision::IdleFeeding),
                (1.0, Decision::RandomWalk),
                (0.0, Decision::MoveToSafeZone),
                (0.0, Decision::MoveOutSafeZone),
            ],
        };
        res.normalize();
        res
    }
}

impl StateChance {
    fn normalize(&mut self) {
        let mut sum = 0.0;
        for (w, _) in &self.next_state {
            sum += *w;
        }

        for (w, _) in &mut self.next_state {
            *w /= sum;
        }
    }
}

pub fn scared_sheeps(
    mut commands: Commands,
    mut event_reader: EventReader<Bark>,
    mut sheeps: Query<(Entity, &Transform, &mut WalkController), With<Sheep>>,
) {
    if let Some(bark) = event_reader.read().next() {
        let bark_origin = bark.position;
        for mut sheep in &mut sheeps {
            if sheep.1.translation.distance(bark_origin) <= bark.radius {
                let scare = IsScared::default();
                sheep.2 .target_velocity = (sheep.1.translation - bark_origin).normalize_or_zero() * SHEEP_SPEED;
                sheep.2 .target_velocity.y = 0.0; //sheep must not fly and be in fixed height
                commands.entity(sheep.0).insert(scare);
            }
        }
    }
    event_reader.clear();
}

#[derive(Event)]
pub struct InitRandomWalk {
    pub e : Entity,
}

#[derive(Component)]
pub struct RandomWalk {
    pub target : Vec3,
}

fn init_random_walk(
    mut commands: Commands,
    mut event_reader: EventReader<InitRandomWalk>,
    poses : Query<&Transform, With<Sheep>>
) {

    let mut rand = rand::thread_rng();
    for ev in event_reader.read() {
        if let Ok(t) = poses.get_component::<Transform>(ev.e) {
            info!("init random walk for {:?}", ev.e);
            let r = rand.gen_range(0.0..RANDOM_WALK_RANGE);
            let angle = rand.gen_range(0.0..PI*2.0);
            
            commands.entity(ev.e).insert(RandomWalk {
                target: t.translation + Vec3::new(angle.cos() * r, 0.0, angle.sin() * r),
            });
        } else {
            warn!("init random walk for {:?} failed", ev.e);
        }
    }
    event_reader.clear();
}

fn random_walk_system(
    mut commands: Commands,
    mut random_walks: Query<(Entity, &mut Transform, &mut WalkController, &mut Decision, &RandomWalk)>,
) {
    for (e, mut t, mut v, mut dec, rw) in &mut random_walks.iter_mut() {
        if t.translation.distance(rw.target) < RANDOM_WALK_ACCEPT_RADIUS {
            v.target_velocity = Vec3::ZERO;
            commands.entity(e).remove::<RandomWalk>();
            *dec = Decision::Idle;
        } else {
            v.target_velocity = (rw.target - t.translation).normalize() * SHEEP_SPEED;
        }
    }
}

pub fn sheep_state(
    mut commands: Commands,
    state_matrix: Res<StateChance>,
    mut sheeps: Query<(Entity, &mut Decision), With<Sheep>>,
    mut init_random_walk: EventWriter<InitRandomWalk>,
) {
    let mut rand = rand::thread_rng();
    for (e, mut dec) in &mut sheeps.iter_mut() {
        if *dec == Decision::Idle {
            let p = rand.gen_range(0.0..1.0);
            let mut sum = 0.0;
            let mut next_dec = Decision::Idle;
            for (w, d) in &state_matrix.next_state {
                sum += *w;
                if p < sum {
                    next_dec = *d;
                    break;
                }
            }

            *dec = next_dec;

            match next_dec {
                Decision::Idle => {
                    info!("new idle for {:?}", e);
                },
                Decision::Feed => {
                    info!("new feed for {:?}", e);
                },
                Decision::RandomWalk => {
                    info!("new random walk for {:?}", e);
                    init_random_walk.send(InitRandomWalk { e });
                },
                Decision::MoveToSafeZone => {
                    info!("new move to safe zone for {:?}", e);
                },
                Decision::MoveOutSafeZone => {
                    info!("new move out safe zone for {:?}", e);
                },
                Decision::IdleFeeding => {
                    info!("new idle feeding for {:?}", e);
                    commands.entity(e).insert(IdleFeeding {
                        time: rand.gen_range(0.0..IDLE_FEEDING_TIME_RANGE) + IDLE_FEEDING_TIME,
                    });
                },
                Decision::Scared => {

                },
            }
        }
    }
}

pub fn update_scared_sheeps(
    mut commands: Commands,
    time: Res<Time>,
    mut sheeps: Query<(Entity, &mut WalkController, &mut Decision, &mut IsScared), With<Sheep>>,
) {
    for mut sheep in sheeps.iter_mut() {
        if sheep.3.time > 2. {
            *sheep.2 = Decision::Idle;
            sheep.1.target_velocity = Vec3::ZERO;
            commands.entity(sheep.0).remove::<IsScared>();
        } else {
            sheep.3.time += time.delta_seconds();
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
    let r = TEST_LEVEL_SIZE / 1.5;
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
                    .with_scale(Vec3::new(13.0 / 10.0, 1.0, 1.0)),
                ..default()
            },
            Sheep,
            Decision::Idle,
            Velocity::default(),
            WalkController {
                target_velocity: Vec3::new(0.0, 0.0, 0.0),
                acceleration: SHEEP_ACCELERATION,
                max_speed: SHEEP_SPEED,
            }
        ));
    }
}

#[derive(Component)]
pub struct IdleFeeding {
    pub time: f32,
}

fn idle_feeding_system(
    mut commands: Commands,
    mut sheeps: Query<(Entity, &mut Decision, &mut IdleFeeding)>,
    time: Res<Time>,
) {
    for (e, mut dec, mut idle) in sheeps.iter_mut() {
        idle.time -= time.delta_seconds();
        if idle.time < 0.0 {
            *dec = Decision::Idle;
            commands.entity(e).remove::<IdleFeeding>();
        }
    }
}