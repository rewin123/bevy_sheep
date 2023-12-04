#![allow(clippy::type_complexity)]


use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
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

        app.add_systems(Startup, just_test_setup);
    }
}

fn just_test_setup(
    mut commands : Commands
) {
    commands.spawn(SpriteBundle {
        sprite: Sprite { 
            color: Color::WHITE,
            rect : Some(Rect::new(0.0, 0.0, 100.0, 100.0)),
            ..default()
        },
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}