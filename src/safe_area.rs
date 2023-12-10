//safe area description and logic

use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

use crate::sheep::Sheep;

pub struct SafeAreaPlugin;

impl Plugin for SafeAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_safe_area);

        app.add_systems(FixedUpdate, count_sheeps);

        app.init_resource::<SheepCounter>();
    }
}

#[derive(Component, Clone)]
pub enum SafeArea {
    Rect { pos: Vec2, size: Vec2 },
    Circle { pos: Vec2, radius: f32 },
}

#[derive(Component, Clone)]
pub struct LandSafeArea {
    pub start_area: SafeArea,
} //Mark for day safe area

impl SafeArea {
    pub fn in_area(&self, sheep_pos: Vec2) -> bool {
        match self {
            SafeArea::Rect { pos, size } => {
                let dx = sheep_pos.x - pos.x;
                let dy = sheep_pos.y - pos.y;

                dx.abs() < size.x / 2.0 && dy.abs() < size.y / 2.0
            }
            SafeArea::Circle { pos, radius } => (*pos - sheep_pos).length() < *radius,
        }
    }

    pub fn set_pos(&mut self, mew_pos: Vec2) {
        match self {
            SafeArea::Rect { pos, size: _ } => {
                *pos = mew_pos;
            }
            SafeArea::Circle { pos, radius: _ } => {
                *pos = mew_pos;
            }
        }
    }

    pub fn downscale(&mut self, scale: f32) {
        match self {
            SafeArea::Rect { pos: _, size } => {
                *size = *size / Vec2::new(scale, scale);
            }
            SafeArea::Circle { pos: _, radius } => {
                *radius = *radius / scale;
            }
        }
    }

    pub fn get_random_point_inside(&self, level_size: f32) -> Vec3 {
        let mut rng = rand::thread_rng();
        let v2 = (0..)
            .map(|_| Vec2 {
                x: rng.gen_range(-level_size..level_size),
                y: rng.gen_range(-level_size..level_size),
            })
            .find(|point| self.in_area(*point))
            .unwrap();
        Vec3 {
            x: v2.x,
            y: 0.0,
            z: v2.y,
        }
    }

    pub fn get_center(&self) -> Vec3 {
        match self {
            SafeArea::Rect { pos, size: _ } => Vec3::new(pos.x, 0.0, pos.y),
            SafeArea::Circle { pos, radius: _ } => Vec3::new(pos.x, 0.0, pos.y),
        }
    }

    pub fn get_center_2d(&self) -> Vec2 {
        match self {
            SafeArea::Rect { pos, size: _ } => Vec2::new(pos.x, pos.y),
            SafeArea::Circle { pos, radius: _ } => Vec2::new(pos.x, pos.y),
        }
    }

    pub fn get_width(&self) -> f32 {
        match self {
            SafeArea::Rect { pos: _, size } => size.x,
            SafeArea::Circle { pos: _, radius } => *radius,
        }
    }

    pub fn get_scaled(&self, scale: f32) -> SafeArea {
        match self {
            SafeArea::Rect { pos, size } => SafeArea::Rect {
                pos: *pos,
                size: *size * scale,
            },
            SafeArea::Circle { pos, radius } => SafeArea::Circle {
                pos: *pos,
                radius: *radius * scale,
            },
        }
    }
}

#[derive(Component)]
pub struct HiddenSafeArea;

fn draw_safe_area(mut gizmos: Gizmos, query: Query<&SafeArea, Without<HiddenSafeArea>>) {
    for safe_area in query.iter() {
        match safe_area {
            SafeArea::Rect { pos, size } => {
                gizmos.rect(
                    Vec3::new(pos.x, 0.001, pos.y),
                    Quat::from_euler(EulerRot::XYZ, PI / 2.0, 0.0, 0.0),
                    *size,
                    Color::RED,
                );
            }
            SafeArea::Circle { pos, radius } => {
                gizmos.circle(Vec3::new(pos.x, 0.001, pos.y), Vec3::Y, *radius, Color::RED);
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct SheepCounter {
    pub count: u32,
}

#[derive(Component)]
pub struct OutOfSafeArea;

fn count_sheeps(
    mut commands: Commands,
    safe_areas: Query<&SafeArea>,
    sheep: Query<(Entity, &Transform), With<Sheep>>,
    mut counter: ResMut<SheepCounter>,
) {
    let mut count = 0;
    for (e, sheep) in sheep.iter() {
        let mut in_safe = false;
        for safe_area in safe_areas.iter() {
            if safe_area.in_area(Vec2::new(sheep.translation.x, sheep.translation.z)) {
                in_safe = true;
            }
        }
        if in_safe {
            count += 1;
            commands.entity(e).remove::<OutOfSafeArea>();
        } else {
            commands.entity(e).insert(OutOfSafeArea);
        }
    }
    counter.count = count;
}
