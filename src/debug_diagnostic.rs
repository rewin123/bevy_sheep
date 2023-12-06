use bevy::prelude::*;

use crate::safe_area::SheepCounter;

const FONT_SIZE: f32 = 24.0;

pub struct DiagnosticPlugin;

impl Plugin for DiagnosticPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_diagnostic_panel,
                apply_deferred,
                setup_counter,
                setup_sheep_counter,
            )
                .chain(),
        )
        .add_systems(Update, (fps_counting, sheep_counter_text))
        .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default());
    }
}

#[derive(Component)]
pub struct DiagnosticPanel;

pub fn setup_diagnostic_panel(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Px(200.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,

                align_self: AlignSelf::Stretch,

                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.5)),
            ..default()
        })
        .insert(DiagnosticPanel);
}

#[derive(Component)]
pub struct FrameCounter;

pub fn setup_counter(mut commands: Commands, panels: Query<Entity, With<DiagnosticPanel>>) {
    let mut text_style = TextStyle::default();
    text_style.font_size = FONT_SIZE;

    let frame_counter = commands
        .spawn(TextBundle::from_section("FPS: ", text_style))
        .insert(FrameCounter)
        .id();

    if let Ok(panel) = panels.get_single() {
        commands.entity(panel).add_child(frame_counter);
    }
}

fn fps_counting(mut query: Query<&mut Text, With<FrameCounter>>, time: Res<Time>) {
    for mut text in &mut query {
        text.sections[0].value = format!("FPS: {:.0}", 1.0 / time.delta_seconds());
    }
}

#[derive(Component)]
pub struct ShipDebugCounter;

pub fn setup_sheep_counter(mut commands: Commands, panels: Query<Entity, With<DiagnosticPanel>>) {
    let mut text_style = TextStyle::default();
    text_style.font_size = FONT_SIZE;
    let sheep_counter = commands
        .spawn(TextBundle::from_section("Sheep in safe area: ", text_style))
        .insert(ShipDebugCounter)
        .id();

    if let Ok(panel) = panels.get_single() {
        commands.entity(panel).add_child(sheep_counter);
    }
}

pub fn sheep_counter_text(
    mut query: Query<&mut Text, With<ShipDebugCounter>>,
    sheep_counter: Res<SheepCounter>,
) {
    for mut text in &mut query {
        text.sections[0].value = format!("Sheep in safe area: {}", sheep_counter.count);
    }
}
