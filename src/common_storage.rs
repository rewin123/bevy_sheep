use bevy::prelude::*;

#[derive(Resource)]
pub struct CommonStorage {
    pub plane : Handle<Mesh>
}

pub struct CommonStoragePlugin;

impl Plugin for CommonStoragePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_common_storage);
    }
}

pub fn init_common_storage(
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
) {
    let storage = CommonStorage {
        plane : meshes.add(Mesh::from(shape::Plane { size: 1.0, ..default() }))
    };

    commands.insert_resource(storage);
}