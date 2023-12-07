#![allow(clippy::type_complexity)]

pub mod common_storage;
pub mod debug_diagnostic;
pub mod level_ui;
pub mod physics;
pub mod player;
pub mod safe_area;
pub mod sheep;
pub mod sprite_material;
pub mod storyteller;
pub mod test_level;
pub mod torch;

use std::f32::consts::PI;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[allow(dead_code)]
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>();

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }

        app.add_plugins(debug_diagnostic::DiagnosticPlugin);

        app.add_plugins((
            player::PlayerPlugin,
            physics::PhysicsPlugin,
            common_storage::CommonStoragePlugin,
            torch::TorchPlugin,
            safe_area::SafeAreaPlugin,
            sprite_material::SpriteMaterialPlugin,
            sheep::SheepPlugin,
            storyteller::StorytellerPlugin,
            level_ui::LevelUiPlugin,
        ));

        //For long term updates
        app.insert_resource(Time::<Fixed>::from_seconds(1.0));

        app.add_systems(Startup, (test_level::setup, sheep::setup));
    }
}

pub fn get_sprite_rotation() -> Quat {
    Quat::from_euler(EulerRot::XYZ, -PI / 2.0 - PI / 4.0, 0.0, 0.0)
}
