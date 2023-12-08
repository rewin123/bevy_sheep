use bevy::prelude::*;

use crate::{GameState, storyteller::{Score, FailReason}, GameSet};

pub struct FinishScreenPlugin;

impl Plugin for FinishScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Finish), setup_finish_screen);

        app.add_systems(OnExit(GameState::Finish), cleanup_finish_screen);
        app.add_systems(Update, finish_screen_system.in_set(GameSet::Finish));
    }
}

#[derive(Component)]
struct FinishScreen;

fn setup_finish_screen(
    mut commands: Commands,
    score : Res<Score>,
    fail : Option<Res<FailReason>>
) {
    let mut text_style = TextStyle::default();
    text_style.font_size = 24.0;

    commands.spawn((FinishScreen, NodeBundle {
        style : Style { 
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::rgba(0.15, 0.15, 0.15, 0.5).into(),
        ..default()
    })).with_children(|parent| {
        let text = if let Some(fail) = fail {
            match *fail {
                FailReason::SheepDied => "Uh-oh. \nWhat bad luck, half the sheep have been eaten, try again :( \nIf you escape from the vampire.",
                FailReason::TaskFailed => "Uh-oh. \nYou failed the task, try again :( \nIf you escape from the vampire.",
            }
        } else {
            "Congratulations! \nYou survived your workday! \nYou did well, the vampire is waiting for you tomorrow."
        };

        parent.spawn(TextBundle::from_section(
            format!("{} \nScore: {:.1}", text, score.0), 
            TextStyle::default()
        ));

        parent.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
            border_color: Color::WHITE.into(),
            ..default()
        }).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Ok",
                text_style.clone()
            ));
        });
    });
}

fn cleanup_finish_screen(
    mut commands: Commands,
    query: Query<Entity, With<FinishScreen>>
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn finish_screen_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::Menu);
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::rgb(0.15, 0.15, 0.15).into();
            }
        }
    }
}