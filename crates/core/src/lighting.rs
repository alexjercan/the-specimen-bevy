use bevy::prelude::*;
use gameplay::levels::{
    FacilityLight, FacilityPower, LightIntensity, LightingSettings, MainMenuAccentLight,
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
    power: Res<FacilityPower>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let brightness = if !power.on {
        settings.outage_brightness
    } else {
        settings.ambient_brightness * settings.ambient_modifier
    };
    if ambient.brightness != brightness {
        ambient.brightness = brightness;
    }
}
