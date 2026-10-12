use std::time::Duration;

use bevy::prelude::*;
use gameplay::{
    achievements::{Achievement, AchievementPlugin, AchievementProgress, RunAchievements},
    ambience::AmbiencePlugin,
    devices::DevicePlugin,
    facility::{
        build_first_floor, DoorPlugin, FacilityPower, FacilityPowerPlugin, Module, PowerCue, Prop,
        MAX_OUTAGE_DELAY_SECS, MIN_OUTAGE_DELAY_SECS,
    },
    fuses::FuseSeed,
    interaction::PickupPlugin,
    monster::MonsterPlugin,
    player::{PlayerController, PlayerControlsEnabled},
};

mod common;

fn set_outage_in(app: &mut App, secs: f32) {
    let mut power = app.world_mut().resource_mut::<FacilityPower>();
    let remaining = power.outage_in().expect("outage pending");
    power.tick(remaining - secs);
}

#[test]
fn boiler_repairs_repeated_timed_outages_with_shared_f_interaction() {
    let mut app = common::player_app(Duration::from_millis(100));
    app.insert_resource(FuseSeed(42))
        .add_plugins((
            DoorPlugin,
            FacilityPowerPlugin,
            PickupPlugin,
            DevicePlugin,
            MonsterPlugin,
            AmbiencePlugin,
            AchievementPlugin,
        ))
        .add_systems(Startup, build_first_floor);
    app.finish();
    app.cleanup();
    app.update();

    let world = app.world_mut();
    let boiler = world
        .query::<(Entity, &Prop, &Transform)>()
        .iter(world)
        .find(|(_, prop, _)| prop.0 == Module::BoilerUnit)
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
    set_outage_in(&mut app, 0.15);
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    for _ in 0..5 {
        app.update();
    }
    assert!(app.world().resource::<FacilityPower>().is_on());
    let pending = app.world().resource::<FacilityPower>().outage_in().unwrap();
    assert!((pending - 0.15).abs() < 1e-4);
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = true;
    app.update();
    assert!(app.world().resource::<FacilityPower>().is_on());
    app.update();
    assert!(!app.world().resource::<FacilityPower>().is_on());
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PowerCue>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        vec![
            PowerCue::BreakerTrip { boiler: boiler.0 },
            PowerCue::PowerDown,
        ]
    );
    app.update();
    assert!(app
        .world_mut()
        .resource_mut::<Messages<PowerCue>>()
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
    assert!(app.world().resource::<FacilityPower>().is_on());
    assert!(app
        .world()
        .resource::<FacilityPower>()
        .outage_in()
        .is_some());
    assert!(
        app.world()
            .get::<RunAchievements>(player)
            .unwrap()
            .restored_boiler
    );
    assert!(app
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .contains(&Achievement::RestoreBoiler));
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PowerCue>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        vec![
            PowerCue::BoilerReset { boiler: boiler.0 },
            PowerCue::BoilerRestart { boiler: boiler.0 },
        ]
    );
    let next_delay = app.world().resource::<FacilityPower>().outage_in().unwrap();
    assert!((MIN_OUTAGE_DELAY_SECS..MAX_OUTAGE_DELAY_SECS).contains(&next_delay));
    set_outage_in(&mut app, 0.15);
    app.update();
    app.update();
    assert!(!app.world().resource::<FacilityPower>().is_on());
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PowerCue>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        vec![
            PowerCue::BreakerTrip { boiler: boiler.0 },
            PowerCue::PowerDown,
        ]
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyF);
    app.update();
    assert!(app.world().resource::<FacilityPower>().is_on());
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PowerCue>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        vec![
            PowerCue::BoilerReset { boiler: boiler.0 },
            PowerCue::BoilerRestart { boiler: boiler.0 },
        ]
    );
    assert!(app.world().get::<PlayerController>(player).is_some());
}
