use super::*;

#[test]
fn graphics_preset_applies_to_existing_and_new_cameras() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<GameSettings>()
        .add_systems(Update, apply_graphics);
    let first = app.world_mut().spawn(Camera3d::default()).id();
    app.update();
    assert_eq!(*app.world().get::<Msaa>(first).unwrap(), Msaa::Sample4);
    app.world_mut().resource_mut::<GameSettings>().graphics = GraphicsQuality::Low;
    let second = app.world_mut().spawn(Camera3d::default()).id();
    app.update();
    assert_eq!(*app.world().get::<Msaa>(first).unwrap(), Msaa::Off);
    assert_eq!(*app.world().get::<Msaa>(second).unwrap(), Msaa::Off);
    app.world_mut().resource_mut::<GameSettings>().graphics = GraphicsQuality::Medium;
    app.update();
    assert_eq!(*app.world().get::<Msaa>(first).unwrap(), Msaa::Sample2);
}

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
