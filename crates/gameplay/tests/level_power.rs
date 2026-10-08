use std::time::Duration;

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_audio::{PlaySound, PlaySourceSound, Sound, SourceSounds};
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin, PlayerControlsEnabled},
    levels::{
        build_first_floor, FacilityPower, FacilityPowerPlugin, FuseSeed, Prop, PropSoundsPlugin,
        BOILER_UNIT, MAX_OUTAGE_DELAY_SECS, MIN_OUTAGE_DELAY_SECS,
    },
};

#[test]
fn boiler_repairs_repeated_timed_outages_with_shared_f_interaction() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .insert_resource(FuseSeed(42))
        .add_plugins(PlayerControllerPlugin::default().without_camera())
        .add_plugins((FacilityPowerPlugin, PropSoundsPlugin))
        .add_systems(Startup, build_first_floor);
    app.finish();
    app.cleanup();
    app.update();

    let world = app.world_mut();
    let boiler = world
        .query::<(Entity, &Prop, &Transform)>()
        .iter(world)
        .find(|(_, prop, _)| prop.0 == BOILER_UNIT)
        .map(|(entity, _, transform)| (entity, transform.translation))
        .expect("first floor has a boiler");
    let eye = boiler.1 + Vec3::new(0.0, 1.6, 2.0);
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_translation(eye).looking_at(boiler.1 + Vec3::Y * 1.2, Vec3::Y),
        ))
        .id();
    app.world_mut()
        .resource_mut::<FacilityPower>()
        .remaining_secs = 0.15;
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
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sound, Sound::PowerDown);
    assert_eq!(events[0].position, None);
    let trips: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySourceSound>>()
        .drain()
        .collect();
    assert_eq!(trips.len(), 1);
    assert_eq!(trips[0].source, boiler.0);
    assert_eq!(trips[0].sound, Sound::BreakerTrip);
    app.update();
    assert!(app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .next()
        .is_none());

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
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySourceSound>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| event.source == boiler.0));
    assert_eq!(events[0].sound, Sound::BoilerReset);
    assert_eq!(events[1].sound, Sound::BoilerRestart);
    assert!(app
        .world()
        .get::<SourceSounds>(boiler.0)
        .unwrap()
        .0
        .contains(&(Sound::BoilerReset, Vec3::Y * 1.2)));
    let next_delay = app.world().resource::<FacilityPower>().remaining_secs;
    assert!((MIN_OUTAGE_DELAY_SECS..MAX_OUTAGE_DELAY_SECS).contains(&next_delay));
    app.world_mut()
        .resource_mut::<FacilityPower>()
        .remaining_secs = 0.15;
    app.update();
    app.update();
    assert!(!app.world().resource::<FacilityPower>().on);
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sound, Sound::PowerDown);
    let trips: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySourceSound>>()
        .drain()
        .collect();
    assert_eq!(trips.len(), 1);
    assert_eq!(trips[0].source, boiler.0);
    assert_eq!(trips[0].sound, Sound::BreakerTrip);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyF);
    app.update();
    assert!(app.world().resource::<FacilityPower>().on);
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySourceSound>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| event.source == boiler.0));
    assert_eq!(events[0].sound, Sound::BoilerReset);
    assert_eq!(events[1].sound, Sound::BoilerRestart);
    assert!(app.world().get::<PlayerController>(player).is_some());
}
