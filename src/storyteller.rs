//This global AI is responsible for creating problems for player
//This module will be determine where and how sheep will be try to escape from safe zone

use std::{
    f32::consts::{E, PI},
    time::Duration,
};

use bevy::prelude::*;
use rand::Rng;

use crate::{
    player::{Dog, DOG_SPEED},
    sheep::{
        Decision, GoTo, IdleFeeding, IsScared, Sheep, StartSheepCount,
        RANDOM_WALK_SPEED_MULTIPLIER, SHEEP_SPEED,
    },
    test_level::LevelSize,
    GameSet, GameState,
};

pub struct StorytellerPlugin;

impl Plugin for StorytellerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Storyteller {
            level_start_time: 0.0,
            level_duration: 4.0 * 60.0,
            next_wave: None,
        })
        .init_resource::<Score>()
        .add_systems(
            Update,
            (storyteller_system, level_timer).in_set(GameSet::Playing),
        )
        .add_systems(OnEnter(GameState::Playing), setup_start_time)
        .add_systems(
            FixedUpdate,
            (score_system, fail_system).in_set(GameSet::Playing),
        )
        .add_state::<GlobalTask>();
    }
}

#[derive(Debug, Clone)]
pub struct SheepWave {
    pub count: usize,
    pub beams: usize,
    pub time: f32,
}

#[derive(Resource)]
pub struct Storyteller {
    pub level_start_time: f32,
    pub level_duration: f32,
    pub next_wave: Option<SheepWave>,
}

impl Storyteller {
    pub fn get_level_time(&self, time: &Time) -> f32 {
        time.elapsed_seconds() - self.level_start_time
    }

    pub fn get_level_unfirom_time(&self, time: &Time) -> f32 {
        self.get_level_time(time) / self.level_duration
    }
}

#[derive(Resource, Default)]
pub struct Score(pub f32);

fn setup_start_time(mut commands: Commands, mut teller: ResMut<Storyteller>, time: Res<Time>) {
    commands.remove_resource::<FailReason>();
    teller.level_start_time = time.elapsed_seconds();
}

fn storyteller_system(
    mut commands: Commands,
    sheep: Query<(Entity, &Transform), (With<Sheep>, Without<IsScared>, Without<GoTo>)>,
    mut teller: ResMut<Storyteller>,
    time: Res<Time>,
    level_size: Res<LevelSize>,
    dog: Query<&Transform, With<Dog>>,
) {
    let Ok(dog_transform) = dog.get_single() else {
        return;
    };
    if teller.next_wave.is_none() {
        let level_time = time.elapsed_seconds() - teller.level_start_time;
        let unfiorm_time = level_time / teller.level_duration;

        let sheep_count = sheep.iter().count() as f32;

        let c = sheep_count * unfiorm_time * 0.2 + 1.0 + 0.05 * sheep_count;
        let dt = 10.0 - 3.0 * unfiorm_time;
        let n = 1.0 + 2.0 * unfiorm_time;

        teller.next_wave = Some(SheepWave {
            count: c as usize,
            beams: n as usize,
            time: time.elapsed_seconds() + dt,
        });

        info!("Next wave: {:?}", teller.next_wave);
    } else {
        let wave = teller.next_wave.as_ref().unwrap().clone();
        let cur_time = time.elapsed_seconds();
        if wave.time <= cur_time {
            teller.next_wave = None;

            let mut rand = rand::thread_rng();

            let split_c = (wave.count / wave.beams).max(1);
            for _ in 0..wave.beams {
                let random_dir =
                    Vec3::new(rand.gen_range(-1.0..1.0), 0.0, rand.gen_range(-1.0..1.0))
                        .normalize();
                //create a sorted list of sheep which far in direction
                let mut sorted_sheep = sheep
                    .iter()
                    .map(|(e, t)| {
                        let dir = t.translation;
                        let proj_dir = dir.dot(random_dir);
                        let dist_to_dog = (t.translation - dog_transform.translation).length();
                        (e, dir, proj_dir + dist_to_dog)
                    })
                    .collect::<Vec<_>>();
                sorted_sheep.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

                //send split_c sheep in that direction
                for i in 0..split_c {
                    if let Some((e, pos, dist)) = sorted_sheep.get(i) {
                        info!("Sending {:?} with {:?}", e, dist);
                        commands
                            .entity(*e)
                            .insert(GoTo {
                                target: *pos + level_size.0 * 2.0 * random_dir,
                            })
                            .insert(Decision::Escape)
                            .remove::<IdleFeeding>();
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct LevelTimer;

fn level_timer(
    mut timers: Query<&mut Text, With<LevelTimer>>,
    teller: Res<Storyteller>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    score: Res<Score>,
) {
    for mut timer in timers.iter_mut() {
        let level_time = time.elapsed_seconds() - teller.level_start_time;
        if teller.level_duration - level_time > 0.0 {
            let dur = Duration::from_secs_f32(teller.level_duration - level_time);

            let time = format!("{:02}:{:02}", dur.as_secs() / 60, dur.as_secs() % 60);
            let score_text = format!("Score: {:.1}", score.0);

            timer.sections[0].value = format!("{}\n{}", time, score_text);
        } else {
            timer.sections[0].value = format!("{:02}:{:02}", 0, 0);
            next_state.set(GameState::Finish);
        }
    }
}

fn score_system(
    mut score: ResMut<Score>,
    mut alived_sheep: Query<&Sheep>,
    mut teller: ResMut<Storyteller>,
    time: Res<Time>,
    start_sheep_count: Res<StartSheepCount>,
) {
    let lived_sheep = alived_sheep.iter().count() as f32 / start_sheep_count.0;
    score.0 = lived_sheep * time.elapsed_seconds();
}

fn fail_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    alived_sheep: Query<&Sheep>,
    start_sheep_count: Res<StartSheepCount>,
) {
    if (alived_sheep.iter().count() as f32 / start_sheep_count.0) < 0.5 {
        next_state.set(GameState::Finish);
        commands.insert_resource(FailReason::SheepDied);
    }
}

#[derive(Resource)]
pub enum FailReason {
    SheepDied,
    TaskFailed,
}

pub enum TaskStatus {
    Active,
    Done,
    Failed,
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum GlobalTask {
    #[default]
    None,
    SheepEscape,
    WolfAttack,
    CollectSheepInArea,
}
