use bevy::prelude::*;
use gameplay::facility::LightingSettings;

pub struct HalloweenLightingPlugin;

impl Plugin for HalloweenLightingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightingSettings>();
        let mut settings = app.world_mut().resource_mut::<LightingSettings>();
        settings.facility_modifier *= 0.45;
        settings.menu_modifier *= 0.45;
        settings.ambient_modifier *= 0.55;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halloween_configures_only_the_shared_lighting_settings() {
        let mut app = App::new();
        app.add_plugins(HalloweenLightingPlugin);
        let settings = app.world().resource::<LightingSettings>();
        let normal = LightingSettings::default();
        assert_eq!(settings.facility_modifier, normal.facility_modifier * 0.45);
        assert_eq!(settings.menu_modifier, normal.menu_modifier * 0.45);
        assert_eq!(settings.ambient_modifier, normal.ambient_modifier * 0.55);
        assert_eq!(settings.ambient_brightness, normal.ambient_brightness);
        assert_eq!(settings.outage_brightness, normal.outage_brightness);
    }
}
