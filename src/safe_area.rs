//safe area description and logic

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::sheep::Sheep;

pub struct SafeAreaPlugin;

impl Plugin for SafeAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_safe_area);

        app.add_systems(FixedUpdate, count_sheeps);

        app.init_resource::<SheepCounter>();
    }
}

#[derive(Component)]
pub enum SafeArea {
    Rect { pos: Vec2, size: Vec2 },
    Ellipse { pos1: Vec2, pos2: Vec2, radius: f32 },
}

impl SafeArea {
    fn in_area(&self, sheep_pos: Vec2) -> bool {
        match self {
            SafeArea::Rect { pos, size } => {
                let dx = sheep_pos.x - pos.x;
                let dy = sheep_pos.y - pos.y;

                dx.abs() < size.x / 2.0 && dy.abs() < size.y / 2.0
            },
            SafeArea::Ellipse { pos1, pos2, radius } => {
                let d = (*pos1 - *pos2).length();
                let r = (*pos1 - sheep_pos).length();
                r * r <= d * d
            },
        }
    }
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

#[derive(Resource, Default)]
pub struct SheepCounter {
    pub count: u32,
}

fn count_sheeps(
    mut safe_areas : Query<&SafeArea>,
    mut sheep : Query<&Transform, With<Sheep>>,
    mut counter : ResMut<SheepCounter>,
) {
    let mut count = 0;
    for safe_area in safe_areas.iter() {
        for sheep in sheep.iter() {
            if safe_area.in_area(Vec2::new(sheep.translation.x, sheep.translation.z)) {
                count += 1;
            }
        }
    }
    counter.count = count;
}