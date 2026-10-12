mod common;

use std::time::Duration;

use bevy::{
    ecs::{resource::IsResource, system::SystemIdMarker},
    input::InputPlugin,
    prelude::*,
    state::app::StatesPlugin,
    time::TimeUpdateStrategy,
};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::{
    facility::{FacilityPower, Room, RunRoot, StartRun},
    fuses::FuseSeed,
    monster::Monster,
    player::PlayerController,
    presets::{GamePreset, PresetRules},
    GameplayPlugin,
};

fn run_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        StatesPlugin,
        EnhancedInputPlugin,
    ))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
    .insert_resource(common::controller_settings())
    .insert_resource(FuseSeed(common::SEED))
    .add_plugins(GameplayPlugin);
    app.finish();
    app.update();
    app
}

fn count<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), F>()
        .iter(app.world())
        .count()
}

fn blackout_preset() -> GamePreset {
    GamePreset::Custom(PresetRules {
        blackout: true,
        ..PresetRules::NORMAL
    })
}

#[test]
fn start_run_builds_the_world_once_with_the_preset() {
    let mut app = run_app();
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert_eq!(count::<With<Room>>(&mut app), 0);

    let preset = blackout_preset();
    app.world_mut().write_message(StartRun { preset });
    app.update();

    assert_eq!(*app.world().resource::<GamePreset>(), preset);
    assert!(!app.world().resource::<FacilityPower>().is_on());
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(count::<With<Monster>>(&mut app), 1);
    let rooms = count::<With<Room>>(&mut app);
    assert!(rooms > 0);

    app.update();
    app.update();
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(count::<With<Monster>>(&mut app), 1);
    assert_eq!(count::<With<Room>>(&mut app), rooms);
}

#[test]
fn a_second_run_after_the_first_ends_rebuilds_with_power_reset() {
    let mut app = run_app();
    app.world_mut().write_message(StartRun {
        preset: blackout_preset(),
    });
    app.update();
    let rooms = count::<With<Room>>(&mut app);
    assert!(!app.world().resource::<FacilityPower>().is_on());

    let roots: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<RunRoot>>()
        .iter(app.world())
        .collect();
    for root in roots {
        app.world_mut().entity_mut(root).despawn();
    }
    app.update();
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(
        *app.world().resource::<FacilityPower>(),
        FacilityPower::default()
    );

    let preset = GamePreset::default();
    app.world_mut().write_message(StartRun { preset });
    app.update();
    assert_eq!(*app.world().resource::<GamePreset>(), preset);
    assert!(app.world().resource::<FacilityPower>().is_on());
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(count::<With<Monster>>(&mut app), 1);
    assert_eq!(count::<With<Room>>(&mut app), rooms);
}

// NOTE: the first run registers cached systems and their id resources, which stay by design.
fn world_entities(app: &mut App) -> usize {
    count::<(Without<SystemIdMarker>, Without<IsResource>)>(app)
}

fn end_run(app: &mut App) {
    let roots: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<RunRoot>>()
        .iter(app.world())
        .collect();
    for root in roots {
        app.world_mut().entity_mut(root).despawn();
    }
    app.update();
}

#[test]
fn ending_a_run_leaves_only_the_entities_from_before_it() {
    let mut app = run_app();
    let before = world_entities(&mut app);
    for preset in [blackout_preset(), GamePreset::default()] {
        app.world_mut().write_message(StartRun { preset });
        app.update();
        assert!(world_entities(&mut app) > before);
        end_run(&mut app);
        assert_eq!(world_entities(&mut app), before);
    }
}
