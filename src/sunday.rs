use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{storyteller::Storyteller, GameSet, safe_area::{SafeArea, LandSafeArea}};

pub struct SundayPlugin;

pub const DAY_SUN_COLOR: &str = "f2ecbe";
pub const EVENING_SUN_COLOR: &str = "cfaf56";
pub const DUSK_SUN_COLOR: &str = "f2ecbe";
pub const NIGHT_SUN_COLOR: &str = "506886";

pub const SUN_BASE_ILLUMINANCE: f32 = 50000.0;
pub const AMBIENT_BASE_ILLUMINANCE: f32 = 1.0;

pub const SUN_EVENING_ILLUMINANCE: f32 = 10000.0;
pub const SUN_DUSK_ILLUMINANCE: f32 = 10000.0;
pub const SUN_NIGHT_ILLUMINANCE: f32 = 10000.0;

pub const AMBIENT_DAY_COLOR: &str = "2ba4a9";
pub const AMBIENT_NIGHT_COLOR: &str = "643a69";

pub const AMBIENT_DAY_ILLUMINANCE: f32 = 1.0;
pub const AMBIENT_NIGHT_ILLUMINANCE: f32 = 0.1;

const DAY_TIME: f32 = 0.3;
const EVENING_TIME: f32 = 0.6;
const NIGHT_TIME: f32 = 0.7;

impl Plugin for SundayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sunday_system.in_set(GameSet::Playing));
        app.add_systems(Update, set_day_state.in_set(GameSet::Playing));
        app.add_state::<DayState>();

        app.add_systems(Update, safe_area_evening_decrease.in_set(GameSet::Playing).run_if(in_state(DayState::Evening)));
        app.add_systems(OnEnter(DayState::Night), delete_land_area_at_night);
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum DayState {
    #[default]
    Day,
    Evening,
    Night,
}

fn set_day_state(
    mut state : ResMut<NextState<DayState>>,
    current_state : Res<State<DayState>>,
    teller : Res<Storyteller>,
    time : Res<Time>
) {
    let uniform_time = teller.get_level_unfirom_time(&time);
    if uniform_time < DAY_TIME {
        if *current_state != DayState::Day {
            state.set(DayState::Day);
        }
    } else if uniform_time < EVENING_TIME {
        if *current_state != DayState::Evening {
            state.set(DayState::Evening);
        }
    } else if uniform_time < NIGHT_TIME {
        if *current_state != DayState::Night {
            state.set(DayState::Night);
        }
    }
}

fn sunday_system(
    time: Res<Time>,
    teller : Res<Storyteller>,
    mut sun : Query<(&mut Transform, &mut DirectionalLight)>,
    mut ambient_light: ResMut<AmbientLight>
) {
    let Ok((mut transform, mut light)) = sun.get_single_mut() else {
        warn!("Could not get directional light");
        return;
    };

    let uniform_time = teller.get_level_unfirom_time(&time);

    if uniform_time < DAY_TIME {
        let sun_falloff = 1.0;
        ambient_light.color = Color::hex(AMBIENT_DAY_COLOR).unwrap();
        let sun_angle = sun_falloff * std::f32::consts::PI / 4.0;
        let pos = transform.translation.clone();
        transform.look_at(pos + Vec3::new(-(PI / 4.0).cos(), -sun_angle.sin(), -sun_angle.cos()), Vec3::Y);

    } else if uniform_time < EVENING_TIME {
        let sun_falloff = 1.0 - (uniform_time - DAY_TIME) / (EVENING_TIME - DAY_TIME);
        let color = 
            Color::hex(DAY_SUN_COLOR).unwrap() * sun_falloff 
                + Color::hex(EVENING_SUN_COLOR).unwrap() * (1.0 - sun_falloff);
        let sun_angle = sun_falloff * std::f32::consts::PI / 4.0;
        let illuminance = SUN_BASE_ILLUMINANCE * sun_falloff + SUN_EVENING_ILLUMINANCE * (1.0 - sun_falloff);

        let pos = transform.translation.clone();
        transform.look_at(pos + Vec3::new(-(PI / 4.0).cos(), -sun_angle.sin(), -sun_angle.cos()), Vec3::Y);
        light.color = color;
        light.illuminance = illuminance;

        ambient_light.color = Color::hex(AMBIENT_DAY_COLOR).unwrap();
    } else if uniform_time < NIGHT_TIME {
        let sun_falloff = 1.0 - (uniform_time - EVENING_TIME) / (NIGHT_TIME - EVENING_TIME);
        let color = Color::hex(NIGHT_SUN_COLOR).unwrap();
        let sun_angle = (1.0 - sun_falloff) * std::f32::consts::PI / 4.0;
        let illuminance = SUN_NIGHT_ILLUMINANCE;
        let pos = transform.translation.clone();
        transform.look_at(pos + Vec3::new(-(PI / 4.0).cos(), -sun_angle.sin(), -sun_angle.cos()), Vec3::Y);
        light.color = color;
        light.illuminance = illuminance;

        ambient_light.color = 
            Color::hex(AMBIENT_NIGHT_COLOR).unwrap() * (1.0 - sun_falloff) + Color::hex(AMBIENT_DAY_COLOR).unwrap() * sun_falloff;
        ambient_light.brightness = AMBIENT_NIGHT_ILLUMINANCE * (1.0 - sun_falloff) + sun_falloff * AMBIENT_DAY_ILLUMINANCE;
    } else {
        
    }
}



fn safe_area_evening_decrease(
    mut areas : Query<(&mut SafeArea, &LandSafeArea)>,
    time : Res<Time>,
    teller : Res<Storyteller>
) {
    let uniform_time = teller.get_level_unfirom_time(&time);
    let evening_time = (uniform_time - DAY_TIME) / (EVENING_TIME - DAY_TIME);
    let scale = 1.0 - evening_time;
    for (mut area, land_area) in areas.iter_mut() {
        *area = land_area.start_area.get_scaled(scale);
    }
}


fn delete_land_area_at_night(
    mut commands: Commands,
    mut areas : Query<Entity, With<LandSafeArea>>,
) {
    for entity in areas.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}