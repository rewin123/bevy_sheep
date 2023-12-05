use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use rand::prelude::*;
use std::f32::consts::PI;

const TREE_PATH: &str = "test/pine.png";
const SHEEP_PATH: &str = "test/sheep.png";

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 100.0, 100.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    //green plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 1000.0,
            ..default()
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.0, 0.5, 0.0),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    //spawn sun
    let mut cascades = CascadeShadowConfigBuilder::default();
    cascades.maximum_distance = 1000.0;
    cascades.minimum_distance = 1.0;
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            color: Color::WHITE,
            illuminance: 50000.0,
            ..default()
        },

        cascade_shadow_config: cascades.build(),
        ..default()
    });

    //ambient ligjt
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    let square = meshes.add(
        shape::Plane {
            size: 1.0,
            ..default()
        }
        .into(),
    );
    let sheep_texture: Handle<Image> = asset_server.load(SHEEP_PATH);
    let tree_texture: Handle<Image> = asset_server.load(TREE_PATH);

    let sheep_material = materials.add(StandardMaterial {
        base_color_texture: Some(sheep_texture),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    //spawn ships
    let r = 50.0;
    let mut rng = rand::thread_rng();
    let sheep_count = 2000;

    for _ in 0..sheep_count {
        let x = rng.gen_range(-r..r);
        let y = 0.0;
        let z = rng.gen_range(-r..r);

        let pos = Vec3::new(x, y, z);
        if pos.length() > r {
            continue;
        }

        commands.spawn(PbrBundle {
            mesh: square.clone(),
            material: sheep_material.clone(),
            transform: Transform::from_xyz(pos.x, pos.y + 3.0, pos.z)
                .with_rotation(Quat::from_euler(EulerRot::XYZ, PI / 2.0 - PI / 4.0, 0.0, 0.0))
                .with_scale(Vec3::splat(10.0)),
            ..default()
        });
    }

    //spawn trees
    let tree_count = 5000;
    let tree_material = materials.add(StandardMaterial {
        base_color_texture: Some(tree_texture),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.1,
        ..default()
    });

    let tree_r = 150.0;
    let cut_r = r + 20.0;

    for _ in 0..tree_count {
        let x = rng.gen_range(-tree_r..tree_r);
        let y = 0.0;
        let z = rng.gen_range(-tree_r..tree_r);

        let pos = Vec3::new(x, y, z);
        if pos.length() < cut_r {
            continue;
        }

        commands.spawn(PbrBundle {
            mesh: square.clone(),
            material: tree_material.clone(),
            transform: Transform::from_xyz(pos.x, pos.y + 8.0, pos.z)
                .with_rotation(Quat::from_euler(EulerRot::XYZ, PI / 2.0, 0.0, 0.0))
                .with_scale(Vec3::new(10.0, 10.0, 20.0)),
            ..default()
        });
    }
}
