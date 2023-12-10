use bevy::prelude::*;

use crate::{
    common_storage::CommonStorage,
    get_sprite_rotation,
    global_task::torch_blinking::TorchDelight,
    safe_area::{HiddenSafeArea, SafeArea},
    GameSet, GameStuff,
};

const TORCH_PATH: &str = "test/torch.png";

pub const TORCH_ILLUMINATION: f32 = 10000.0;
pub const TORCH_BASE_RADIUS: f32 = 10.0;

pub struct TorchPlugin;

impl Plugin for TorchPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTorch>()
            .add_systems(Startup, setup_material)
            .add_systems(Update, spawn_torch)
            .add_event::<IgniteTorch>()
            .add_systems(Update, ignite_torch.in_set(GameSet::Playing));
    }
}

#[derive(Component)]
pub struct TorchLight;

#[derive(Component)]
pub struct TorchBase {
    pub light: Entity,
    pub lit: bool,
    pub fuel: f32,
    pub color: Color,
    pub max_fuel: f32,
    pub radius: f32,
}

#[derive(Event)]
pub struct SpawnTorch {
    pub position: Vec3,
}

#[derive(Resource)]
pub struct TorchMaterial(pub Handle<StandardMaterial>);

fn setup_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(TorchMaterial(materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(TORCH_PATH)),
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.01,
        ..default()
    })));
}

fn spawn_torch(
    mut commands: Commands,
    mut events: EventReader<SpawnTorch>,
    common_storage: Res<CommonStorage>,
    torch_material: Res<TorchMaterial>,
) {
    for event in events.read() {
        let torch_radius = TORCH_BASE_RADIUS;
        let smooth_radius = torch_radius + 0.01;
        let light_height = torch_radius * 0.5;

        let outer_spot_angle = ((torch_radius / light_height) as f32).atan();
        let inner_spot_angle = outer_spot_angle * 0.95;

        let light_id = commands
            .spawn(SpotLightBundle {
                spot_light: SpotLight {
                    color: Color::ORANGE,
                    intensity: 0.0,
                    radius: 0.0,
                    range: torch_radius * 10.0,
                    shadows_enabled: false,
                    inner_angle: inner_spot_angle,
                    outer_angle: outer_spot_angle,
                    ..default()
                },
                transform: Transform::from_translation(event.position + Vec3::Y * light_height)
                    .looking_at(event.position, Vec3::Z),
                ..default()
            })
            .insert(TorchLight)
            .insert(GameStuff)
            .id();

        let torch = TorchBase {
            lit: false,
            fuel: 0.0,
            color: Color::ORANGE,
            max_fuel: 1.0,
            radius: torch_radius,
            light: light_id,
        };

        commands.spawn((
            torch,
            PbrBundle {
                transform: Transform::from_translation(event.position)
                    .with_rotation(get_sprite_rotation())
                    .with_scale(Vec3::new(2.0 / 7.0, 2.0 / 7.0, 2.0)),
                material: torch_material.0.clone(),
                mesh: common_storage.plane.clone(),
                ..default()
            },
            GameStuff,
        ));
    }
    events.clear();
}

#[derive(Event)]
pub struct IgniteTorch {
    pub position: Vec3,
    pub radius: f32,
}

fn ignite_torch(
    mut commands: Commands,
    mut events: EventReader<IgniteTorch>,
    mut query: Query<(Entity, &mut TorchBase, &Transform)>,
    mut lights: Query<&mut SpotLight, With<TorchLight>>,
) {
    for event in events.read() {
        for (torch_e, mut torch, transform) in &mut query {
            if (transform.translation - event.position).length() < event.radius {
                torch.lit = true;
                torch.fuel = torch.max_fuel;
                if let Ok(mut light) = lights.get_mut(torch.light) {
                    light.intensity = TORCH_ILLUMINATION;
                    commands
                        .entity(torch_e)
                        .insert(SafeArea::Circle {
                            pos: Vec2::new(transform.translation.x, transform.translation.z),
                            radius: torch.radius,
                        })
                        .insert(HiddenSafeArea)
                        .remove::<TorchDelight>();

                    light.outer_angle = 2.0_f32.atan();
                    light.inner_angle = light.outer_angle * 0.95;
                    light.color = torch.color;
                };
            }
        }
    }
    events.clear();
}
