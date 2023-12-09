//safe area description and logic

use std::f32::consts::PI;

use bevy::{prelude::*, transform::commands};
use rand::Rng;

use crate::{sheep::Sheep, storyteller::Storyteller};

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
    Rect { pos: Vec2, size: Vec2},
    Ellipse { pos1: Vec2, pos2: Vec2, radius: f32},
    Circle { pos: Vec2, radius: f32},
}

#[derive(Component)]
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
            SafeArea::Ellipse {
                pos1,
                pos2,
                radius: _,
            } => {
                let d = (*pos1 - *pos2).length();
                let r = (*pos1 - sheep_pos).length();
                r * r <= d * d
            }
            SafeArea::Circle { pos, radius } => {
                (*pos - sheep_pos).length() < *radius
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
            SafeArea::Ellipse {
                pos1,
                pos2,
                radius: _,
            } => Vec3::new((pos1.x + pos2.x) / 2.0, 0.0, (pos1.y + pos2.y) / 2.0),
            SafeArea::Circle { pos, radius } => {
                Vec3::new(pos.x, 0.0, pos.y)
            }
        }
    }

    pub fn get_scaled(&self, scale: f32) -> SafeArea {
        match self {
            SafeArea::Rect { pos, size } => {
                SafeArea::Rect {
                    pos: *pos,
                    size: *size * scale,
                }
            }
            SafeArea::Ellipse { pos1, pos2, radius } => {
                SafeArea::Ellipse {
                    pos1: *pos1,
                    pos2: *pos2,
                    radius: *radius * scale,
                }
            },
            SafeArea::Circle { pos, radius } => {
                SafeArea::Circle {
                    pos: *pos,
                    radius: *radius * scale,
                }
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
                    Color::ORANGE,
                );
            }
            SafeArea::Ellipse {
                pos1: _,
                pos2: _,
                radius: _,
            } => {}
            SafeArea::Circle { pos, radius } => {
                gizmos.circle(Vec3::new(pos.x, 0.001, pos.y), Vec3::Y, *radius, Color::ORANGE);
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