#![cfg(feature = "halloween")]

use game_app::{AppBuilder, Frontend};
use gameplay::facility::LightingSettings;

#[test]
fn headless_builds_skip_the_cosmetic_halloween_theme() {
    let app = AppBuilder::new(Frontend::Headless).build();
    let dimmed = app
        .world()
        .get_resource::<LightingSettings>()
        .is_some_and(|settings| {
            settings.facility_modifier < LightingSettings::default().facility_modifier
        });
    assert!(!dimmed);
}
