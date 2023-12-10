use bevy::{prelude::*, audio::{PlaybackMode, VolumeLevel, Volume}};

pub struct AmbientPlugin;

#[derive(Resource)]
pub struct ForestAmbient;

impl Plugin for AmbientPlugin {
    fn build(&self, app: &mut App) {
        app.
            add_systems(Startup, startup);
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(AudioBundle {
        source: asset_server.load("audio/forest.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..Default::default()
        },
        ..default()
    });

    commands.spawn(AudioBundle {
        source: asset_server.load("audio/sheep.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..Default::default()
        },
        ..default()
    });
}

