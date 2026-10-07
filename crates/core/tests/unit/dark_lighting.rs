use super::*;

#[test]
fn dim_new_lights_scales_authored_intensity_once() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, dim_new_lights);
    let entity = app
        .world_mut()
        .spawn((
            PointLight {
                intensity: 1_000.0,
                ..default()
            },
            LightIntensity(1_000.0),
        ))
        .id();
    app.update();
    let base = app.world().get::<LightIntensity>(entity).unwrap().0;
    let intensity = app.world().get::<PointLight>(entity).unwrap().intensity;
    assert_eq!(base, 1_000.0 * WINDOWED_LIGHT_SCALE);
    assert_eq!(intensity, base);
    app.update();
    let base_after = app.world().get::<LightIntensity>(entity).unwrap().0;
    assert_eq!(base_after, base);
}
