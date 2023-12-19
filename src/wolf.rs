use bevy::{prelude::*, audio::Volume};
use rand::Rng;

use crate::{
    common_storage::CommonStorage,
    get_sprite_rotation,
    physics::{Velocity, WalkController},
    player::{Bark, DOG_SPEED},
    safe_area::{OutOfSafeArea, SafeArea},
    test_level::LevelSize,
    GameStuff, auto_anim::{AnimSet, AnimRange, AutoAnimPlugin, AutoAnim}, corpse::SpawnCorpse,
};

const WOLF_SPEED: f32 = DOG_SPEED * 1.3;
const WOLF_ACCEL: f32 = WOLF_SPEED * 2.0;

pub struct WolfPlugin;

impl Plugin for WolfPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_wolf_material).add_systems(
            Update,
            (
                apply_deferred,
                wolf_spawner,
                catch_system,
                apply_deferred,
                eating_system,
                apply_deferred,
                go_out_system,
                apply_deferred,
                run_out_system,
                bark,
                apply_deferred,
            )
                .chain(),
        ).add_plugins(AutoAnimPlugin::<WolfAnim>::default());
    }
}

#[derive(Default)]
pub enum WolfAnim {
    Eat,
    #[default]
    Run
}

impl AnimSet for WolfAnim {
    fn get_folder_path() -> String {
        "wolf".to_string()
    }

    fn get_index_range(&self) -> crate::auto_anim::AnimRange {
        match self {
            WolfAnim::Eat => AnimRange::new(0, 5),
            WolfAnim::Run => AnimRange::new(6, 11),
        }
    }

    fn get_tile_count() -> usize {
        20
    }
}

#[derive(Component)]
pub struct Wolf;

#[derive(Component)]
pub struct TryToCatchSheep {
    pub target: Entity,
    pub ignore_safe : bool
}

#[derive(Component)]
pub struct GoOut;

#[derive(Component)]
pub struct UnderHunting;

#[derive(Resource)]
pub struct WolfStorage {
    pub material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct Eating {
    pub time: f32,
}

fn setup_wolf_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(WolfStorage {
        material: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("test/wolf.png")),
            alpha_mode: AlphaMode::Opaque,
            ..default()
        }),
    })
}

fn wolf_spawner(
    mut commands: Commands,
    sheep: Query<(Entity, &Transform), (With<OutOfSafeArea>, Without<UnderHunting>)>,
    level_size: Res<LevelSize>,
    common_storage: Res<CommonStorage>,
    wolf_storage: Res<WolfStorage>,
    wolfs : Query<(), With<Wolf>>,
    safe_areas : Query<&SafeArea>,
) {
    let num_wolfs = wolfs.iter().count();
    if num_wolfs > 20 {
        return;
    }
    for (sheep_entity, sheep_transform) in sheep.iter() {

        let start_pos = sheep_transform.translation.normalize() * level_size.0 * 2.0;
        let dir = (sheep_transform.translation - start_pos).normalize();
        let t_to_ship = (sheep_transform.translation - start_pos).length();
        let mut has_cross_to_safe = false;
        for safe_area in safe_areas.iter() {
            if let SafeArea::Circle { pos, radius } = safe_area {
                let t = (-start_pos.x * dir.x - start_pos.y * dir.y + pos.x * dir.x + pos.y * dir.y) / (dir.x * dir.x + dir.y * dir.y);
                let nearest_pos = start_pos + dir * t;
                if (Vec2::new(nearest_pos.x, nearest_pos.z) - *pos).length() < *radius && t > 0.0 && t < t_to_ship {
                    has_cross_to_safe = true;
                    break;
                }
            }
        }

        if has_cross_to_safe {
            continue;
        }

        commands.spawn((
            Wolf,
            PbrBundle {
                mesh: common_storage.plane.clone(),
                material: wolf_storage.material.clone(),
                transform: Transform::from_translation(
                    sheep_transform.translation.normalize() * level_size.0 * 2.0,
                )
                .with_rotation(get_sprite_rotation())
                .with_scale(Vec3::new(1.0, 1.0, 1.0) * 3.0),
                ..default()
            },
            TryToCatchSheep {
                target: sheep_entity,
                ignore_safe : false
            },
            Velocity::default(),
            WalkController {
                max_speed: WOLF_SPEED,
                acceleration: WOLF_ACCEL,
                target_velocity: Vec3::ZERO,
            },
            GameStuff,
            AutoAnim {
                set: WolfAnim::Run,
                current_frame: 0,
                timer: Timer::from_seconds(0.1 + rand::thread_rng().gen_range(-0.01..=0.01), TimerMode::Repeating),
            }
        ));

        commands.entity(sheep_entity).insert(UnderHunting);
    }
}

#[derive(Component)]
pub struct SheepDying;

fn catch_system(
    mut commands: Commands,
    sheep: Query<&Transform>,
    mut wolfs: Query<(Entity, &Transform, &mut WalkController, &TryToCatchSheep)>,
    asset_server : Res<AssetServer>,
    mut spawn_corpse : EventWriter<SpawnCorpse>,
    sheep_dying : Query<(), With<SheepDying>>
) {
    let mut sheep_dying_count = sheep_dying.iter().count();
    for (wolf, wolf_transform, mut walk_controller, try_to_catch_sheep) in wolfs.iter_mut() {
        let wolf_translation = wolf_transform.translation;
        if let Ok(sheep) = sheep.get(try_to_catch_sheep.target) {
            if wolf_translation.distance(sheep.translation) < 1.0 {
                commands
                    .entity(wolf)
                    .insert(Eating { time: 2.0 })
                    .remove::<TryToCatchSheep>()
                    .insert(AutoAnim {
                        set: WolfAnim::Eat,
                        current_frame: 0,
                        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                    });

                commands
                    .entity(try_to_catch_sheep.target)
                    .despawn_recursive();

                spawn_corpse.send(SpawnCorpse { position: sheep.translation });

                if sheep_dying_count < 3 {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("audio/kill_sound.ogg"),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: Volume::new_relative(0.7),
                            spatial: true,
                            ..default()
                        },
                    }).insert(SheepDying);
                    sheep_dying_count += 3;
                }
            } else {
                walk_controller.target_velocity =
                    (sheep.translation - wolf_translation).normalize() * WOLF_SPEED;
                walk_controller.target_velocity = walk_controller
                    .target_velocity
                    .clamp_length_max((sheep.translation - wolf_translation).length() * 2.0);
            }
        }
    }
}

fn eating_system(
    mut commands: Commands,
    time: Res<Time>,
    mut wolfs: Query<(Entity, &Transform, &mut Eating, &mut WalkController)>,
    unhunted_sheep: Query<(Entity, &Transform), (With<OutOfSafeArea>, Without<UnderHunting>)>,
) {
    for (wolf, wolf_transform, mut eating, mut walk_controller) in wolfs.iter_mut() {
        eating.time -= time.delta_seconds();
        if eating.time <= 0.0 {
            //test if we can eat more
            let mut nearest : Option<(Entity, f32)> = None;
            for (sheep_e, sheep_transform) in unhunted_sheep.iter() {
                let dist = (wolf_transform.translation - sheep_transform.translation).length();
                if nearest.is_none() || dist < nearest.unwrap().1 {
                    nearest = Some((sheep_e, dist));
                }
            }

            if let Some((e, _)) = nearest {
                commands.entity(wolf).remove::<Eating>().insert(TryToCatchSheep {
                    target: e,
                    ignore_safe: false
                });

                commands.entity(e).insert(UnderHunting);
            } else {
                commands.entity(wolf).remove::<Eating>().insert(GoOut)
                    .insert(AutoAnim {
                        set: WolfAnim::Run,
                        current_frame: 0,
                        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                    });
            }
        } else {
            walk_controller.target_velocity = Vec3::ZERO;
        }
    }
}

fn go_out_system(
    mut commands: Commands,
    mut wolfs: Query<(Entity, &Transform, &mut WalkController, &GoOut)>,
    safearea: Query<&SafeArea>,
    level_size: Res<LevelSize>,
) {
    for (wolf, wolf_transform, mut walk_controller, _go_out) in wolfs.iter_mut() {
        let dir = wolf_transform.translation.normalize();
        walk_controller.target_velocity = dir * WOLF_SPEED;

        if wolf_transform.translation.distance(Vec3::ZERO) > level_size.0 * 3.0 {
            commands.entity(wolf).despawn_recursive();
        }
        if safearea.iter().any(|area| {
            area.in_area(Vec2 {
                x: wolf_transform.translation.x,
                y: wolf_transform.translation.z,
            })
        }) {
            commands.entity(wolf).insert(GoOut).remove::<Eating>()
                .insert(AutoAnim {
                    set: WolfAnim::Run,
                    current_frame: 0,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                });
        }
    }
}

fn run_out_system(
    mut commands: Commands,
    mut wolfs: Query<
        (
            Entity,
            &Transform,
            &mut WalkController,
            Option<&TryToCatchSheep>,
        ),
        (With<Wolf>, Without<GoOut>),
    >,
    safearea: Query<&SafeArea>,
) {
    for (wolf, wolf_transform, mut walk_controller, catch) in wolfs.iter_mut() {
        if let Some(catch) = catch {
            if catch.ignore_safe {
                continue;
            }
        }

        let in_safe_area = safearea.iter().filter(|area| {
            area.in_area(Vec2 {
                x: wolf_transform.translation.x,
                y: wolf_transform.translation.z,
            })
        });
        if let Some(area) = in_safe_area.last() {
            walk_controller.target_velocity =
                (wolf_transform.translation - area.get_center()).normalize() * WOLF_SPEED;
            commands
                .entity(wolf)
                .insert(GoOut)
                .remove::<TryToCatchSheep>()
                .remove::<Eating>()
                .insert(AutoAnim {
                    set: WolfAnim::Run,
                    current_frame: 0,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                });

            if let Some(catch) = catch {
                commands.entity(catch.target).remove::<UnderHunting>();
            }
        }
    }
}

fn bark(
    mut commands: Commands,
    mut wolfs: Query<(Entity, &Transform, Option<&TryToCatchSheep>), With<Wolf>>,
    mut barks: EventReader<Bark>,
) {
    let Some(bark) = barks.read().next() else {
        return;
    };

    for (wolf, wolf_transform, catch) in wolfs.iter_mut() {
        if wolf_transform.translation.distance(bark.position) < bark.radius {
            commands
                .entity(wolf)
                .insert(GoOut)
                .remove::<Eating>()
                .remove::<TryToCatchSheep>()
                .insert(AutoAnim {
                    set: WolfAnim::Run,
                    current_frame: 0,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                });

            if let Some(catch) = catch {
                commands.entity(catch.target).remove::<UnderHunting>();
            }
        }
    }
}
