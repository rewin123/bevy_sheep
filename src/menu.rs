use bevy::prelude::*;

use crate::{GameSet, GameState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_main_menu);
        app.add_systems(OnExit(GameState::Menu), clear_menu);
        app.add_systems(Update, button_system.in_set(GameSet::Menu));
    }
}

#[derive(Component)]
pub struct MainMenu;

fn setup_main_menu(mut commands: Commands) {
    let mut text_style = TextStyle::default();
    text_style.font_size = 24.0;
    commands
        .spawn((
            MainMenu,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "You are a shepherd dog servant of a vampire shepherd. \n
Don't let the sheep flock run away and be eaten by wolves, and fulfill the vampire's tasks. \n
If you fail, you will be replaced. \n
Good luck!\n",
                TextStyle::default(),
            ));

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::WHITE),
                    background_color: BackgroundColor(Color::BLACK),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Start", text_style.clone()));
                });
        });
}

fn clear_menu(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::DARK_GRAY);
            }
            Interaction::None => {
                *color = BackgroundColor(Color::BLACK);
            }
        }
    }
}
