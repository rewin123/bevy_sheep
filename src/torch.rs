use bevy::prelude::*;

use crate::{common_storage::CommonStorage, get_sprite_rotation};

const TORCH_PATH: &str = "test/torch.png";

pub struct TorchPlugin;

impl Plugin for TorchPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnTorch>()

            .add_systems(Startup, setup_material)
            .add_systems(Update, spawn_torch);
    }
}

#[derive(Component)]
pub struct Torch {
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
    asset_server : Res<AssetServer>,
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
    common_storage : Res<CommonStorage>,
    torch_material: Res<TorchMaterial>,

) {
    for event in events.read() {
        let torch = Torch {
            lit: false,
            fuel: 0.0,
            color: Color::ORANGE,
            max_fuel: 1.0,
            radius: 50.0,
        };

        commands.spawn((
            torch,
            PbrBundle {
                transform: Transform::from_translation(event.position + Vec3::new(0.0, 2.5, 0.0)).with_rotation(get_sprite_rotation()).with_scale(Vec3::new(1.0, 1.0, 7.0)),
                material: torch_material.0.clone(),
                mesh: common_storage.plane.clone(),
                ..default()
            }
        )).with_children(|parent| {
            parent.spawn(PointLightBundle {
                point_light: PointLight {
                    color: Color::ORANGE,
                    intensity: 10000.0,
                    range: 300.0,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 2.0, -0.4)),
                ..default()
            });
        });
    }
    events.clear();
}