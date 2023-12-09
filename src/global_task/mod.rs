pub mod sheep_escape;
pub mod collect_sheep_in_area;
pub mod wolf_attack;

use bevy::prelude::*;

pub struct GlobalTaskPlugin;

impl Plugin for GlobalTaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sheep_escape::SheepEscapePlugin,
        ));
    }
}