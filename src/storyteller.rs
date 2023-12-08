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
        Decision, GoTo, IdleFeeding, IsScared, Sheep, RANDOM_WALK_SPEED_MULTIPLIER, SHEEP_SPEED,
    },
    test_level::LevelSize,
};

pub struct StorytellerPlugin;

impl Plugin for StorytellerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Storyteller {
            level_start_time: 0.0,
            level_duration: 4.0 * 60.0,
            next_wave: None,
        })
        .add_systems(Update, (storyteller_system, level_timer))
        .add_systems(PostStartup, setup_start_time);
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

fn setup_start_time(mut teller: ResMut<Storyteller>, time: Res<Time>) {
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
) {
    for mut timer in timers.iter_mut() {
        let level_time = time.elapsed_seconds() - teller.level_start_time;
        if (teller.level_duration - level_time > 0.0) {
            let dur = Duration::from_secs_f32(teller.level_duration - level_time);

            timer.sections[0].value =
                format!("{:02}:{:02}", dur.as_secs() / 60, dur.as_secs() % 60);
        } else {
            timer.sections[0].value = format!("{:02}:{:02}", 0, 0);
        }
    }
}
