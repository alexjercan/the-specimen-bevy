use super::*;

#[test]
fn facility_lights_and_menu_accents_use_shared_settings_once() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FacilityLightingPlugin);
    let facility = app
        .world_mut()
        .spawn((
            PointLight {
                intensity: 1_000.0,
                ..default()
            },
            LightIntensity(1_000.0),
            FacilityLight,
        ))
        .id();
    let unpowered = app
        .world_mut()
        .spawn((
            PointLight {
                intensity: 0.0,
                ..default()
            },
            LightIntensity(1_000.0),
            FacilityLight,
        ))
        .id();
    let other = app
        .world_mut()
        .spawn((
            PointLight {
                intensity: 1_000.0,
                ..default()
            },
            LightIntensity(1_000.0),
        ))
        .id();
    let menu = app
        .world_mut()
        .spawn((
            MainMenuAccentLight,
            SpotLight {
                intensity: 18_000.0,
                ..default()
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<LightingSettings>()
        .menu_modifier = 0.45;
    app.update();
    assert_eq!(
        app.world().get::<LightIntensity>(facility).unwrap().0,
        450.0
    );
    assert_eq!(
        app.world().get::<PointLight>(facility).unwrap().intensity,
        450.0
    );
    assert_eq!(
        app.world().get::<LightIntensity>(unpowered).unwrap().0,
        450.0
    );
    assert_eq!(
        app.world().get::<PointLight>(unpowered).unwrap().intensity,
        450.0
    );
    assert_eq!(app.world().get::<LightIntensity>(other).unwrap().0, 1_000.0);
    assert_eq!(
        app.world().get::<SpotLight>(menu).unwrap().intensity,
        8_100.0
    );
    app.update();
    assert_eq!(
        app.world().get::<LightIntensity>(facility).unwrap().0,
        450.0
    );
    assert_eq!(
        app.world().get::<SpotLight>(menu).unwrap().intensity,
        8_100.0
    );
}

#[test]
fn ambient_modifier_does_not_brighten_outages() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FacilityLightingPlugin);
    app.world_mut()
        .resource_mut::<LightingSettings>()
        .ambient_modifier = 0.55;
    app.update();
    assert_eq!(
        app.world().resource::<GlobalAmbientLight>().brightness,
        6.0 * 0.55
    );
    app.insert_resource(FacilityPower::new(42));
    app.world_mut().resource_mut::<FacilityPower>().outage();
    app.update();
    assert_eq!(
        app.world().resource::<GlobalAmbientLight>().brightness,
        0.35
    );
    app.world_mut().resource_mut::<FacilityPower>().restore();
    app.update();
    assert_eq!(
        app.world().resource::<GlobalAmbientLight>().brightness,
        6.0 * 0.55
    );
}
