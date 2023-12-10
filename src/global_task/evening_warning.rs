pub use bevy::prelude::*;

use crate::{level_ui::TaskText, sunday::DayState, storyteller::GlobalTask, GameSet};

pub struct EveningWarningPlugin;

impl Plugin for EveningWarningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update, write_warning_message.run_if(in_state(GlobalTask::None)).run_if(in_state(DayState::Evening)).in_set(GameSet::Playing)
        ).add_systems(OnExit(DayState::Evening), clear_message.in_set(GameSet::Playing));
    }
}

fn write_warning_message(
    mut texts : Query<&mut Text, With<TaskText>>
) {
    for mut text in texts.iter_mut() {
        text.sections[0].value = "The night is coming! Safe zones are disappearing!\nGather the sheep near the torches!".to_string();
    }
}

fn clear_message(
    mut texts : Query<&mut Text, With<TaskText>>
) {
    for mut text in texts.iter_mut() {
        text.sections[0].value = "".to_string();
    }
}