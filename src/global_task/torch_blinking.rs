use bevy::{prelude::*, transform::commands, utils::hashbrown::HashSet};
use rand::{seq::SliceRandom, Rng};

use crate::{
    level_ui::TaskText,
    safe_area::SafeArea,
    sheep::Sheep,
    storyteller::{FailReason, GlobalTask},
    sunday::EpisodeTime,
    torch::{TorchBase, TorchLight, TORCH_BASE_RADIUS, TORCH_ILLUMINATION},
    GameSet, GameState,
};

pub const BAD_TORCH_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);

pub struct TorchBlinkingPlugin;

impl Plugin for TorchBlinkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalTask::TorchProblem), start_fire_problems)
            .add_systems(
                Update,
                (
                    apply_deferred,
                    delight,
                    update_delight_system,
                    apply_deferred,
                )
                    .chain()
                    .run_if(in_state(GlobalTask::TorchProblem))
                    .in_set(GameSet::Playing),
            );
    }
}

#[derive(Default, Component)]
pub struct TorchDelight {
    pub be_scared_time: f32,
    pub rest_time: f32,
    pub change_duration: f32,
}

#[derive(Resource)]
pub struct TorchDelightStatus {
    pub time_for_mission: f32,
    pub start_sheep_count: usize,
    pub max_dead_sheep: usize,
    pub torches_to_lit: Vec<Entity>,
}

fn start_fire_problems(
    mut commands: Commands,
    torches: Query<(Entity, &SafeArea), With<TorchBase>>,
    episode_time: Res<EpisodeTime>,
    sheep: Query<&Transform, With<Sheep>>,
    mut global_task: ResMut<NextState<GlobalTask>>,
) {
    let torch_count = torches.iter().count();

    let mut rand = rand::thread_rng();
    let problem_part = episode_time.0 * 0.5;

    let problem_torhes_count = (torch_count as f32 - 1.0).max(1.0);
    let change_time = 10.0;

    let mut problem_torches = torches
        .iter()
        .map(|(a, b)| (a, b, 0_usize))
        .collect::<Vec<_>>();

    for (e, safe_area, count) in problem_torches.iter_mut() {
        for sheep in sheep.iter() {
            if safe_area.in_area(Vec2::new(sheep.translation.x, sheep.translation.z)) {
                *count = *count + 1;
            }
        }
    }

    let mut problem_torches = problem_torches
        .iter()
        .filter(|(_, _, count)| *count > 0)
        .collect::<Vec<_>>();

    if (problem_torches.len() == 0) {
        global_task.set(GlobalTask::None);
        return;
    }

    while problem_torches.len() > problem_torhes_count as usize {
        let i = rand.gen_range(0..problem_torches.len());
        problem_torches.remove(i);
    }

    let mut sheep_in_torches = 0;

    for (idx, (e, safe_area, count)) in problem_torches.iter().enumerate() {
        commands.entity(*e).insert(TorchDelight {
            be_scared_time: idx as f32 * 5.0,
            rest_time: change_time,
            change_duration: change_time,
        });

        sheep_in_torches += *count;
    }

    commands.insert_resource(TorchDelightStatus {
        time_for_mission: 40.0,
        start_sheep_count: sheep.iter().count(),
        max_dead_sheep: (sheep_in_torches / 2).max(10),
        torches_to_lit: problem_torches
            .iter()
            .map(|(e, _, _)| *e)
            .collect::<Vec<_>>(),
    });

    info!("Start torch problem with {} torches", problem_torches.len());
}

fn update_delight_system(
    mut commands: Commands,
    mut status: ResMut<TorchDelightStatus>,
    mut texts: Query<&mut Text, With<TaskText>>,
    torches: Query<(&TorchBase, Option<&TorchDelight>)>,
    time: Res<Time>,
    mut gamestate: ResMut<NextState<GameState>>,
    mut global_task: ResMut<NextState<GlobalTask>>,
    sheep: Query<&Sheep>,
) {
    if !status.torches_to_lit.is_empty() {
        status.time_for_mission -= time.delta_seconds();
        if status.time_for_mission < 0.0 {
            gamestate.set(GameState::Finish);
            commands.insert_resource(FailReason::TaskFailed(format!(
                "Not all the torches were lit. You should be better at waking up ancient vampires."
            )));
            return;
        }

        let mut ok_torches = 0;
        for e in status.torches_to_lit.iter() {
            if let Ok((base, delight)) = torches.get(*e) {
                if base.lit && delight.is_none() {
                    ok_torches += 1;
                }
            }
        }

        if ok_torches == status.torches_to_lit.len() {
            global_task.set(GlobalTask::None);
        } else {
            let lived_sheep_count = sheep.iter().count();
            if status.start_sheep_count - lived_sheep_count > status.max_dead_sheep {
                gamestate.set(GameState::Finish);
                commands.insert_resource(FailReason::TaskFailed(format!(
                    "Too many sheep was eaten. You should be better at waking up ancient vampires."
                )));
                return;
            }
        }

        if let Ok(mut text) = texts.get_single_mut() {
            text.sections[0].value = format!("The torches are going out! Urgently wake up the shepherd to light the torches!\n{} / {} torches lit\n{} seconds left\nDont let to eat more then {}", ok_torches, status.torches_to_lit.len(), status.time_for_mission, status.max_dead_sheep);
        }
    }
}

fn delight(
    mut commands: Commands,
    mut torches: Query<(Entity, &Transform, &mut TorchDelight, &mut TorchBase)>,
    mut lights: Query<&mut SpotLight, With<TorchLight>>,
    time: Res<Time>,
) {
    for (e, tr, mut delight, mut base) in &mut torches {
        delight.be_scared_time -= time.delta_seconds();
        if delight.be_scared_time > 0.0 {
            if let Ok(mut light) = lights.get_mut(base.light) {
                light.color = Color::ORANGE_RED;
            }
            continue;
        }

        delight.rest_time -= time.delta_seconds();

        if delight.rest_time < 0.0 {
            if let Ok(mut light) = lights.get_mut(base.light) {
                light.intensity = 0.0;
            }
            base.lit = false;
            commands
                .entity(e)
                .remove::<TorchDelight>()
                .remove::<SafeArea>();
        } else if let Ok(mut light) = lights.get_mut(base.light) {
            light.color = BAD_TORCH_COLOR;
            light.intensity = TORCH_ILLUMINATION * delight.rest_time / delight.change_duration;

            let new_r = TORCH_BASE_RADIUS * delight.rest_time / delight.change_duration;
            let h = TORCH_BASE_RADIUS * 0.5;

            let outer_angle = (new_r / h).atan();
            let inner_angle = outer_angle * 0.85;

            light.inner_angle = inner_angle;
            light.outer_angle = outer_angle;

            commands.entity(e).insert(SafeArea::Circle {
                pos: Vec2::new(tr.translation.x, tr.translation.z),
                radius: new_r,
            });
        }
    }
}
