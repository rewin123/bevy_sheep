pub mod collect_sheep_in_area;
pub mod sheep_escape;
pub mod wolf_attack;
pub mod torch_blinking;

use bevy::prelude::*;

pub struct GlobalTaskPlugin;

impl Plugin for GlobalTaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sheep_escape::SheepEscapePlugin,
            torch_blinking::TorchBlinkingPlugin,
        ));
    }
}
