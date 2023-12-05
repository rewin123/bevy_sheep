use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

const SHEEP_PATH: &str = "test/sheep.png";

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let square = meshes.add(
        shape::Plane {
            size: 1.0,
            ..default()
        }
        .into(),
    );
    let sheep_texture: Handle<Image> = asset_server.load(SHEEP_PATH);

    let sheep_material = materials.add(StandardMaterial {
        base_color_texture: Some(sheep_texture),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    //spawn sheeps
    let r = 50.0;
    let mut rng = rand::thread_rng();
    let sheep_count = 10;

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
                .with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    PI / 2.0 - PI / 4.0,
                    0.0,
                    0.0,
                ))
                .with_scale(Vec3::splat(10.0)),
            ..default()
        });
    }
}
