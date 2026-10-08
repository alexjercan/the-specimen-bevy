use std::time::Duration;

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin, PlayerControlsEnabled},
    levels::{
        build_first_floor, FacilityPower, FacilityPowerPlugin, Prop, BOILER_UNIT,
        MAX_OUTAGE_DELAY_SECS, MIN_OUTAGE_DELAY_SECS,
    },
};

#[test]
fn boiler_repairs_repeated_timed_outages_with_shared_f_interaction() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(100)))
        .add_plugins(PlayerControllerPlugin::default().without_camera())
        .add_plugins(FacilityPowerPlugin)
        .add_systems(Startup, build_first_floor);
    app.finish();
    app.cleanup();
    app.update();

    let world = app.world_mut();
    let boiler = world
        .query::<(&Prop, &Transform)>()
        .iter(world)
        .find(|(prop, _)| prop.0 == BOILER_UNIT)
        .map(|(_, transform)| transform.translation)
        .expect("first floor has a boiler");
    let eye = boiler + Vec3::new(0.0, 1.6, 2.0);
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_translation(eye).looking_at(boiler + Vec3::Y * 1.2, Vec3::Y),
        ))
        .id();
    app.world_mut().resource_mut::<FacilityPower>().remaining_secs = 0.15;
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    for _ in 0..5 {
        app.update();
    }
    assert!(app.world().resource::<FacilityPower>().on);
    assert_eq!(app.world().resource::<FacilityPower>().remaining_secs, 0.15);
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = true;
    app.update();
    assert!(app.world().resource::<FacilityPower>().on);
    app.update();
    assert!(!app.world().resource::<FacilityPower>().on);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyF);
    app.update();
    assert!(app.world().resource::<FacilityPower>().on);
    assert!(app.world().resource::<FacilityPower>().outage_pending);
    let next_delay = app.world().resource::<FacilityPower>().remaining_secs;
    assert!((MIN_OUTAGE_DELAY_SECS..MAX_OUTAGE_DELAY_SECS).contains(&next_delay));
    app.world_mut().resource_mut::<FacilityPower>().remaining_secs = 0.15;
    app.update();
    app.update();
    assert!(!app.world().resource::<FacilityPower>().on);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyF);
    app.update();
    assert!(app.world().resource::<FacilityPower>().on);
    assert!(app.world().get::<PlayerController>(player).is_some());
}
