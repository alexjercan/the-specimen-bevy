use bevy::prelude::*;

use super::{
    builder::LightIntensity,
    lights::{AmbientNits, FacilityLight, LightingSettings, MainMenuAccentLight},
    power::FacilityPower,
};

pub struct FacilityLightingPlugin;

impl Plugin for FacilityLightingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightingSettings>()
            .init_resource::<GlobalAmbientLight>()
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
        base.0 = settings.facility_modifier.of(base.0);
        light.intensity = base.0;
    }
}

fn apply_menu_lights(
    settings: Res<LightingSettings>,
    mut lights: Query<&mut SpotLight, (With<MainMenuAccentLight>, Added<MainMenuAccentLight>)>,
) {
    for mut light in &mut lights {
        light.intensity = settings.menu_modifier.of(light.intensity);
    }
}

fn apply_ambient(
    settings: Res<LightingSettings>,
    power: Res<FacilityPower>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let AmbientNits(brightness) = if !power.is_on() {
        settings.outage_brightness
    } else {
        settings.ambient_brightness * settings.ambient_modifier
    };
    if ambient.brightness != brightness {
        ambient.brightness = brightness;
    }
}
