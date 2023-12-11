use bevy::prelude::*;
use rand::Rng;

use crate::{GameSet, GameStuff};

pub struct CorpsePlugin;

impl Plugin for CorpsePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (corpse_system, spawn_corpse_system).in_set(GameSet::Playing))
            .add_event::<SpawnCorpse>()
            .add_systems(Startup, setup_corpse_storage);
    }
}

#[derive(Component)]
pub struct Corpse {
    pub time : f32
}

fn corpse_system(
    mut commands : Commands,
    time : Res<Time>,
    mut query : Query<(Entity, &mut Corpse)>
) {
    for (entity, mut corpse) in query.iter_mut() {
        corpse.time -= time.delta_seconds();
        if corpse.time <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Resource)]
pub struct CorpseStoage {
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

fn setup_corpse_storage(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut meshs : ResMut<Assets<Mesh>>
) {
    commands.insert_resource(CorpseStoage {
        material: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("sheep/tile001.png")),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.1,
            ..default()
        }),
        mesh: meshs.add(shape::Plane::default().into())
    });
}

#[derive(Event)]
pub struct SpawnCorpse {
    pub position : Vec3
}

fn spawn_corpse_system(
    mut commands : Commands,
    mut event : EventReader<SpawnCorpse>,
    storage : Res<CorpseStoage>
) {
    for event in event.iter() {
        commands.spawn((
            PbrBundle {
                mesh: storage.mesh.clone(),
                material: storage.material.clone(),
                transform: Transform::from_translation(event.position + Vec3::Y * rand::thread_rng().gen_range(0.01..=0.02)).with_scale(Vec3::new(2.0, 2.0, 2.0)),
                ..default()
            },
            Corpse {
                time: 20.0
            },
            GameStuff
        ));
    }
    event.clear();
}