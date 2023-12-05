use bevy::prelude::*;

pub struct DiagnosticPlugin;

impl Plugin for DiagnosticPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (setup_diagnostic_panel, apply_deferred, setup_counter),
        )
        .add_systems(Update, (fps_counting,));
    }
}

#[derive(Component)]
pub struct DiagnosticPanel;

pub fn setup_diagnostic_panel(mut commands: Commands) {
    commands.spawn(NodeBundle {
        style: Style {
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Px(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.5)),
        ..default()
    });
}

#[derive(Component)]
pub struct FrameCounter;

pub fn setup_counter(mut commands: Commands, panels: Query<Entity, With<DiagnosticPanel>>) {
    let frame_counter = commands
        .spawn(TextBundle::from_section("FPS: ", TextStyle::default()))
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
