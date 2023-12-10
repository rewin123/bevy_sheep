#![allow(clippy::type_complexity)]

pub mod common_storage;
pub mod debug_diagnostic;
pub mod finish_screen;
pub mod global_task;
pub mod level_ui;
pub mod menu;
pub mod physics;
pub mod player;
pub mod safe_area;
pub mod sheep;
pub mod shepherd;
pub mod sprite_material;
pub mod storyteller;
pub mod sunday;
pub mod test_level;
pub mod torch;
pub mod wolf;
pub mod ambient;

use std::f32::consts::PI;

#[cfg(feature = "dev")]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::{app::App, core_pipeline::clear_color::ClearColorConfig};

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[allow(dead_code)]
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
    // Finished State
    Finish,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    Loading,
    Menu,
    Playing,
    Finish,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>();

        //Terrible set configuration
        app.configure_sets(
            Update,
            GameSet::Loading.run_if(in_state(GameState::Loading)),
        );
        app.configure_sets(Update, GameSet::Menu.run_if(in_state(GameState::Menu)));
        app.configure_sets(
            Update,
            GameSet::Playing.run_if(in_state(GameState::Playing)),
        );
        app.configure_sets(Update, GameSet::Finish.run_if(in_state(GameState::Finish)));

        app.configure_sets(
            FixedUpdate,
            GameSet::Loading.run_if(in_state(GameState::Loading)),
        );
        app.configure_sets(FixedUpdate, GameSet::Menu.run_if(in_state(GameState::Menu)));
        app.configure_sets(
            FixedUpdate,
            GameSet::Playing.run_if(in_state(GameState::Playing)),
        );
        app.configure_sets(
            FixedUpdate,
            GameSet::Finish.run_if(in_state(GameState::Finish)),
        );

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
            wolf::WolfPlugin,
            menu::MenuPlugin,
            finish_screen::FinishScreenPlugin,
            sunday::SundayPlugin,
            shepherd::ShepherdPlugin,
            global_task::GlobalTaskPlugin,
        ));

        app.add_plugins(
            ambient::AmbientPlugin
        );

        //For long term updates
        app.insert_resource(Time::<Fixed>::from_seconds(1.0));

        app.add_systems(
            OnEnter(GameState::Playing),
            (test_level::setup, sheep::setup),
        );

        app.add_systems(Startup, (loading, camera_setup));

        app.add_systems(OnEnter(GameState::Menu), clear_game_stuff);
    }
}

pub fn get_sprite_rotation() -> Quat {
    Quat::from_euler(EulerRot::XYZ, -PI / 2.0 - PI / 4.0, 0.0, 0.0)
}

fn loading(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu);
}

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            hdr: true,
            ..default()
        },
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        ..default()
    });
}

#[derive(Component)]
pub struct GameStuff;

fn clear_game_stuff(mut commands: Commands, game_stuff: Query<Entity, With<GameStuff>>) {
    for entity in game_stuff.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
