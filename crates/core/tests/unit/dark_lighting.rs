use super::*;

#[test]
fn facility_lighting_is_shared_by_normal_and_rendered_transport() {
    assert!(AppBuilder::new().facility_lighting_enabled());
    assert!(AppBuilder::new()
        .with_transport()
        .facility_lighting_enabled());
    assert!(!AppBuilder::headless()
        .with_transport()
        .facility_lighting_enabled());
    assert!(!AppBuilder::new()
        .with_main_plugin(GamePlugin)
        .facility_lighting_enabled());
}

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
