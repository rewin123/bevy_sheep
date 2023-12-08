use bevy::prelude::*;

use crate::{safe_area::OutOfSafeArea, player::{DOG_SPEED, Bark}, test_level::LevelSize, common_storage::CommonStorage, get_sprite_rotation, physics::{Velocity, WalkController}, GameStuff};

const WOLF_SPEED: f32 = DOG_SPEED * 1.3;
const WOLF_ACCEL: f32 = WOLF_SPEED * 2.0;

pub struct WolfPlugin;

impl Plugin for WolfPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_wolf_material)
            .add_systems(
                Update,
                (
                    apply_deferred,
                    wolf_spawner,
                    catch_system,
                    eating_system,
                    go_out_system,
                    bark,
                    apply_deferred
                ).chain()
            );
    }
}

#[derive(Component)]
pub struct Wolf;

#[derive(Component)]
pub struct TryToCatchSheep {
    pub target: Entity
}

#[derive(Component)]
pub struct GoOut;

#[derive(Component)]
pub struct UnderHunting;

#[derive(Resource)]
pub struct WolfStorage {
    pub material : Handle<StandardMaterial>
}

#[derive(Component)]
pub struct Eating {
    pub time : f32
}

fn setup_wolf_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server : Res<AssetServer>,
) {
    commands.insert_resource(WolfStorage {
        material: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("test/wolf.png")),
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })
    })
}

fn wolf_spawner(
    mut commands: Commands,
    sheep: Query<(Entity, &Transform), (With<OutOfSafeArea>, Without<UnderHunting>)>,
    level_size : Res<LevelSize>,
    common_storage : Res<CommonStorage>,
    wolf_storage: Res<WolfStorage>
) {
    for (sheep_entity, sheep_transform) in sheep.iter() {
        commands
            .spawn((
                Wolf,
                PbrBundle {
                    mesh: common_storage.plane.clone(),
                    material: wolf_storage.material.clone(),
                    transform: Transform::from_translation(sheep_transform.translation.normalize() * level_size.0 * 2.0).with_rotation(get_sprite_rotation()),
                    ..default()
                },
                TryToCatchSheep {
                    target: sheep_entity
                },
                Velocity::default(),
                WalkController {
                    max_speed: WOLF_SPEED,
                    acceleration: WOLF_ACCEL,
                    target_velocity: Vec3::ZERO
                },
                GameStuff,
        ));

        commands.entity(sheep_entity).insert(UnderHunting);
    }
}

fn catch_system(
    mut commands: Commands,
    sheep : Query<&Transform>,
    mut wolfs : Query<(Entity, &Transform, &mut WalkController, &TryToCatchSheep)>
) {
    for (wolf, wolf_transform, mut walk_controller, try_to_catch_sheep) in wolfs.iter_mut() {
        let wolf_translation = wolf_transform.translation;
        if let Ok(sheep) = sheep.get(try_to_catch_sheep.target) {
            if wolf_translation.distance(sheep.translation) < 1.0 {
                commands.entity(wolf).insert(Eating {
                    time: 10.0
                }).remove::<TryToCatchSheep>();

                commands.entity(try_to_catch_sheep.target).despawn_recursive();
            } else {
                walk_controller.target_velocity = (sheep.translation - wolf_translation).normalize() * WOLF_SPEED;
                walk_controller.target_velocity = walk_controller.target_velocity.clamp_length_max((sheep.translation - wolf_translation).length() * 2.0);
            }
        }
    }
}

fn eating_system(
    mut commands: Commands,
    time: Res<Time>,
    mut wolfs : Query<(Entity, &mut Eating, &mut WalkController)>
) {
    for (wolf, mut eating, mut walk_controller) in wolfs.iter_mut() {
        eating.time -= time.delta_seconds();
        if eating.time <= 0.0 {
            commands.entity(wolf).remove::<Eating>()
                .insert(GoOut);
        } else {
            walk_controller.target_velocity = Vec3::ZERO;
        }
    }
}

fn go_out_system(
    mut commands: Commands,
    mut wolfs : Query<(Entity, &mut Transform, &mut WalkController, &GoOut)>,
    level_size : Res<LevelSize>
) {
    for (wolf, mut wolf_transform, mut walk_controller, go_out) in wolfs.iter_mut() {
        let dir = wolf_transform.translation.normalize();
        walk_controller.target_velocity = dir * WOLF_SPEED;

        if wolf_transform.translation.distance(Vec3::ZERO) > level_size.0 * 3.0 {
            commands.entity(wolf).despawn_recursive();
        }
    }
}

fn bark(
    mut commands: Commands,
    mut wolfs: Query<(Entity, &Transform), With<Wolf>>,
    mut barks : EventReader<Bark>,
) {
    let Some(bark) = barks.iter().next() else {
        return;
    };

    for (wolf, wolf_transform) in wolfs.iter_mut() {
        if wolf_transform.translation.distance(bark.position) < bark.radius {
            commands
                .entity(wolf)
                .insert(GoOut)
                .remove::<Eating>()
                .remove::<TryToCatchSheep>();
        }
    }
}