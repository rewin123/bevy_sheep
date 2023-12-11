use bevy::prelude::*;

use crate::{storyteller::LevelTimer, GameStuff, player::Stamina, GameSet};

pub struct LevelUiPlugin;

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateLevelUi>()
            .add_systems(Update, create_level_ui_system)
            .add_systems(Update, show_stamina.in_set(GameSet::Playing));
    }
}

#[derive(Event)]
pub struct CreateLevelUi;

#[derive(Component)]
pub struct LevelUi;


#[derive(Component)]
pub struct TaskText;

#[derive(Component)]
pub struct StaminaState;

fn create_level_ui_system(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut ev_create_level_ui: EventReader<CreateLevelUi>,
) {
    if ev_create_level_ui.is_empty() {
        return;
    }

    let mut text_style = TextStyle::default();
    text_style.font_size = 24.0;

    //Spawn top info bar
    commands
    .spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(60.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    min_height: Val::Px(80.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.5)),
                ..default()
            },
            LevelUi,
            GameStuff,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section("", text_style.clone()),
                LevelUi,
                LevelTimer,
            ));

            parent.spawn((
                TextBundle::from_section(
                    "",
                    text_style.clone()
                ),
                LevelUi,
                TaskText
            ));

            spawn_bar(parent);
        });
    });

    ev_create_level_ui.clear();
}

fn spawn_bar(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                height: Val::Px(10.0),
                width: Val::Px(200.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {

            parent.spawn(TextBundle::from_section("Stamina", TextStyle::default()));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(100.),
                        height: Val::Px(10.),
                        padding: UiRect::all(Val::Px(1.)),
                        align_items: AlignItems::Stretch,
                        top: Val::Px(2.0),
                        left: Val::Px(6.0),
                        ..Default::default()
                    },
                    background_color: Color::BLACK.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                width : Val::Percent(50.0),
                                ..Default::default()
                            },
                            background_color: Color::GREEN.into(),
                            ..Default::default()
                        },
                        StaminaState,
                    ));
                });
        });
}

fn show_stamina(
    mut query: Query<(&mut Style, &mut BackgroundColor), With<StaminaState>>,
    staminas : Query<&Stamina>
) {
    let Ok(stamina) = staminas.get_single() else {
        warn!("Stamina not found");
        return;
    };

    let Ok((mut style, mut background_color)) = query.get_single_mut() else {
        warn!("Stamina ui not found");
        return;
    };

    style.width = Val::Percent(stamina.value * 100.0);

    if stamina.blocked {
        background_color.0 = Color::ORANGE_RED;
    } else {
        background_color.0 = Color::GREEN;
    }
}