use std::{f32::consts::PI, time::Duration, fmt::format};

use bevy::{prelude::*, utils::HashSet};
use rand::{rngs::ThreadRng, Rng};

use crate::{
    get_sprite_rotation,
    global_task::sheep_escape::ShawshankRedemption,
    physics::{Velocity, WalkController},
    player::{Bark, Dog, DOG_SPEED},
    safe_area::SafeArea,
    sprite_material::create_plane_mesh,
    test_level::LevelSize,
    GameSet, GameStuff, auto_anim::{AnimRange, AnimSet, AutoAnimPlugin, AutoAnim},
};

use bevy_spatial::{
    kdtree::KDTree3, AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode,
};

const SHEEP_PATH: &str = "test/sheep.png";
const SHEEP_FOLDER_PATH: &str = "sheep/";

pub const SHEEP_SPEED: f32 = DOG_SPEED * 0.5;
const SHEEP_ACCELERATION: f32 = SHEEP_SPEED * 3.0;

const RANDOM_WALK_RANGE: f32 = 2.0;
const RANDOM_WALK_ACCEPT_RADIUS: f32 = 0.5;
pub const RANDOM_WALK_SPEED_MULTIPLIER: f32 = 0.3;

//IDLE FEEDING must be large enough so that player can see sheep and react for escapes
const IDLE_FEEDING_TIME: f32 = 2.5;
const IDLE_FEEDING_TIME_RANGE: f32 = 1.0;

const MOVE_IN_DIST: f32 = 11.0;
const MOVE_OUT_DIST: f32 = 10.0;

const SCARE_MAX_DIST: f32 = 10.0;

pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StateChance>();

        app.add_systems(
            Update,
            (scared_sheeps, update_scared_sheeps).in_set(GameSet::Playing),
        );

        app.add_systems(Update, (sheep_state,).in_set(GameSet::Playing));

        //random walk
        app.add_event::<InitRandomWalk>()
            .add_systems(Update, (init_random_walk,).in_set(GameSet::Playing));

        app.add_systems(Update, (goto_system,).in_set(GameSet::Playing));

        //idle feeding
        app.add_systems(Update, idle_feeding_system.in_set(GameSet::Playing));

        // Move to safearea
        app.add_event::<SafeAreaWalk>()
            .add_systems(Update, (init_safeareawalk_walk,).in_set(GameSet::Playing));

        app.add_event::<EscapeWalk>()
            .add_systems(Update, init_escape.in_set(GameSet::Playing));

        app.register_type::<StateChance>()
            .register_type::<Decision>()
            .register_type::<IsScared>();

        app.add_plugins(
            AutomaticUpdate::<Sheep>::new()
                .with_frequency(Duration::from_millis(250))
                .with_transform(TransformMode::Transform)
                .with_spatial_ds(SpatialStructure::KDTree3),
        )
        .add_systems(Update, collect_field)
        .add_plugins(AutoAnimPlugin::<SheepAnim>::default())
        
        .add_systems(Update, set_anim_state.in_set(GameSet::Playing))
        
        .add_systems(Update, update_nearest);
    }
}

#[derive(Default)]
pub enum SheepAnim {
    #[default]
    Idle,
    Walk,
    Feed
}

impl AnimSet for SheepAnim {
    fn get_folder_path() -> String {
        format!("sheep")
    }

    fn get_index_range(&self) -> AnimRange {
        match self {
            SheepAnim::Idle => AnimRange::new(28, 39),
            SheepAnim::Walk => AnimRange::new(23, 28),
            SheepAnim::Feed => AnimRange::new(13, 16),
        }
    }

    fn get_tile_count() -> usize {
        40
    }
}

#[derive(Resource)]
pub struct StartSheepCount(pub f32);

#[derive(Default, PartialEq, Debug, Clone, Component, Reflect)]
pub struct Sheep;

#[derive(Default, PartialEq, Debug, Clone, Component, Reflect)]
#[reflect(Component, Default)]
pub struct IsScared {
    time: f32,
    last_vel: Vec3,
}

#[derive(Component, Default)]
pub struct SheepTargetVel(pub Vec3);

#[derive(Default, PartialEq, Eq, Debug, Clone, Component, Reflect, Copy)]
#[reflect(Component, Default)]
pub enum Decision {
    #[default]
    Idle, //set sheep state to waiting next decision. Not just be idle. For stending we will use Feed or deleted IdleFeeding. We need to have transition states to move to next decision. Addition set of moves will be seen as sheep has plan to move and will be nicely waiting
    Feed,
    RandomWalk,
    MoveToSafeArea,
    Escape,
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
            //its weights are relative
            //must be all between 0 and 1
            //normalization will be done automatically

            //For testing i made all weights 1 so all next states are equally likely to be chosen
            next_state: vec![
                (1.0, Decision::Feed), //zero values for unimplemented things
                (0.5, Decision::RandomWalk),
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

    //I separated next decision selection to function
    fn select_next(&self, rng: &mut ThreadRng) -> Decision {
        let mut sum = 0.0; //This decisicion selection is based on weights, not prop graph. Just for testing and more stable behavior. Dont change please
        let p = rng.gen_range(0.0..1.0);
        for (w, d) in &self.next_state {
            sum += *w;
            if p < sum {
                return *d;
            }
        }
        Decision::Idle
    }
}

pub fn scared_sheeps(
    mut commands: Commands,
    mut event_reader: EventReader<Bark>,
    mut sheeps: Query<(Entity, &Transform, &mut SheepTargetVel, &mut Decision), (With<Sheep>, Without<IsScared>)>,
) {
    if let Some(bark) = event_reader.read().next() {
        let bark_origin = bark.position;
        for mut sheep in &mut sheeps {
            if sheep.1.translation.distance(bark_origin) <= bark.radius {
                let scare = IsScared::default();
                commands
                    .entity(sheep.0)
                    .insert(scare)
                    .remove::<IdleFeeding>()
                    .remove::<GoTo>()
                    .remove::<ShawshankRedemption>();
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
pub struct GoTo {
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

            commands.entity(ev.e).insert(GoTo {
                target: t.translation + Vec3::new(angle.cos() * r, 0.0, angle.sin() * r),
            });
        }
    }
    event_reader.clear();
}

fn goto_system(
    mut commands: Commands,
    mut goto_query: Query<(
        Entity,
        &mut Transform,
        &mut SheepTargetVel,
        &mut Decision,
        &GoTo,
    )>,
) {
    for (e, t, mut v, mut dec, rw) in &mut goto_query.iter_mut() {
        if t.translation.distance(rw.target) < RANDOM_WALK_ACCEPT_RADIUS {
            v.0 = Vec3::ZERO;
            commands.entity(e).remove::<GoTo>();
            *dec = Decision::Idle;
        } else {
            v.0 = (rw.target - t.translation).normalize()
                * SHEEP_SPEED
                * RANDOM_WALK_SPEED_MULTIPLIER;
        }
    }
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
        if let Ok(t) = poses.get_component::<Transform>(ev.e) {
            let inside_point = safearea.get_random_point_inside(level_size.0 / 3.0);
            let dir = (inside_point - t.translation).normalize_or_zero();
            commands.entity(ev.e).insert(GoTo {
                target: t.translation + dir * MOVE_IN_DIST, // move to near center, so move will be safe, opposite to RandomWalk or Move out safe zone
            });
        }
    }
    event_reader.clear();
}

pub fn sheep_state(
    mut commands: Commands,
    state_matrix: Res<StateChance>,
    mut sheeps: Query<(Entity, &mut Decision, &Sheep), Without<IsScared>>,
    mut init_random_walk: EventWriter<InitRandomWalk>,
    mut init_safe_walk: EventWriter<SafeAreaWalk>,
    mut init_escape_walk: EventWriter<EscapeWalk>,
) {
    let mut rand = rand::thread_rng();
    for (e, mut dec, _sheep) in &mut sheeps.iter_mut() {
        if *dec == Decision::Idle {
            let next_dec = state_matrix.select_next(&mut rand);

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
                Decision::Escape => {
                    // For now this seems ok
                    init_escape_walk.send(EscapeWalk { e });
                }
                Decision::Scared => {
                    *dec = Decision::Idle;
                }
            }
        }
    }
}

#[derive(Event)]
pub struct EscapeWalk {
    pub e: Entity,
}

pub fn init_escape(
    mut commands: Commands,
    mut event_reader: EventReader<EscapeWalk>,
    poses: Query<&Transform, With<Sheep>>,
    safe_zones: Query<&SafeArea>,
) {
    for ev in event_reader.read() {
        if let Ok(t) = poses.get_component::<Transform>(ev.e) {
            //find nearest safe zone
            let mut nearest = None;
            let mut nearest_dist = f32::MAX;

            for sa in safe_zones.iter() {
                let dist = t.translation.distance(sa.get_center());
                if dist < nearest_dist {
                    nearest = Some(sa);
                    nearest_dist = dist;
                }
            }

            if let Some(sa) = nearest {
                let dir = (t.translation - sa.get_center()).normalize_or_zero();
                info!("escape {:?}", t.translation);
                commands.entity(ev.e).insert(GoTo {
                    target: t.translation + dir * MOVE_OUT_DIST,
                });
            }
        }
    }
    event_reader.clear();
}

pub fn update_scared_sheeps(
    mut commands: Commands,
    time: Res<Time>,
    mut sheeps: Query<
        (
            Entity,
            &Transform,
            &mut SheepTargetVel,
            &mut Decision,
            &mut IsScared,
            &NearestSheep
        ),
        With<Sheep>,
    >,
    dog: Query<&Transform, With<Dog>>,
) {
    let Ok(dog_transform) = dog.get_single() else {
        return;
    };

    for (e, t, mut walk, mut dec, mut scare, nearest) in sheeps.iter_mut() {
        if scare.time > 3. {
            *dec = Decision::Idle;
            walk.0 = Vec3::ZERO;
            commands.entity(e).remove::<IsScared>();
        } else {
            scare.time += time.delta_seconds();

            let dog_dpos = t.translation - dog_transform.translation;
            let dog_distance = dog_dpos.length();

            let dir = dog_dpos.normalize_or_zero();

            let speed_amount = (SHEEP_SPEED * (1.0 - dog_distance / SCARE_MAX_DIST)
                + SHEEP_SPEED * RANDOM_WALK_SPEED_MULTIPLIER)
                .max(SHEEP_SPEED * RANDOM_WALK_SPEED_MULTIPLIER);

            let nearest = &nearest.0;
            let mut mean_nearest_sheep = Vec3::ZERO;
            let mut count = 0.0;
            for (pos, _) in nearest.iter().skip(1) {
                // if (*pos - t.translation).length() < 5.0 {
                    let ddog = *pos - dog_transform.translation;
                    if ddog.dot(dog_dpos) >= 0.0 {
                        mean_nearest_sheep += *pos;
                        count += 1.0;
                    }
                // }
            }
            if count > 0.0 {
                let mean_nearest_sheep = mean_nearest_sheep / (count as f32);
                if (mean_nearest_sheep - dog_transform.translation).length() < dog_dpos.length() {
                    walk.0 = (mean_nearest_sheep - dog_transform.translation).normalize_or_zero() * speed_amount;
                    scare.last_vel = walk.0;
                } else {
                    walk.0 = ((mean_nearest_sheep - dog_transform.translation).normalize_or_zero() + (mean_nearest_sheep - t.translation).normalize_or_zero() * 1.5).normalize_or_zero()
                                 * speed_amount;
                    scare.last_vel = walk.0;
                }
            } else {
                walk.0 = dir * speed_amount;
                scare.last_vel = walk.0;
            }
            
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
    let r = level_size.0 / 1.5 / 2.0;
    let mut rng = rand::thread_rng();
    let sheep_count = 1000;

    let mut exact_sheep_count = 0;

    while exact_sheep_count < sheep_count {
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
                material: sheep_material.clone(),
                transform: Transform::from_xyz(pos.x, pos.y, pos.z)
                    .with_rotation(get_sprite_rotation())
                    .with_scale(Vec3::new(1.0, 1.0, 1.0) * 2.0),
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
            SheepTargetVel::default(),
            GameStuff,
            AutoAnim {
                set: SheepAnim::Idle,
                timer: Timer::from_seconds(0.1 + rng.gen_range(-0.01..=0.01), TimerMode::Repeating),
                current_frame: 0
            },
            NearestSheep::default()
        ));
        exact_sheep_count += 1;
    }

    commands.insert_resource(StartSheepCount(exact_sheep_count as f32));
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

type NNTree = KDTree3<Sheep>;

const PREFERED_DISTANCE: f32 = 1.0;
const PREFERED_DY: f32 = 0.1;

#[derive(Component, Default)]
pub struct NearestSheep(pub Vec<(Vec3, Option<Entity>)>);

#[derive(Component)]
pub struct UpdatedSheep;

fn update_nearest(
    mut commands: Commands,
    mut sheep : Query<(Entity, &Transform, &mut NearestSheep), (With<Sheep>, Without<UpdatedSheep>)>,
    mut updated_sheep : Query<Entity, With<UpdatedSheep>>,
    field: ResMut<NNTree>,
) {
    let max_sheep_count = 100;
    let mut idx = 0;
    for (e, t, mut nearest) in sheep.iter_mut() {
        idx += 1;
        if idx > max_sheep_count {
            break;
        }

        nearest.0 = field.k_nearest_neighbour(t.translation, 7);
        commands.entity(e).insert(UpdatedSheep);
    }

    if sheep.is_empty() {
        for e in updated_sheep.iter() {
            commands.entity(e).remove::<UpdatedSheep>();
        }
    }
}

fn collect_field(
    mut sheep: Query<
        (
            &Transform,
            &SheepTargetVel,
            &mut WalkController,
            &Velocity,
            &Decision,
            &NearestSheep
        ),
        With<Sheep>,
    >,
) {
    unsafe {
        for (t, vel, mut walk, _, dec, nearest) in sheep.iter_unsafe() {
            if *dec != Decision::Idle {
                let neighboors = &nearest.0;

                let mut sum = Vec3::ZERO;
                let mut distance_force = Vec3::ZERO;

                let mut sum_targets = Vec3::ZERO;
                let mut target_weight_sum = 0.0;

                let mut sum_dz = 0.0;
                let mut count = 0;
                for (_, n_e) in neighboors.iter().skip(1) {
                    if let Some(n_e) = n_e {
                        if let Ok((n_t, n_tar, _, n_vel, _, _)) = sheep.get(*n_e) {
                            sum += n_vel.0;
                            count += 1;

                            let dp = t.translation - n_t.translation;
                            let length = dp.length();
                            if length < PREFERED_DISTANCE {
                                distance_force += dp * (1.0 - length / PREFERED_DISTANCE);
                            }

                            if dp.z.abs() < PREFERED_DY {
                                sum_dz += (1.0 - dp.z.abs() / PREFERED_DY) * dp.z.signum();
                            }

                            sum_targets += n_vel.0;
                            target_weight_sum += n_vel.0.length();
                        }
                    }
                }

                if count > 0 {
                    let mean_targets = sum_targets / (target_weight_sum + 0.000001);
                    let dist_force = 10.0 * distance_force;
                    let dz = Vec3::new(0.0, 0.0, sum_dz);

                    let wsum = vel.0.length() + dist_force.length() + dz.length();
                    let max_length = vel
                        .0
                        .length()
                        .max(mean_targets.length())
                        .max(dist_force.length());
                    walk.target_velocity =
                        (vel.0 + dist_force + dz) / (wsum + 0.000001) * max_length;
                } else {
                    walk.target_velocity = vel.0;
                }
            }
        }
    }
}

fn set_anim_state(
    mut sheep : Query<(&mut AutoAnim<SheepAnim>, Option<&GoTo>, Option<&IdleFeeding>, Option<&IsScared>), With<Sheep>>
) {
    for (mut anim, go_to, idle, scared) in sheep.iter_mut() {
        if go_to.is_some() {
            anim.set = SheepAnim::Walk;
        } else if idle.is_some() {
            anim.set = SheepAnim::Feed;
        } else if scared.is_some() {
            anim.set = SheepAnim::Walk;
        } else {
            anim.set = SheepAnim::Idle;
        }
    }
}