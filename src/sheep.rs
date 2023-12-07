use std::f32::consts::PI;

use bevy::prelude::*;
use rand::{Rng, rngs::ThreadRng};

use crate::{
    get_sprite_rotation,
    physics::{Velocity, WalkController},
    player::{Bark, DOG_SPEED, Dog},
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

//IDLE FEEDING must be large enough so that player can see sheep and react for escapes
const IDLE_FEEDING_TIME: f32 = 5.0;
const IDLE_FEEDING_TIME_RANGE: f32 = 1.5;

const MOVE_IN_DIST: f32 = 11.0;
const MOVE_OUT_DIST: f32 = 10.0;

const SCARE_RADIUS: f32 = 5.0;
const SCARE_MAX_DIST: f32 = 20.0;

pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StateChance>();

        app.add_systems(Update, (scared_sheeps, update_scared_sheeps));

        app.add_systems(Update, (sheep_state,));

        //random walk
        app.add_event::<InitRandomWalk>()
            .add_systems(Update, (init_random_walk, goto_system));

        //idle feeding
        app.add_systems(Update, idle_feeding_system);

        // Move to safearea
        app.add_event::<SafeAreaWalk>()
            .add_systems(Update, (init_safeareawalk_walk, ));

        app.add_event::<EscapeWalk>()
            .add_systems(Update, init_escape);

        app.register_type::<StateChance>()
            .register_type::<Decision>()
            .register_type::<IsScared>();
    }
}

#[derive(Default, PartialEq, Debug, Clone, Component, Reflect)]
pub struct Sheep;

#[derive(Default, PartialEq, Debug, Clone, Component, Reflect)]
#[reflect(Component, Default)]
pub struct IsScared {
    time: f32,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Component, Reflect, Copy)]
#[reflect(Component, Default)]
pub enum Decision {
    #[default]
    Idle,  //set sheep state to waiting next decision. Not just be idle. For stending we will use Feed or deleted IdleFeeding. We need to have transition states to move to next decision. Addition set of moves will be seen as sheep has plan to move and will be nicely waiting
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
                (1.0, Decision::RandomWalk),
                (1.0, Decision::MoveToSafeArea),
                (0.5, Decision::Escape),
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
    fn select_next(&self, rng : &mut ThreadRng) -> Decision {
        let mut sum = 0.0; //This decisicion selection is based on weights, not prop graph. Just for testing and more stable behavior. Dont change please
        let p = rng.gen_range(0.0..1.0);
        for (w, d) in &self.next_state {
            sum += *w;
            if p < sum {
                return *d;
            }
        }
        return Decision::Idle;
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
                    .remove::<IdleFeeding>()
                    .remove::<GoTo>();
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
        &mut WalkController,
        &mut Decision,
        &GoTo,
    )>,
) {
    for (e, t, mut v, mut dec, rw) in &mut goto_query.iter_mut() {
        if t.translation.distance(rw.target) < RANDOM_WALK_ACCEPT_RADIUS {
            v.target_velocity = Vec3::ZERO;
            commands.entity(e).remove::<GoTo>();
            *dec = Decision::Idle;
        } else {
            v.target_velocity = (rw.target - t.translation).normalize()
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
    time: Res<Time>,
    mut sheeps: Query<(Entity, &mut Decision, &mut Sheep), Without<IsScared>>,
    mut init_random_walk: EventWriter<InitRandomWalk>,
    mut init_safe_walk: EventWriter<SafeAreaWalk>,
    mut init_escape_walk: EventWriter<EscapeWalk>,
) {
    let mut rand = rand::thread_rng();
    for (e, mut dec, mut sheep) in &mut sheeps.iter_mut() {
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
    mut sheeps: Query<(Entity, &Transform, &mut WalkController, &mut Decision, &mut IsScared), With<Sheep>>,
    dog : Query<&Transform, With<Dog>>,
    safeareas: Query<&SafeArea>
) {
    let Ok(dog_transform) = dog.get_single() else {
        return;
    };

    for (e, t, mut walk, mut dec, mut scare) in sheeps.iter_mut() {
        if scare.time > 2. {
            *dec = Decision::Idle;
            walk.target_velocity = Vec3::ZERO;
            commands.entity(e).remove::<IsScared>();
        } else {
            scare.time += time.delta_seconds();

            let dog_dpos = dog_transform.translation - t.translation;
            let dog_distance = dog_dpos.length();

            if dog_distance < SCARE_RADIUS {
                scare.time = 0.0;
            }

            let dir = dog_dpos.normalize_or_zero();

            let mut nearest_sa = None;
            let mut nearest_dist = f32::MAX;
            for sa in safeareas.iter() {
                let dist = t.translation.distance(sa.get_center());
                if dist < nearest_dist {
                    nearest_sa = Some(sa);
                    nearest_dist = dist;
                }
            }

            let speed_amount = SHEEP_SPEED * (1.0 - dog_distance / SCARE_MAX_DIST);

            if let Some(sa) = nearest_sa {
                let dir_to_sa = (sa.get_center() - t.translation).normalize_or_zero();

                if dir_to_sa.dot(dir) > 0.0 {
                    walk.target_velocity = -dir * speed_amount;
                } else {
                    walk.target_velocity = (-dir + dir_to_sa).normalize_or_zero() * speed_amount;
                }
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
