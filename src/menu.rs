use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {

    }
}

#[derive(Component)]
pub struct MainMenu;

fn setup_main_menu(
    mut commands: Commands,
) {
    
}