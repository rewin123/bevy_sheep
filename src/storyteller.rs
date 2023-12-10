//This global AI is responsible for creating problems for player
//This module will be determine where and how sheep will be try to escape from safe zone

use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    player::Dog,
    sheep::{Sheep, StartSheepCount, IsScared, GoTo},
    sunday::{DayState, EpisodeTime},
    GameSet, GameState, test_level::LevelSize,
};

pub struct StorytellerPlugin;

impl Plugin for StorytellerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Storyteller {
            level_start_time: 0.0,
            level_duration: 6.0 * 60.0,
            safearea_count: 1,
            change_safe_area_was_spanwed: false
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
        .init_resource::<NextTaskDelay>()
        .add_systems(OnEnter(GlobalTask::None), setup_delay)
        .add_state::<GlobalTask>();
    }
}

#[derive(Resource)]
pub struct Storyteller {
    pub level_start_time: f32,
    pub level_duration: f32,
    pub safearea_count: u8,

    pub change_safe_area_was_spanwed: bool,
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

#[derive(Resource)]
pub struct NextTaskDelay(pub f32);

impl Default for NextTaskDelay {
    fn default() -> Self {
        Self(10.0)
    }
}

fn setup_delay(mut delay: ResMut<NextTaskDelay>) {
    *delay = NextTaskDelay(10.0);
}

fn storyteller_system(
    mut commands: Commands,
    sheep: Query<(Entity, &Transform), (With<Sheep>, Without<IsScared>, Without<GoTo>)>,
    mut teller: ResMut<Storyteller>,
    time: Res<Time>,
    level_size: Res<LevelSize>,
    dog: Query<&Transform, With<Dog>>,

    current_task: Res<State<GlobalTask>>,
    mut next_task: ResMut<NextState<GlobalTask>>,
    day_state: Res<State<DayState>>,
    episode_time: Res<EpisodeTime>,
    mut delay: ResMut<NextTaskDelay>,
) {
    if *current_task != GlobalTask::None {
        return;
    }

    let Ok(_dog_transform) = dog.get_single() else {
        return;
    };
    if *current_task == GlobalTask::None {
        delay.0 -= time.delta_seconds();
        if delay.0 > 0.0 {
            return;
        }
        let level_time = time.elapsed_seconds() - teller.level_start_time;
        let unfiorm_time = level_time / teller.level_duration;

        // let episode_time = episode_time.0;

        match &day_state.get() {
            DayState::Day => {
                if episode_time.0 > 0.5 && !teller.change_safe_area_was_spanwed {
                    teller.change_safe_area_was_spanwed = true;
                    next_task.set(GlobalTask::ChangeSafeArea);
                } else {
                    next_task.set(GlobalTask::SheepEscape);
                }
            }
            DayState::Evening => {}
            DayState::Night => {
                let mut rng = rand::thread_rng();
                let rand_choise = rng.gen_range(0..2);
                if rand_choise == 0 {
                    next_task.set(GlobalTask::SheepEscape);
                } else if rand_choise == 1 {
                    next_task.set(GlobalTask::TorchProblem);
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
    alived_sheep: Query<&Sheep>,
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
    TaskFailed(String),
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum GlobalTask {
    #[default]
    None,
    SheepEscape,
    WolfAttack,
    CollectSheepInArea,
    TorchProblem,
    ChangeSafeArea
}
