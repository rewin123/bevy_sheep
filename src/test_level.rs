use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use rand::prelude::*;
use std::f32::consts::PI;

use crate::{
    get_sprite_rotation,
    level_ui::CreateLevelUi,
    player::SpawnPlayer,
    safe_area::{LandSafeArea, SafeArea},
    shepherd::SpawnShepherd,
    sprite_material::create_plane_mesh,
    sunday::{AMBIENT_BASE_ILLUMINANCE, DAY_SUN_COLOR, SUN_BASE_ILLUMINANCE},
    torch::SpawnTorch,
    GameStuff,
};

const TREE_PATH: &str = "test/pine.png";

#[derive(Clone, Resource)]
pub struct LevelSize(pub f32);

impl Default for LevelSize {
    fn default() -> Self {
        Self(50.)
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut spawn_player_event: EventWriter<SpawnPlayer>,
    mut spawn_torch: EventWriter<SpawnTorch>,
    level_size: Res<LevelSize>,
    mut create_level_ui: EventWriter<CreateLevelUi>,
    mut spawn_shepherd: EventWriter<SpawnShepherd>,
) {
    //spawn sun
    let mut cascades = CascadeShadowConfigBuilder::default();
    cascades.maximum_distance = 100.0;
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(30.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            color: Color::hex(DAY_SUN_COLOR).unwrap(),
            illuminance: SUN_BASE_ILLUMINANCE,
            ..default()
        },

        cascade_shadow_config: cascades.build(),
        ..default()
    });

    //ambient ligjt
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: AMBIENT_BASE_ILLUMINANCE,
    });

    let square = meshes.add(create_plane_mesh());
    let tree_texture: Handle<Image> = asset_server.load(TREE_PATH);

    let r = level_size.0;
    let mut rng = rand::thread_rng();

    //spawn trees
    let tree_material = materials.add(StandardMaterial {
        base_color_texture: Some(tree_texture),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.9,
        ..default()
    });

    let tree_r = r * 2.0;
    let cut_r = r + 5.0;

    let tree_area_size = PI * tree_r * tree_r - PI * cut_r * cut_r;
    let tree_per_meter = 0.5;
    let tree_count = (tree_area_size * tree_per_meter) as usize;

    for _ in 0..tree_count {
        let x = rng.gen_range(-tree_r..tree_r);
        let y = 0.0;
        let z = rng.gen_range(-tree_r..tree_r);

        let pos = Vec3::new(x, y, z);
        if pos.length() < cut_r {
            continue;
        }

        commands
            .spawn(PbrBundle {
                mesh: square.clone(),
                material: tree_material.clone(),
                transform: Transform::from_xyz(pos.x, pos.y, pos.z)
                    .with_rotation(get_sprite_rotation())
                    .with_scale(Vec3::new(2.5, 2.6, 5.0)),
                ..default()
            })
            .insert(GameStuff);
    }

    //green plane
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: tree_r * 2.0,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("5d9669").unwrap(),
                reflectance: 0.05,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(GameStuff);

    spawn_player_event.send(SpawnPlayer {
        position: Vec3::new(-r - 2.0, 0.0, 0.0),
    });

    let num_of_torchs = 20;
    let torch_r = r / 2.0;
    for _ in 0..num_of_torchs {
        let pos = Vec3::new(
            rng.gen_range(-torch_r..torch_r),
            0.0,
            rng.gen_range(-torch_r..torch_r),
        );

        spawn_torch.send(SpawnTorch { position: pos });
    }

    let safe_area = SafeArea::Rect {
        pos: Vec2::ZERO,
        size: Vec2::new(r * 1.5, r * 1.5),
    };
    commands
        .spawn(safe_area.clone())
        .insert(LandSafeArea {
            start_area: safe_area.clone(),
        })
        .insert(GameStuff);

    spawn_shepherd.send(SpawnShepherd {
        pos: Vec3::new(0.0, 0.0, -level_size.0),
    });

    create_level_ui.send(CreateLevelUi);
}
