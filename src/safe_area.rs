//safe area description and logic

use std::f32::consts::PI;

use bevy::prelude::*;

pub struct SafeAreaPlugin;

impl Plugin for SafeAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_safe_area);
    }
}

#[derive(Component)]
pub enum SafeArea {
    Rect { pos: Vec2, size: Vec2 },
    Ellipse { pos1: Vec2, pos2: Vec2, radius: f32 },
}

fn draw_safe_area(mut gizmos: Gizmos, query: Query<&SafeArea>) {
    for safe_area in query.iter() {
        match safe_area {
            SafeArea::Rect { pos, size } => {
                gizmos.rect(
                    Vec3::new(pos.x, 0.001, pos.y),
                    Quat::from_euler(EulerRot::XYZ, PI / 2.0, 0.0, 0.0),
                    *size,
                    Color::GREEN,
                );
            }
            SafeArea::Ellipse { pos1, pos2, radius } => {}
        }
    }
}
