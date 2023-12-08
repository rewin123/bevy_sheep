use bevy::prelude::*;

use crate::{storyteller::LevelTimer, GameStuff};

pub struct LevelUiPlugin;

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateLevelUi>()
            .add_systems(Update, create_level_ui_system);
    }
}

#[derive(Event)]
pub struct CreateLevelUi;

#[derive(Component)]
pub struct LevelUi;

fn create_level_ui_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_create_level_ui: EventReader<CreateLevelUi>,
) {
    if ev_create_level_ui.is_empty() {
        return;
    }

    let mut text_style = TextStyle::default();
    text_style.font_size = 24.0;

    //Spawn top info bar
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,

                    align_self: AlignSelf::Stretch,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.5)),
                ..default()
            },
            LevelUi,
            GameStuff
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section("", text_style.clone()),
                LevelUi,
                LevelTimer,
            ));
        });

    ev_create_level_ui.clear();
}
