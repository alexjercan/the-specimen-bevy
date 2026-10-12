use std::time::Duration;

use bevy::prelude::*;
use gameplay::facility::{
    build_main_menu_background, FacilityPower, FacilityPowerPlugin, MainsPowered, Module,
    PowerState, Prop, MAX_OUTAGE_DELAY_SECS, MIN_OUTAGE_DELAY_SECS,
};

mod common;

#[test]
fn seeded_outages_repeat_after_each_repair() {
    let mut power = FacilityPower::new(42);
    assert_eq!(power, FacilityPower::new(42));
    assert!((MIN_OUTAGE_DELAY_SECS..MAX_OUTAGE_DELAY_SECS).contains(&power.outage_in().unwrap()));
    power.tick(power.outage_in().unwrap() - 0.1);
    assert!(power.is_on());
    power.tick(0.2);
    assert!(!power.is_on());
    assert!(power.outage_in().is_none());
    power.restore();
    assert!(power.is_on());
    assert!(power.outage_in().is_some());
    assert!((MIN_OUTAGE_DELAY_SECS..MAX_OUTAGE_DELAY_SECS).contains(&power.outage_in().unwrap()));
    assert_eq!(power, {
        let mut another = FacilityPower::new(42);
        another.tick(1000.0);
        another.restore();
        another
    });
    power.tick(power.outage_in().unwrap());
    assert!(!power.is_on());
    power.restore();
    assert!(power.is_on());
    assert!(power.outage_in().is_some());
    power.tick(power.outage_in().unwrap());
    assert!(!power.is_on());
}

#[test]
fn blackout_starts_dark_and_never_schedules_or_allows_repair() {
    let mut power = FacilityPower::blackout(42);
    assert!(!power.is_on());
    assert!(power.outage_in().is_none());
    assert!(!power.repairable());
    power.tick(1000.0);
    assert!(!power.is_on());
    assert!(!power.repairable());

    let mut normal = FacilityPower::new(42);
    assert!(!normal.repairable());
    normal.outage();
    assert!(normal.repairable());
}

#[test]
fn power_state_follows_schedule_trip_repair_and_blackout() {
    let idle = FacilityPower::default();
    assert_eq!(idle.state(), PowerState::On { until_outage: None });
    let mut idle_outage = idle;
    idle_outage.outage();
    idle_outage.tick(1000.0);
    assert_eq!(idle_outage.state(), PowerState::On { until_outage: None });

    let mut power = FacilityPower::new(42);
    let PowerState::On {
        until_outage: Some(delay),
    } = power.state()
    else {
        panic!("a seeded run schedules its first outage");
    };
    power.tick(delay);
    assert_eq!(power.state(), PowerState::Tripped);
    power.outage();
    assert_eq!(power.state(), PowerState::Tripped);
    power.restore();
    assert!(matches!(
        power.state(),
        PowerState::On {
            until_outage: Some(_)
        }
    ));
    power.outage();
    assert_eq!(power.state(), PowerState::Tripped);

    let mut dark = FacilityPower::blackout(42);
    assert_eq!(dark.state(), PowerState::Blackout);
    dark.outage();
    dark.tick(1000.0);
    assert_eq!(dark.state(), PowerState::Blackout);
    dark.restore();
    assert!(matches!(
        dark.state(),
        PowerState::On {
            until_outage: Some(_)
        }
    ));
}

#[test]
fn mains_powered_marks_only_grid_props_outside_the_menu_background() {
    let mut app = common::player_app(Duration::from_millis(100));
    app.add_plugins(FacilityPowerPlugin);
    app.finish();
    app.cleanup();
    let props: Vec<(Module, Entity)> = Module::ALL
        .iter()
        .map(|&module| {
            let entity = app
                .world_mut()
                .spawn((Prop(module), Transform::default()))
                .id();
            (module, entity)
        })
        .collect();
    app.add_systems(Startup, |mut commands: Commands| {
        build_main_menu_background(&mut commands);
    });
    app.update();

    for (module, entity) in props {
        assert_eq!(
            app.world().get::<MainsPowered>(entity).is_some(),
            module.mains_powered(),
            "{module:?}"
        );
    }
    assert!(!Module::WallLampRed.mains_powered());
    assert!(!Module::ExitSign.mains_powered());
    assert!(!Module::BoilerUnit.mains_powered());
    assert!(Module::CeilingLightCool.mains_powered());

    let world = app.world_mut();
    let nested: Vec<(Module, Entity, bool)> = world
        .query::<(&Prop, &ChildOf, Has<MainsPowered>)>()
        .iter(world)
        .map(|(prop, parent, mains)| (prop.0, parent.parent(), mains))
        .collect();
    let (menu_props, dressing): (Vec<&(Module, Entity, bool)>, Vec<_>) = nested
        .iter()
        .partition(|(_, parent, _)| world.get::<Prop>(*parent).is_none());
    assert!(!menu_props.is_empty());
    assert!(menu_props.iter().any(|(module, ..)| module.mains_powered()));
    assert!(menu_props.iter().all(|(_, _, mains)| !mains));
    assert!(dressing
        .iter()
        .all(|(module, _, mains)| *mains == module.mains_powered()));
}
