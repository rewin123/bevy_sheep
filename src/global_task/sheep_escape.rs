use bevy::prelude::*;
use rand::Rng;

use crate::{storyteller::{Storyteller, GlobalTask, FailReason}, sunday::{DayState, EpisodeTime}, sheep::{Sheep, IsScared, GoTo, IdleFeeding, Decision}, player::Dog, test_level::LevelSize, GameSet, GameState, level_ui::TaskText};

pub struct SheepEscapePlugin;


impl Plugin for SheepEscapePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GlobalTask::SheepEscape), generate_new_wave)
            .add_systems(Update, 
                (
                    apply_deferred,
                    check_wave_finish,
                    wave_executor,
            ).chain().run_if(in_state(GlobalTask::SheepEscape)).in_set(GameSet::Playing))
            
            .init_resource::<NextWave>()
            .init_resource::<SheepWaveStatus>();
    }
}

#[derive(Resource, Default)]
pub struct NextWave(Option<SheepWave>);

#[derive(Component)]
pub struct ShawshankRedemption;

#[derive(Debug, Clone)]
pub struct SheepWave {
    pub count: usize,
    pub beams: usize,
    pub time: f32,
}

#[derive(Resource, Default)]
pub struct SheepWaveStatus {
    pub start_count : usize,
    pub sheep : Vec<Entity>
}

fn check_wave_finish(
    mut commands: Commands,
    escapers : Query<Entity, (With<ShawshankRedemption>, With<Sheep>)>,
    sheep : Query<&Sheep>,
    mut global_task : ResMut<NextState<GlobalTask>>,
    mut sheep_wave_status : ResMut<SheepWaveStatus>,
    mut game_state : ResMut<NextState<GameState>>,
    next_wave : Res<NextWave>,
    mut info_texts : Query<&mut Text, With<TaskText>>,
) {
    let loose_limit = (sheep_wave_status.start_count / 2).max(10);
    if escapers.is_empty() && next_wave.0.is_none() && sheep_wave_status.start_count != 0 {

        let mut alived_sheep = 0;
        for e in &sheep_wave_status.sheep {
            if sheep.get(*e).is_ok() {
                alived_sheep += 1;
            }
        }
        info!("WAVE FINISHED");

        if sheep_wave_status.start_count - alived_sheep > loose_limit {
            commands.insert_resource(FailReason::TaskFailed("Half of the runaway sheep were eaten :(".to_string()));
            game_state.set(GameState::Finish);
            global_task.set(GlobalTask::None);
        } else {
            global_task.set(GlobalTask::None);
        }

        sheep_wave_status.start_count = 0;
        sheep_wave_status.sheep.clear();

        for mut t in info_texts.iter_mut() {
            t.sections[0].value = "".to_string();
        }
    } else if next_wave.0.is_some() {
        for mut t in info_texts.iter_mut() {
            t.sections[0].value = "The sheep are worried, wait for it".to_string();
        }
    } else if sheep_wave_status.start_count != 0 {
        for mut t in info_texts.iter_mut() {
            t.sections[0].value = format!("{} sheep are trying to escape! Stop them! Dont lose more than {}", escapers.iter().count(), loose_limit);
        }
    }
}

fn generate_new_wave(
    mut commands: Commands,
    time: Res<Time>,
    mut next_wave: ResMut<NextWave>,
    teller : ResMut<Storyteller>,
    day_state : Res<State<DayState>>,
    episode_time : Res<EpisodeTime>,
    sheep: Query<(Entity, &Transform), (With<Sheep>, Without<IsScared>, Without<GoTo>)>,
) {
    let level_time = time.elapsed_seconds() - teller.level_start_time;
    let unfiorm_time = level_time / teller.level_duration;

    let episode_time = episode_time.0;

    if *day_state == DayState::Day {
        let sheep_count = sheep.iter().count() as f32;
        let c = sheep_count * episode_time * 0.2 + 1.0;
        let mut dt = 15.0 - 3.0 * episode_time;
        let n = 1.0 + 1.0 * episode_time;

        if level_time < 5.0 {
            dt = 5.0;
        }

        next_wave.0 = Some(SheepWave {
            count: c as usize,
            beams: n as usize,
            time: time.elapsed_seconds() + dt,
        });
    } else if *day_state == DayState::Night {
        let sheep_count = sheep.iter().count() as f32;
        let c = sheep_count * episode_time * 0.1 + 1.0;
        let dt = 15.0 - 3.0 * episode_time;
        let n = 1;

        next_wave.0 = Some(SheepWave {
            count: c as usize,
            beams: n,
            time: time.elapsed_seconds() + dt,
        });
    }

    info!("Next wave: {:?}", next_wave.0);
}

fn wave_executor(
    mut commands: Commands,
    mut next_wave : ResMut<NextWave>,
    time : Res<Time>,
    sheep: Query<(Entity, &Transform), (With<Sheep>, Without<IsScared>, Without<GoTo>)>,
    dog: Query<&Transform, With<Dog>>,
    level_size: Res<LevelSize>,
    mut sheep_wave_status : ResMut<SheepWaveStatus>,
) {
    let Ok(dog_transform) = dog.get_single() else {
        return;
    };
    
    if next_wave.0.is_some() {
        let wave = next_wave.0.as_ref().unwrap().clone();
        let cur_time = time.elapsed_seconds();
        if wave.time <= cur_time {
            next_wave.0 = None;
            *sheep_wave_status = Default::default();

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
                            .remove::<IdleFeeding>()
                            .insert(ShawshankRedemption);
                        sheep_wave_status.sheep.push(*e);
                    }
                }
            }

            info!("WAVE STARTED");
            sheep_wave_status.start_count = sheep_wave_status.sheep.len();
        }
    }
}