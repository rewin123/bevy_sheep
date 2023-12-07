use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    get_sprite_rotation,
    physics::{Velocity, WalkController},
    player::{Bark, DOG_SPEED},
    safe_area::SafeArea,
    sprite_material::create_plane_mesh,
    test_level::LevelSize,
};

const SHEEP_PATH: &str = "test/sheep.png";

const SHEEP_SPEED: f32 = DOG_SPEED * 0.5;
const SHEEP_ACCELERATION: f32 = SHEEP_SPEED * 3.0;

const RANDOM_WALK_RANGE: f32 = 5.0;
const RANDOM_WALK_ACCEPT_RADIUS: f32 = 0.5;
const RANDOM_WALK_SPEED_MULTIPLIER: f32 = 0.2;

const IDLE_FEEDING_TIME: f32 = 1.0;
const IDLE_FEEDING_TIME_RANGE: f32 = 0.5;

pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StateChance>();

        app.add_systems(Update, (scared_sheeps, update_scared_sheeps));

        app.add_systems(Update, (sheep_state,));

        //random walk
        app.add_event::<InitRandomWalk>()
            .add_systems(Update, (init_random_walk, random_walk_system));

        //idle feeding
        app.add_systems(Update, idle_feeding_system);

        // Move to safearea
        app.add_event::<SafeAreaWalk>()
            .add_systems(Update, (init_safeareawalk_walk, walk_to_safe_zone_system));

        app.register_type::<StateChance>()
            .register_type::<Decision>()
            .register_type::<IsScared>();
    }
}

#[derive(Default, PartialEq, Debug, Clone, Component, Reflect)]
pub struct Sheep {
    time: f32,
}

#[derive(Default, PartialEq, Debug, Clone, Component, Reflect)]
#[reflect(Component, Default)]
pub struct IsScared {
    time: f32,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Component, Reflect, Copy)]
#[reflect(Component, Default)]
pub enum Decision {
    #[default]
    Idle,
    Feed,
    RandomWalk,
    MoveToSafeArea,
    MoveOutSafeArea,
    Scared, //Mark that sheep will not be able to another decision
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
                (0.35, Decision::Idle),
                (0.5, Decision::Feed), //zero values for unimplemented things
                (0.1, Decision::RandomWalk),
                (0.7, Decision::MoveToSafeArea),
                (1.0, Decision::MoveOutSafeArea),
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
        self.next_state.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
}

pub fn scared_sheeps(
    mut commands: Commands,
    mut event_reader: EventReader<Bark>,
    mut sheeps: Query<(Entity, &Transform, &mut WalkController, &mut Decision), With<Sheep>>,
) {
    if let Some(bark) = event_reader.read().next() {
        let bark_origin = bark.position;
        for mut sheep in &mut sheeps {
            if sheep.1.translation.distance(bark_origin) <= bark.radius {
                let scare = IsScared::default();
                sheep.2.target_velocity =
                    (sheep.1.translation - bark_origin).normalize_or_zero() * SHEEP_SPEED;
                sheep.2.target_velocity.y = 0.0; //sheep must not fly and be in fixed height
                commands
                    .entity(sheep.0)
                    .insert(scare)
                    .remove::<RandomWalk>()
                    .remove::<IdleFeeding>()
                    .remove::<SafeAreaTarget>();
                *sheep.3 = Decision::Scared;
            }
        }
    }
    event_reader.clear();
}

#[derive(Event)]
pub struct InitRandomWalk {
    pub e: Entity,
}

#[derive(Event)]
pub struct SafeAreaWalk {
    pub e: Entity,
}

#[derive(Component)]
pub struct RandomWalk {
    pub target: Vec3,
}

#[derive(Component)]
pub struct SafeAreaTarget {
    pub target: Vec3,
}

fn init_random_walk(
    mut commands: Commands,
    mut event_reader: EventReader<InitRandomWalk>,
    poses: Query<&Transform, With<Sheep>>,
) {
    let mut rand = rand::thread_rng();
    for ev in event_reader.read() {
        if let Ok(t) = poses.get_component::<Transform>(ev.e) {
            // info!("init random walk for {:?}", ev.e);
            let r = rand.gen_range(0.0..RANDOM_WALK_RANGE);
            let angle = rand.gen_range(0.0..PI * 2.0);

            commands.entity(ev.e).insert(RandomWalk {
                target: t.translation + Vec3::new(angle.cos() * r, 0.0, angle.sin() * r),
            });
        }
    }
    event_reader.clear();
}

fn init_safeareawalk_walk(
    mut commands: Commands,
    mut event_reader: EventReader<SafeAreaWalk>,
    poses: Query<&Transform, With<Sheep>>,
    safeareas: Query<&SafeArea>,
    level_size: Res<LevelSize>,
) {
    let Ok(safearea) = safeareas.get_single() else {
        return;
    };

    for ev in event_reader.read() {
        if poses.get_component::<Transform>(ev.e).is_ok() {
            commands.entity(ev.e).insert(SafeAreaTarget {
                target: safearea.get_random_point_inside(level_size.0),
            });
        }
    }
    event_reader.clear();
}

fn random_walk_system(
    mut commands: Commands,
    mut random_walks: Query<(
        Entity,
        &mut Transform,
        &mut WalkController,
        &mut Decision,
        &RandomWalk,
    )>,
) {
    for (e, t, mut v, mut dec, rw) in &mut random_walks.iter_mut() {
        if t.translation.distance(rw.target) < RANDOM_WALK_ACCEPT_RADIUS {
            v.target_velocity = Vec3::ZERO;
            commands.entity(e).remove::<RandomWalk>();
            *dec = Decision::Idle;
        } else {
            v.target_velocity = (rw.target - t.translation).normalize()
                * SHEEP_SPEED
                * RANDOM_WALK_SPEED_MULTIPLIER;
        }
    }
}

fn walk_to_safe_zone_system(
    mut commands: Commands,
    mut safe_walks: Query<(
        Entity,
        &mut Transform,
        &mut WalkController,
        &mut Decision,
        &SafeAreaTarget,
    )>,
) {
    for (e, t, mut v, mut dec, rw) in &mut safe_walks.iter_mut() {
        if t.translation.distance(rw.target) < RANDOM_WALK_ACCEPT_RADIUS {
            v.target_velocity = Vec3::ZERO;
            commands.entity(e).remove::<SafeAreaTarget>();
            *dec = Decision::Idle;
        } else {
            v.target_velocity = (rw.target - t.translation).normalize()
                * SHEEP_SPEED
                * RANDOM_WALK_SPEED_MULTIPLIER;
        }
    }
}

pub fn sheep_state(
    mut commands: Commands,
    state_matrix: Res<StateChance>,
    time: Res<Time>,
    mut sheeps: Query<(Entity, &mut Decision, &mut Sheep), Without<IsScared>>,
    mut init_random_walk: EventWriter<InitRandomWalk>,
    mut init_safe_walk: EventWriter<SafeAreaWalk>,
) {
    let mut rand = rand::thread_rng();
    for (e, mut dec, mut sheep) in &mut sheeps.iter_mut() {
        sheep.time += time.delta_seconds();
        if *dec == Decision::Idle && sheep.time > 1.5 * IDLE_FEEDING_TIME {
            sheep.time = 0.;
            let p = rand.gen_range(0.0..1.0);
            let next_dec = state_matrix
                .next_state
                .iter()
                .find(|state| state.0 > p)
                .map(|s| s.1)
                .unwrap_or_default();

            *dec = next_dec;

            match next_dec {
                Decision::Idle => {
                    // info!("new idle for {:?}", e);
                }
                Decision::Feed => {
                    commands.entity(e).insert(IdleFeeding {
                        time: rand.gen_range(0.0..IDLE_FEEDING_TIME_RANGE) + IDLE_FEEDING_TIME,
                    });
                }
                Decision::RandomWalk => {
                    // info!("new random walk for {:?}", e);
                    init_random_walk.send(InitRandomWalk { e });
                }
                Decision::MoveToSafeArea => {
                    init_safe_walk.send(SafeAreaWalk { e });
                }
                Decision::MoveOutSafeArea => {
                    // For now this seems ok
                    init_random_walk.send(InitRandomWalk { e });
                }
                Decision::Scared => {
                    *dec = Decision::Idle;
                }
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
    level_size: Res<LevelSize>,
) {
    let square = meshes.add(create_plane_mesh());
    let sheep_texture: Handle<Image> = asset_server.load(SHEEP_PATH);

    let sheep_material = materials.add(StandardMaterial {
        base_color_texture: Some(sheep_texture.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    //spawn sheeps
    let r = level_size.0 / 1.5;
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
            Sheep::default(),
            Decision::Idle,
            Velocity::default(),
            WalkController {
                target_velocity: Vec3::new(0.0, 0.0, 0.0),
                acceleration: SHEEP_ACCELERATION,
                max_speed: SHEEP_SPEED,
            },
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
