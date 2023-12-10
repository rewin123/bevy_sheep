use bevy::prelude::*;
use rand::Rng;

use crate::{storyteller::{Storyteller, GlobalTask}, safe_area::{SafeArea, LandSafeArea}, test_level::LevelSize, GameSet, level_ui::TaskText, GameStuff};

pub struct ChangeSafeAreaSizePlugin;

impl Plugin for ChangeSafeAreaSizePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GlobalTask::ChangeSafeArea), (start_change_safe_area, apply_deferred).chain())
            .add_systems(Update, change_area_system.run_if(in_state(GlobalTask::ChangeSafeArea)).in_set(GameSet::Playing));
    }
}

#[derive(Component)]
pub struct ChangeSafeArea {
    pub target_scale: f32,
    pub start_scale: f32,

    pub target_pos : Vec2,
    pub start_pos : Vec2,

    pub duration: f32,
    pub time: f32,

    pub start_area: SafeArea
}

const CHANGE_DURATION: f32 = 15.0;

fn start_change_safe_area(
    mut commands: Commands,
    mut teller : ResMut<Storyteller>,
    mut areas: Query<(Entity, &mut SafeArea, &LandSafeArea)>,
    leve_size : Res<LevelSize>
) {
    let mut rng = rand::thread_rng();
    let pos = rng.gen_range(8..=20) as f32;

    for (entity, mut area, land) in areas.iter_mut() {

        let start_pos = area.get_center();
        let start_pos = Vec2::new(start_pos.x, start_pos.z);
        let target_pos = Vec2::new(start_pos.x + pos, start_pos.y + pos);

        let mut change = ChangeSafeArea {
            target_scale: 0.5,
            start_scale: 1.0,

            target_pos: target_pos,
            start_pos: start_pos,

            duration: CHANGE_DURATION,
            time: 0.0,

            start_area: area.clone()
        };

        commands.entity(entity).insert(change);
    }

    //generate circle area
    let area = SafeArea::Circle { 
        pos: Vec2::new(-pos, -pos),
        radius: leve_size.0 / 4.0
     };

     commands.spawn((
        area.clone(),
        LandSafeArea {
            start_area: area.clone()
        },
        ChangeSafeArea {
            target_scale: 1.0,
            start_scale: 0.0,

            target_pos: Vec2::new(-pos, -pos),
            start_pos: Vec2::new(-pos, -pos),

            duration: CHANGE_DURATION,
            time: 0.0,

            start_area: area
        }
     ));
}

fn change_area_system(
    mut commands: Commands,
    mut change: Query<(Entity, &mut ChangeSafeArea)>,
    time : Res<Time>,
    mut global_task : ResMut<NextState<GlobalTask>>,
    mut text : Query<&mut Text, With<TaskText>>
) {

    if change.is_empty() {
        global_task.set(GlobalTask::None);
        for mut t in text.iter_mut() {
            t.sections[0].value = "".to_string();
        }
        return;
    }

    for mut t in text.iter_mut() {
        t.sections[0].value = "The wind has changed. Sheep safe zones are changing!".to_string();
    }

    for (entity, mut change) in change.iter_mut() {
        change.time += time.delta_seconds();

        if change.time >= change.duration {
            commands.entity(entity).remove::<ChangeSafeArea>();
        } else {
            let progress = change.time / change.duration;
            let scale = change.start_scale + (change.target_scale - change.start_scale) * progress;
            let pos = change.start_pos + (change.target_pos - change.start_pos) * progress;

            let mut area = change.start_area.get_scaled(scale);
            area.set_pos(pos);

            commands.entity(entity)
                .insert(area.clone())
                .insert(LandSafeArea {
                    start_area: area
                })
                .insert(GameStuff);
        }
    }
}