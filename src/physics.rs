use bevy::prelude::*;

use crate::GameSet;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (walk_system, apply_velocity).chain().in_set(GameSet::Playing));
    }
}

#[derive(Component, Default, Reflect)]
pub struct Velocity(pub Vec3);

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

#[derive(Component)]
pub struct WalkController {
    pub target_velocity: Vec3,
    pub acceleration: f32,
    pub max_speed: f32,
}

fn walk_system(time: Res<Time>, mut query: Query<(&mut Velocity, &mut WalkController)>) {
    for (mut velocity, controller) in query.iter_mut() {
        let dspeed = controller.target_velocity - velocity.0;
        let accel = controller.acceleration.min(dspeed.length() * 100.0);

        velocity.0 += dspeed.normalize_or_zero() * accel * time.delta_seconds();
        velocity.0 = velocity.0.clamp_length_max(controller.max_speed);
    }
}
