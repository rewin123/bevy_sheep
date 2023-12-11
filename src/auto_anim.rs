use bevy::{prelude::*, render::render_resource::Face};

use crate::GameSet;


pub trait AnimSet {
    fn get_folder_path() -> String;
    fn get_index_range(&self) -> AnimRange;
    fn get_tile_count() -> usize;

    fn get_tile_name(idx : usize) -> String {
        //write in format tileXXX.png with exactly 3 digits
        format!("tile{:03}.png", idx)
    }

    fn get_tile_path(idx : usize) -> String {
        format!("{}/{}", Self::get_folder_path(), Self::get_tile_name(idx))
    }
}

#[derive(Default)]
pub struct AutoAnimPlugin<T : AnimSet> {
    _phantom : std::marker::PhantomData<T>
}

impl<T : AnimSet + Send + Sync + 'static> Plugin for AutoAnimPlugin<T> {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init_storage::<T>)
            .add_systems(Update, auto_anim::<T>.in_set(GameSet::Playing));
    }
}

fn auto_anim<T : AnimSet + Send + Sync + 'static>(
    mut commands : Commands,
    mut auto_anim_query : Query<(Entity, &mut AutoAnim<T>)>,
    time : Res<Time>,
    mut materials : ResMut<MaterialStorage<T>>
) {
    for (entity, mut auto_anim) in auto_anim_query.iter_mut() {
        if auto_anim.timer.tick(time.delta()).just_finished() {
            auto_anim.current_frame = (auto_anim.current_frame + 1) % (auto_anim.set.get_index_range().end + 1 - auto_anim.set.get_index_range().start);
            commands.entity(entity).insert(materials.materials[
                auto_anim.set.get_index_range().start + auto_anim.current_frame
            ].clone());
        }
    }
}

fn init_storage<T : AnimSet + Send + Sync + 'static>(
    mut commands : Commands,
    asset_server : Res<AssetServer>,
    mut materials : ResMut<Assets<StandardMaterial>>
) {
    let mut ms = Vec::new();
    for i in 0..T::get_tile_count() {
        ms.push(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(T::get_tile_path(i))),
            alpha_mode: AlphaMode::Opaque,
            reflectance: 0.1,
            double_sided: true,
            cull_mode: None,
            ..default()
        }));
    }
    commands.insert_resource::<MaterialStorage<T>>(MaterialStorage {
        materials : ms,
        _phantom : std::marker::PhantomData
    });
}

#[derive(Resource)]
pub struct MaterialStorage<T : AnimSet + Send + Sync + 'static> {
    pub materials : Vec<Handle<StandardMaterial>>,
    _phantom : std::marker::PhantomData<T>
}

#[derive(Component)]
pub struct AutoAnim<T : AnimSet> {
    pub set : T,
    pub current_frame : usize,
    pub timer : Timer
}


#[derive(Clone)]
pub struct AnimRange {
    pub start: usize,
    pub end: usize,
}

impl AnimRange {
    pub fn new(start : usize, end : usize) -> Self {
        Self {
            start,
            end
        }
    }

}

