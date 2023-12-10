use bevy::{
    input::mouse::MouseWheel,
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    get_sprite_rotation,
    physics::Velocity,
    sprite_material::{create_plane_mesh, SpriteExtension, SpriteMaterial},
    GameStuff,
};

const DOG_PATH: &str = "test/dog.png";

pub const DOG_SPEED: f32 = 45.0 * 1000.0 / 3600.0; // Sheepdog accepts 35 km/h in reality (but fastest dog can do 67 km/h o.0)
pub const DOG_ACCELERATION: f32 = DOG_SPEED * 4.0;

pub struct PlayerPlugin;

#[derive(Default, Hash, PartialEq, Eq, Debug, States, Clone)]
pub enum MovementStyle {
    Mouse,
    #[default]
    WASD,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPlayer>()
            .add_event::<Bark>()
            .add_state::<MovementStyle>()
            .add_systems(Update, spawn_player_by_event)
            .add_systems(
                Update,
                player_movemnt_by_wasd.run_if(in_state(MovementStyle::WASD)),
            )
            .add_systems(
                Update,
                player_movemnt_by_mouse.run_if(in_state(MovementStyle::Mouse)),
            )
            .add_systems(Update, (change_movement_style, bark))
            .add_systems(Update, (set_cam_distance, camera_movement));
    }
}

fn change_movement_style(
    mut next_state: ResMut<NextState<MovementStyle>>,
    current_state: Res<State<MovementStyle>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        if *current_state.get() == MovementStyle::Mouse {
            next_state.set(MovementStyle::WASD);
        } else {
            next_state.set(MovementStyle::Mouse);
        }
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct CameraDistance(f32);

#[derive(Component)]
pub struct Dog;

#[derive(Event)]
pub struct SpawnPlayer {
    pub position: Vec3,
}

#[derive(Event)]
pub struct Bark {
    pub radius: f32,
    pub position: Vec3,
}

fn spawn_player_by_event(
    mut commands: Commands,
    mut event_reader: EventReader<SpawnPlayer>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    mut sprite_material: ResMut<Assets<SpriteMaterial>>,
) {
    for event in event_reader.read() {
        let plane = meshes.add(create_plane_mesh());
        let material = sprite_material.add(SpriteMaterial {
            base: StandardMaterial {
                base_color_texture: Some(asset_server.load(DOG_PATH)),
                alpha_mode: AlphaMode::Opaque,
                ..default()
            },
            extension: SpriteExtension {
                base_teture: Some(asset_server.load(DOG_PATH)),
            },
        });

        info!("Spawn player at {:?}", event.position);

        commands.spawn((
            MaterialMeshBundle {
                mesh: plane.clone(),
                material: material.clone(),
                transform: Transform::from_translation(event.position)
                    .with_rotation(get_sprite_rotation())
                    .with_scale(Vec3::new(12.0 / 8.0, 1.0, 1.0)),
                ..default()
            },
            Player,
            Dog,
            Velocity::default(),
            GameStuff,
        ));
    }
    event_reader.clear();
}

fn player_movemnt_by_mouse(
    mut player_query: Query<(&Transform, &mut Velocity), With<Player>>,
    time: Res<Time>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok((transform, mut vel)) = player_query.get_single_mut() else {
        return;
    };

    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::Y * transform.translation.y, Vec3::Y) else {
        return;
    };

    let globel_cursor = ray.get_point(distance);

    let speed: f32 = DOG_SPEED;
    let accel: f32 = DOG_ACCELERATION;

    let dir = (globel_cursor - transform.translation).normalize_or_zero();

    let max_speed = speed.min((globel_cursor - transform.translation).length() * 10.0);
    let target_speed = (dir * speed).clamp_length(0.0, max_speed);

    let dspeed = target_speed - vel.0;

    vel.0 += dspeed.normalize_or_zero() * accel * time.delta_seconds();

    vel.0 = vel.0.clamp_length_max(speed);
}

pub fn bark(
    player_query: Query<&Transform, With<Player>>,
    input: Res<Input<KeyCode>>,
    mut event_writer: EventWriter<Bark>,
) {
    let Ok(bark) = player_query.get_single() else {
        return;
    };

    if input.pressed(KeyCode::Space) {
        event_writer.send(Bark {
            radius: 10.,
            position: bark.translation,
        });
    }
}

fn player_movemnt_by_wasd(
    mut player_query: Query<&mut Velocity, With<Player>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut player) = player_query.get_single_mut() else {
        return;
    };

    let speed = DOG_SPEED;
    let accel = DOG_ACCELERATION;

    let mut dir = Vec3::ZERO;

    if input.pressed(KeyCode::W) {
        dir += Vec3::new(0.0, 0.0, -1.0);
    }

    if input.pressed(KeyCode::S) {
        dir += Vec3::new(0.0, 0.0, 1.0);
    }

    if input.pressed(KeyCode::A) {
        dir += Vec3::new(-1.0, 0.0, 0.0);
    }

    if input.pressed(KeyCode::D) {
        dir += Vec3::new(1.0, 0.0, 0.0);
    }

    dir = dir.normalize_or_zero();

    let target_speed = dir * speed;

    let dspeed = target_speed - player.0;

    let accel = accel.min(dspeed.length() * 100.0);

    player.0 += dspeed.normalize_or_zero() * accel * time.delta_seconds();

    player.0 = player.0.clamp_length_max(speed);
}

fn camera_movement(
    mut camera_query: Query<(&mut Transform, &mut CameraDistance), With<Camera>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    time: Res<Time>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut sun: Query<&mut CascadeShadowConfig>,
) {
    let Ok((mut camera, mut distance)) = camera_query.get_single_mut() else {
        return;
    };
    let Ok(player) = player_query.get_single() else {
        return;
    };

    let Ok(mut sun) = sun.get_single_mut() else {
        return;
    };

    for ev in scroll_evr.read() {
        let delta = ev.y;
        if delta < 0.0 {
            distance.0 *= 1.1;
        } else if delta > 0.0 {
            distance.0 /= 1.1;
        }

        distance.0 = distance.0.clamp(10.0, 150.0);

        let mut cascade = CascadeShadowConfigBuilder::default();
        cascade.maximum_distance = distance.0 * 2.0;
        *sun = cascade.build();
    }

    let cam_frw = camera.forward();
    let next_cam_pos = player.translation - cam_frw * distance.0;

    let dp = next_cam_pos - camera.translation;
    let dt = time.delta_seconds();

    camera.translation += dp * dt * 1.5 * (150.0 / distance.0);
}

fn set_cam_distance(
    mut commands: Commands,
    camera_without_dist: Query<(Entity, &Transform), (With<Camera>, Without<CameraDistance>)>,
    player_query: Query<&Transform, With<Player>>,
    _: Query<&CascadeShadowConfig>,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    let Ok((e, camera)) = camera_without_dist.get_single() else {
        return;
    };

    let dist = (player.translation - camera.translation).dot(camera.forward());

    commands.entity(e).insert(CameraDistance(dist));
}
