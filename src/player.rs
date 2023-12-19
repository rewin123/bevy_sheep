use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseWheel,
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder},
    prelude::*,
    window::PrimaryWindow, audio::{Volume, PlaybackMode},
};

use crate::{
    get_sprite_rotation,
    physics::Velocity,
    sprite_material::{create_plane_mesh, SpriteExtension, SpriteMaterial},
    GameStuff, GameSet, auto_anim::{AnimSet, AnimRange, AutoAnimPlugin, AutoAnim},
};

const DOG_PATH: &str = "test/dog.png";

pub const DOG_SPEED: f32 = 45.0 * 1000.0 / 3600.0; // Sheepdog accepts 35 km/h in reality (but fastest dog can do 67 km/h o.0)
pub const DOG_ACCELERATION: f32 = DOG_SPEED * 4.0;

pub const RUN_K: f32 = 2.0;
pub const STAMINA_INCREASE: f32 = 1.0 / 2.5;
pub const STAMINA_DECREASE: f32 = 1.0 / 5.0 + STAMINA_INCREASE;

pub const DOG_RUN_PATH: &str = "audio/running-in-grass.ogg";
pub const BARK_PATH: &str = "audio/barking.ogg";

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
            .add_systems(Update, spawn_player_by_event.in_set(GameSet::Playing))
            .add_systems(
                Update,
                player_movemnt_by_wasd.run_if(in_state(MovementStyle::WASD)),
            )
            .add_systems(
                Update,
                player_movemnt_by_mouse.run_if(in_state(MovementStyle::Mouse)),
            )
            .add_systems(Update, (change_movement_style, bark).in_set(GameSet::Playing))
            .add_systems(Update, (set_cam_distance, camera_movement, stamina_increse).in_set(GameSet::Playing))
            .add_plugins(AutoAnimPlugin::<PlayerAnim>::default())
            .add_systems(Update, set_anim_state.in_set(GameSet::Playing));
    }
}

#[derive(Component)]
pub struct Stamina {
    pub value: f32,
    pub blocked: bool
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

fn stamina_increse(
    mut stamina_query: Query<&mut Stamina>,
    time: Res<Time>,
) {
    for mut stamina in &mut stamina_query {
        stamina.value += STAMINA_INCREASE * time.delta_seconds();
        if stamina.value > 1.0 {
            stamina.value = 1.0;
            stamina.blocked = false;
        }
    }
}

#[derive(Default)]
pub enum PlayerAnim {
    BigBark,
    Bark,
    WalkAndBark,
    Walk,
    #[default]
    Idle
}

impl AnimSet for PlayerAnim {
    fn get_folder_path() -> String {
        "dog".to_string()
    }

    fn get_index_range(&self) -> crate::auto_anim::AnimRange {
        match self {
            PlayerAnim::BigBark => AnimRange::new(0, 2),
            PlayerAnim::Bark => AnimRange::new(3, 5),
            PlayerAnim::WalkAndBark => AnimRange::new(6,7),
            PlayerAnim::Walk => AnimRange::new(9, 11),
            PlayerAnim::Idle => AnimRange::new(12, 15),
        }
    }

    fn get_tile_count() -> usize {
        16
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

#[derive(Resource)]
pub struct DogSounds {
    pub bark: Handle<AudioSource>,
    pub run: Handle<AudioSource>,
}

#[derive(Component)]
pub struct DogBarkSource;

#[derive(Component)]
pub struct FootstepsSource;

fn spawn_player_by_event(
    mut commands: Commands,
    mut event_reader: EventReader<SpawnPlayer>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in event_reader.read() {
        let plane = meshes.add(create_plane_mesh());
        let material = materials.add(
            StandardMaterial {
                base_color_texture: Some(asset_server.load(DOG_PATH)),
                alpha_mode: AlphaMode::Opaque,
                ..default()
            });

        info!("Spawn player at {:?}", event.position);

        commands.spawn((
            PbrBundle {
                mesh: plane.clone(),
                material: material.clone(),
                transform: Transform::from_translation(event.position)
                    .with_rotation(get_sprite_rotation())
                    .with_scale(Vec3::new(1.0, 1.0, 1.0) * 2.0),
                ..default()
            },
            Player,
            Dog,
            Velocity::default(),
            GameStuff,
            Stamina {
                value: 1.0,
                blocked: false
            },
            AutoAnim {
                set: PlayerAnim::Idle,
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                current_frame: 0,
            }
        )).with_children(|parent| {
            parent.spawn((
                DogBarkSource,
                AudioBundle {
                    source: asset_server.load(BARK_PATH),
                    settings: PlaybackSettings {
                        paused: true,
                        mode: PlaybackMode::Loop,
                        volume: Volume::new_relative(0.5),
                        ..default()
                    }
                },
                SpatialBundle::default(),
            ));

            parent.spawn((
                FootstepsSource,
                AudioBundle {
                    source: asset_server.load(DOG_RUN_PATH),
                    settings: PlaybackSettings {
                        paused: true,
                        mode: PlaybackMode::Loop,
                        volume: Volume::new_relative(2.0),
                        ..default()
                    }
                }
            ));
        });
    }

    commands.insert_resource(DogSounds {
        bark: asset_server.load(BARK_PATH),
        run: asset_server.load(DOG_RUN_PATH),
    });

    event_reader.clear();
}

fn player_movemnt_by_mouse(
    mut player_query: Query<(&Transform, &mut Velocity, &mut Stamina), With<Player>>,
    time: Res<Time>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    input: Res<Input<KeyCode>>,
    mut footstep_source: Query<&mut AudioSink, With<FootstepsSource>>,
) {
    let Ok((transform, mut vel, mut stamine)) = player_query.get_single_mut() else {
        return;
    };

    let Ok(mut footstep) = footstep_source.get_single_mut() else {
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

    let mut use_stamina = input.pressed(KeyCode::ShiftLeft);
    if stamine.blocked {
        use_stamina = false;
    }

    if use_stamina {
        stamine.value -= time.delta_seconds() * STAMINA_DECREASE;
        if stamine.value < 0.0 {
            stamine.blocked = true;
        }
    }

    let speed_k = if use_stamina { 1.0 } else {RUN_K};

    let globel_cursor = ray.get_point(distance);

    let speed: f32 = DOG_SPEED * speed_k;
    let accel: f32 = DOG_ACCELERATION;

    let dir = (globel_cursor - transform.translation).normalize_or_zero();

    let max_speed = speed.min((globel_cursor - transform.translation).length() * 10.0);
    let target_speed = (dir * speed).clamp_length(0.0, max_speed);

    let dspeed = target_speed - vel.0;

    vel.0 += dspeed.normalize_or_zero() * accel * time.delta_seconds();

    vel.0 = vel.0.clamp_length_max(speed);

    if vel.0.length() > 1.0 {
        footstep.play();
    } else {
        footstep.pause();
    }
}

pub fn bark(
    player_query: Query<&Transform, With<Player>>,
    input: Res<Input<KeyCode>>,
    mut event_writer: EventWriter<Bark>,
    mut stamina : Query<&mut Stamina>,
    time : Res<Time>,
    bark_sink : Query<&AudioSink, With<DogBarkSource>>,
) {
    let Ok(bark) = player_query.get_single() else {
        return;
    };

    let Ok(mut stamina) = stamina.get_single_mut() else {
        return;
    };

    let mut radius = 10.;

    let mut play_bark = false;

    if input.pressed(KeyCode::ControlLeft) && !stamina.blocked {
        radius *= 1.5;
        stamina.value -= STAMINA_DECREASE * 3.0 * time.delta_seconds();

        if stamina.value < 0.0 {
            stamina.blocked = true;
        }

        event_writer.send(Bark {
            radius: radius,
            position: bark.translation,
        });
        play_bark = true;
    }

    if input.pressed(KeyCode::Space) {
        event_writer.send(Bark {
            radius: radius,
            position: bark.translation,
        });
        play_bark = true;
    }

    if play_bark {
        let Ok(bark) = bark_sink.get_single() else {
            warn!("Could not get bark source");
            return;
        };
        bark.play();
    } else {
        let Ok(bark) = bark_sink.get_single() else {
            warn!("Could not get bark source");
            return;
        };
        bark.pause();
    }
}

fn player_movemnt_by_wasd(
    mut player_query: Query<(&mut Velocity, &mut Stamina), With<Player>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut footstep_source: Query<&mut AudioSink, With<FootstepsSource>>
) {
    let Ok((mut player, mut stamina)) = player_query.get_single_mut() else {
        return;
    };

    let Ok(mut footstep) = footstep_source.get_single_mut() else {
        warn!("Could not get footstep source");
        return;
    };

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

    let mut use_stamina = input.pressed(KeyCode::ShiftLeft);

    if stamina.blocked {
        use_stamina = false;
    }

    if use_stamina {
        stamina.value -= time.delta_seconds() * STAMINA_DECREASE;
        if stamina.value < 0.0 {
            stamina.blocked = true;
        }
    }

    let speed = if use_stamina {
        DOG_SPEED * RUN_K
    } else {
        DOG_SPEED
    };

    dir = dir.normalize_or_zero();

    let target_speed = dir * speed;

    let dspeed = target_speed - player.0;

    let accel = accel.min(dspeed.length() * 100.0);

    player.0 += dspeed.normalize_or_zero() * accel * time.delta_seconds();

    player.0 = player.0.clamp_length_max(speed);

    if player.0.length() > 1.0 {
        footstep.play();
    } else {
        footstep.pause();
    }
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

fn set_anim_state(
    mut player : Query<(&mut AutoAnim<PlayerAnim>, &Velocity, &mut Transform)>,
    input: Res<Input<KeyCode>>
) {
    let Ok((mut player, vel, mut t)) = player.get_single_mut() else {
        return;
    };

    let moving = input.any_pressed([KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D]);
    let barking = input.pressed(KeyCode::Space);
    let big_bark = input.pressed(KeyCode::ControlLeft);

    if big_bark {
        player.set = PlayerAnim::BigBark;
    } else {
        if barking && moving {
            player.set = PlayerAnim::WalkAndBark;
        } else if barking {
            player.set = PlayerAnim::Bark;
        } else if moving {
            player.set = PlayerAnim::Walk;
        } else {
            player.set = PlayerAnim::Idle;
        }
    }

    if vel.0.x > 1.0 {
        t.rotation = get_sprite_rotation();
        t.rotate_local_z(PI);
    } else if vel.0.x < -1.0 {
        t.rotation = get_sprite_rotation();
    }
}