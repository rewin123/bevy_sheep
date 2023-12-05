use bevy::prelude::*;
use rand::prelude::*;

const TREE_PATH: &str = "test/tree.png";
const SHEEP_PATH: &str = "test/sheep.png";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    //spawn big grass texture

    commands.spawn(SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1000.0)),
        sprite: Sprite {
            custom_size: Some(Vec2::new(1000000.0, 1000000.0)),
            color: Color::rgb(0.0, 0.5, 0.0),
            ..default()
        },
        ..default()
    });

    let r = 300.0;

    let sheep_count = 2000;

    let mut rng = rand::thread_rng();
    for _ in 0..sheep_count {
        let random_pos = Vec2::new(rng.gen_range(-r..r), rng.gen_range(-r..r));

        if random_pos.length() > r {
            continue;
        }

        commands.spawn(SpriteBundle {
            texture: asset_server.load(SHEEP_PATH),
            transform: Transform::from_translation(random_pos.extend(-random_pos.y)),
            sprite: Sprite {
                custom_size: Some(Vec2::new(60.0, 60.0)),
                ..default()
            },
            ..default()
        });
    }

    //spawn trees

    let tree_count = 3000;
    let tree_r = 1000.0;
    let cut_r = r + 200.0;

    for _ in 0..tree_count {
        let random_pos = Vec2::new(
            rng.gen_range(-tree_r..tree_r),
            rng.gen_range(-tree_r..tree_r),
        );

        if random_pos.length() < cut_r {
            continue;
        }

        commands.spawn(SpriteBundle {
            texture: asset_server.load(TREE_PATH),
            transform: Transform::from_translation(random_pos.extend(-random_pos.y)),
            sprite: Sprite {
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            ..default()
        });
    }
}
