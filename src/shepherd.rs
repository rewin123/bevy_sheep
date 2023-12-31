use bevy::prelude::*;

use crate::{
    common_storage::CommonStorage,
    get_sprite_rotation,
    global_task::torch_blinking::TorchDelight,
    physics::{Velocity, WalkController},
    player::{Bark, DOG_ACCELERATION, DOG_SPEED},
    sunday::DayState,
    torch::{IgniteTorch, TorchBase},
    GameSet, GameStuff, auto_anim::{AutoAnimPlugin, AutoAnim, AnimSet, AnimRange},
};

const SHEPHERD_PATH: &str = "test/Knight.png";

const SHEPHERD_SPEED: f32 = DOG_SPEED * 0.4;
const SHEPHERD_ACCEL: f32 = DOG_ACCELERATION * 0.4;

const IGNITE_RADIUS: f32 = 5.0;

pub struct ShepherdPlugin;

impl Plugin for ShepherdPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnShepherd>()
            .add_systems(
                Update,
                (spawn_shepherd_system, ignite_all_torhes, bark_system, set_anim).in_set(GameSet::Playing),
            )
            .add_systems(OnEnter(DayState::Evening), start_ignite_torches)
            .add_plugins(AutoAnimPlugin::<ShepherdAnim>::default());
    }
}

#[derive(Default)]
pub enum ShepherdAnim {
    #[default]
    Sleep,
    Walk
}

impl AnimSet for ShepherdAnim {
    fn get_folder_path() -> String {
        "shepherd".to_string()
    }

    fn get_index_range(&self) -> crate::auto_anim::AnimRange {
        match self {
            ShepherdAnim::Sleep => AnimRange::new(0, 19),
            ShepherdAnim::Walk => AnimRange::new(20,27),
        }
    }

    fn get_tile_count() -> usize {
        37
    }
}

#[derive(Event)]
pub struct SpawnShepherd {
    pub pos: Vec3,
}

#[derive(Component, Default)]
pub struct Shepherd;

#[derive(Component)]
pub struct IgniteAllTorhes;

fn start_ignite_torches(mut commands: Commands, query: Query<Entity, With<Shepherd>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).insert(IgniteAllTorhes);
    }
}

fn ignite_all_torhes(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WalkController, &Transform), With<IgniteAllTorhes>>,
    torches: Query<(&Transform, &TorchBase, Option<&TorchDelight>)>,
    mut ignite: EventWriter<IgniteTorch>,
) {
    let Ok((herd_entity, mut walk_controller, transform)) = query.get_single_mut() else {
        return;
    };

    //find nearest torch
    let mut nearest_torch: Option<Vec3> = None;
    let mut nearest_torch_data: Option<&TorchBase> = None;
    let mut dist = f32::MAX;
    for (torch_transform, torch, delight) in torches.iter() {
        let dist_to_torch = (torch_transform.translation - transform.translation).length();
        if dist_to_torch < dist && (!torch.lit || delight.is_some()) {
            nearest_torch = Some(torch_transform.translation);
            nearest_torch_data = Some(torch);
            dist = dist_to_torch;
        }
    }

    if let Some(nearest_torch) = nearest_torch {
        if dist < IGNITE_RADIUS {
            ignite.send(IgniteTorch {
                position: transform.translation,
                radius: IGNITE_RADIUS,
            });
        } else {
            walk_controller.target_velocity =
                (nearest_torch - transform.translation).normalize() * SHEPHERD_SPEED;
        }
    } else {
        commands.entity(herd_entity).remove::<IgniteAllTorhes>();
        walk_controller.target_velocity = Vec3::ZERO;
    }
}

fn spawn_shepherd_system(
    mut commands: Commands,
    mut events: EventReader<SpawnShepherd>,
    asset_server: Res<AssetServer>,
    common_storage: Res<CommonStorage>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        commands.spawn((
            Shepherd::default(),
            PbrBundle {
                transform: Transform::from_translation(event.pos)
                    .with_rotation(get_sprite_rotation())
                    .with_scale(Vec3::new(3.0, 3.0, 3.0)),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load(SHEPHERD_PATH)),
                    ..default()
                }),
                mesh: common_storage.plane.clone(),
                ..default()
            },
            Velocity::default(),
            WalkController {
                max_speed: SHEPHERD_SPEED,
                acceleration: SHEPHERD_ACCEL,
                target_velocity: Vec3::ZERO,
            },
            GameStuff,
            AutoAnim {
                set : ShepherdAnim::Sleep,
                current_frame: 0,
                timer: Timer::from_seconds(0.1, TimerMode::Repeating)
            }
        ));
    }
    events.clear();
}

fn bark_system(
    mut commands: Commands,
    mut events: EventReader<Bark>,
    query: Query<(Entity, &Transform), With<Shepherd>>,
) {
    let Ok((entity, transform)) = query.get_single() else {
        return;
    };
    for event in events.iter() {
        if (transform.translation - event.position).length() < event.radius {
            //wakeup shepherd
            commands.entity(entity).insert(IgniteAllTorhes);
        }
    }
}

fn set_anim(
    mut query : Query<(&mut AutoAnim<ShepherdAnim>, Option<&IgniteAllTorhes>)>
) {
    for (mut anim, ignite) in query.iter_mut() {
        if ignite.is_some() {
            anim.set = ShepherdAnim::Walk;
        } else {
            anim.set = ShepherdAnim::Sleep;
        }
    }
}