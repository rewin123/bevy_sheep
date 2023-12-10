pub mod collect_sheep_in_area;
pub mod sheep_escape;
pub mod torch_blinking;
pub mod wolf_attack;
pub mod change_safe_area_size;
pub mod evening_warning;

use bevy::prelude::*;

pub struct GlobalTaskPlugin;

impl Plugin for GlobalTaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sheep_escape::SheepEscapePlugin,
            torch_blinking::TorchBlinkingPlugin,
            change_safe_area_size::ChangeSafeAreaSizePlugin,
            evening_warning::EveningWarningPlugin,
        ));
    }
}
