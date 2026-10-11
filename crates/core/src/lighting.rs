use bevy::prelude::*;
use gameplay::levels::{
    FacilityLight, FacilityPower, LightIntensity, LightingSettings, MainMenuAccentLight,
};

use crate::menu::GameState;

pub struct FacilityLightingPlugin;

impl Plugin for FacilityLightingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightingSettings>()
            .insert_resource(GlobalAmbientLight {
                color: Color::WHITE,
                brightness: LightingSettings::default().ambient_brightness,
                ..default()
            })
            .add_systems(
                Update,
                (apply_facility_lights, apply_menu_lights, apply_ambient),
            );
    }
}

fn apply_facility_lights(
    settings: Res<LightingSettings>,
    mut lights: Query<
        (&mut PointLight, &mut LightIntensity),
        (With<FacilityLight>, Added<LightIntensity>),
    >,
) {
    for (mut light, mut base) in &mut lights {
        base.0 *= settings.facility_modifier;
        light.intensity = base.0;
    }
}

fn apply_menu_lights(
    settings: Res<LightingSettings>,
    mut lights: Query<&mut SpotLight, (With<MainMenuAccentLight>, Added<MainMenuAccentLight>)>,
) {
    for mut light in &mut lights {
        light.intensity *= settings.menu_modifier;
    }
}

fn apply_ambient(
    settings: Res<LightingSettings>,
    power: Option<Res<FacilityPower>>,
    game: Option<Res<State<GameState>>>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let playing = game.is_none_or(|game| *game.get() == GameState::Playing);
    let brightness = if playing && power.is_some_and(|power| !power.on) {
        settings.outage_brightness
    } else {
        settings.ambient_brightness * settings.ambient_modifier
    };
    if ambient.brightness != brightness {
        ambient.brightness = brightness;
    }
}

#[cfg(test)]
#[path = "../tests/unit/lighting.rs"]
mod tests;
