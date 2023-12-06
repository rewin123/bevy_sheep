//just for testing. Gizmo works very bad with AlphaMode::Blend. Its alternative shader with discard on depth step

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{render_resource::{AsBindGroup, ShaderRef, PrimitiveTopology}, mesh::Indices},
};


pub fn create_plane_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList)
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, 
    vec![
        [-0.5, 0.0, 0.0],
        [0.5, 0.0, 0.0],
        [-0.5, 0.0, 1.0],
        [0.5, 0.0, 1.0],
    ])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, 
        vec![
            [0.0, 1.0],
            [1.0, 1.0],
            [0.0, 0.0],
            [1.0, 0.0],
        ]
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, 
        vec![
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
        ]
    )
    .with_indices(Some(
        Indices::U32(vec![
            0, 1, 2,
            2, 1, 3
        ])
    ))
}

pub type SpriteMaterial = ExtendedMaterial<StandardMaterial, SpriteExtension>;

pub struct SpriteMaterialPlugin;

impl Plugin for SpriteMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, SpriteExtension>,
        >::default());
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct SpriteExtension {
    #[texture(101)]
    #[sampler(102)]
    pub base_teture: Option<Handle<Image>>,
}

impl MaterialExtension for SpriteExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/sprite.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/sprite.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "shaders/sprite_prepass.wgsl".into()
    }
}
