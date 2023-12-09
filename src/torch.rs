use bevy::prelude::*;

use crate::{common_storage::CommonStorage, get_sprite_rotation, GameStuff, GameSet};

const TORCH_PATH: &str = "test/torch.png";

const TORCH_ILLUMINATION: f32 = 1000.0;


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
    pub light : Entity,
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

        let light_id = commands.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::ORANGE,
                intensity: 0.0,
                range: 20.0,
                radius: 0.3,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        }).insert(TorchLight).id();

        let torch = TorchBase {
            lit: false,
            fuel: 0.0,
            color: Color::ORANGE,
            max_fuel: 1.0,
            radius: 20.0,
            light: light_id,
        };

        commands
            .spawn((
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
            )).add_child(light_id);
    }
    events.clear();
}

#[derive(Event)]
pub struct IgniteTorch {
    pub position: Vec3,
    pub radius: f32,
}

fn ignite_torch(
    mut events: EventReader<IgniteTorch>,
    mut query: Query<(&mut TorchBase, &Transform)>,
    mut lights : Query<&mut PointLight, With<TorchLight>>
) {
    for event in events.read() {
        for (mut torch, transform) in &mut query {
            if (transform.translation - event.position).length() < event.radius {
                torch.lit = true;
                torch.fuel = torch.max_fuel;
                if let Ok(mut light) = lights.get_mut(torch.light) {
                    light.intensity = TORCH_ILLUMINATION;
                };
            }
        }
    }
    events.clear();
}