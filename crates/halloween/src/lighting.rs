use bevy::prelude::*;
use gameplay::levels::LightIntensity;

const FACILITY_LIGHT_SCALE: f32 = 0.45;
const FACILITY_AMBIENT_BRIGHTNESS: f32 = 6.0;
const AMBIENT_LIGHT_SCALE: f32 = 0.55;

pub struct HalloweenLightingPlugin;

impl Plugin for HalloweenLightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (dim_facility_lights, dim_menu_flashlight, dim_ambient),
        );
    }
}

fn dim_facility_lights(
    mut lights: Query<(&mut PointLight, &mut LightIntensity), Added<LightIntensity>>,
) {
    for (mut light, mut base) in &mut lights {
        base.0 *= FACILITY_LIGHT_SCALE;
        light.intensity *= FACILITY_LIGHT_SCALE;
    }
}

fn dim_menu_flashlight(mut lights: Query<(&Name, &mut SpotLight), Added<SpotLight>>) {
    for (name, mut light) in &mut lights {
        if name.as_str() == "Main menu flashlight accent" {
            light.intensity *= FACILITY_LIGHT_SCALE;
        }
    }
}

fn dim_ambient(mut ambient: Option<ResMut<GlobalAmbientLight>>) {
    if let Some(ambient) = ambient.as_deref_mut() {
        if (ambient.brightness - FACILITY_AMBIENT_BRIGHTNESS).abs() < 0.01 {
            ambient.brightness *= AMBIENT_LIGHT_SCALE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halloween_dimming_keeps_outages_dark_and_scales_new_lights_once() {
        let mut app = App::new();
        app.insert_resource(GlobalAmbientLight {
            brightness: 6.0,
            ..default()
        })
        .add_plugins(HalloweenLightingPlugin);
        let light = app
            .world_mut()
            .spawn((
                PointLight {
                    intensity: 100.0,
                    ..default()
                },
                LightIntensity(100.0),
            ))
            .id();
        let menu_light = app
            .world_mut()
            .spawn((
                Name::new("Main menu flashlight accent"),
                SpotLight {
                    intensity: 18_000.0,
                    ..default()
                },
            ))
            .id();
        let other_light = app
            .world_mut()
            .spawn((Name::new("ordinary spotlight"), SpotLight::default()))
            .id();
        let unpowered_light = app
            .world_mut()
            .spawn((PointLight::default(), LightIntensity(100.0)))
            .id();
        app.world_mut()
            .get_mut::<PointLight>(unpowered_light)
            .unwrap()
            .intensity = 0.0;
        app.update();
        assert_eq!(
            app.world().resource::<GlobalAmbientLight>().brightness,
            6.0 * AMBIENT_LIGHT_SCALE
        );
        assert_eq!(
            app.world().get::<SpotLight>(menu_light).unwrap().intensity,
            18_000.0 * FACILITY_LIGHT_SCALE
        );
        assert_eq!(
            app.world().get::<SpotLight>(other_light).unwrap().intensity,
            SpotLight::default().intensity
        );
        assert_eq!(
            app.world().get::<PointLight>(light).unwrap().intensity,
            100.0 * FACILITY_LIGHT_SCALE
        );
        assert_eq!(
            app.world().get::<LightIntensity>(light).unwrap().0,
            100.0 * FACILITY_LIGHT_SCALE
        );
        assert_eq!(
            app.world()
                .get::<PointLight>(unpowered_light)
                .unwrap()
                .intensity,
            0.0
        );
        assert_eq!(
            app.world()
                .get::<LightIntensity>(unpowered_light)
                .unwrap()
                .0,
            100.0 * FACILITY_LIGHT_SCALE
        );
        app.update();
        assert_eq!(
            app.world().get::<SpotLight>(menu_light).unwrap().intensity,
            18_000.0 * FACILITY_LIGHT_SCALE
        );
        assert_eq!(
            app.world().get::<PointLight>(light).unwrap().intensity,
            100.0 * FACILITY_LIGHT_SCALE
        );
        app.world_mut()
            .resource_mut::<GlobalAmbientLight>()
            .brightness = 0.35;
        app.update();
        assert_eq!(
            app.world().resource::<GlobalAmbientLight>().brightness,
            0.35
        );
    }
}
