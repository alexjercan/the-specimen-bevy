use bevy::prelude::*;
use game_app::{AppBuilder, Frontend, Specimen};
use game_flow::{CoreState, GameState};
use gameplay::{
    facility::{select_fuse_slots, Escaped, FacilityPower, Prop, Room, FUSE_TABLES},
    fuses::{FuseInventory, FuseSeed, FUSE_COUNT},
    interaction::PickupKind,
    monster::{Caught, MonsterFigure},
    player::PlayerController,
};
use std::io::Cursor;
use stdio::{run, StdioTimeline};

mod common;

use common::{count, game_state, named, player};

#[derive(Resource)]
struct Installed;

struct MainPlugin;

impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Installed);
    }
}

#[test]
fn headless_builder_enters_ready_and_builds_first_floor() {
    let mut app = AppBuilder::new(Frontend::Headless).build();
    assert_eq!(
        *app.world().resource::<State<CoreState>>().get(),
        CoreState::Ready
    );
    app.finish();
    app.cleanup();
    app.update();
    let mut rooms = app.world_mut().query::<&Room>();
    assert_eq!(rooms.iter(app.world()).count(), 26);
}

#[test]
fn headless_props_carry_their_world_pose() {
    let mut app = AppBuilder::new(Frontend::Headless).build();
    app.finish();
    app.cleanup();
    app.update();
    app.update();
    let mut props = app
        .world_mut()
        .query_filtered::<(&Transform, &GlobalTransform), (With<Prop>, Without<ChildOf>)>();
    let poses: Vec<_> = props.iter(app.world()).collect();
    assert!(!poses.is_empty());
    assert!(poses
        .iter()
        .any(|(transform, _)| transform.translation != Vec3::ZERO));
    for (transform, global) in poses {
        assert!(transform.translation.distance(global.translation()) < 1e-4);
    }
}

#[test]
fn custom_plugin_replaces_default_game_plugin() {
    let mut app = AppBuilder::new(Frontend::Headless)
        .with_main_plugin(MainPlugin)
        .build();
    assert!(app.world().contains_resource::<Installed>());
    app.finish();
    app.cleanup();
    app.update();
    let mut rooms = app.world_mut().query::<&Room>();
    assert_eq!(rooms.iter(app.world()).count(), 0);
}

fn snapshots(output: &[u8]) -> Vec<serde_json::Value> {
    String::from_utf8(output.to_vec())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn headless_without_stdio_has_no_stdio_timeline() {
    let app = AppBuilder::new(Frontend::Headless).build();
    assert!(!app.world().contains_resource::<StdioTimeline>());
}

#[test]
fn headless_stdio_reports_the_spawned_player() {
    let mut output = Vec::new();
    let app = AppBuilder::new(Frontend::Headless).with_stdio().build();
    assert!(app.world().contains_resource::<StdioTimeline>());
    let exit = run::<Specimen>(
        app,
        Cursor::new("{\"version\":1,\"tick\":2,\"input\":{\"w\":true}}\n"),
        &mut output,
    );
    assert_eq!(exit, AppExit::Success);
    let lines = snapshots(&output);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["tick"], 0);
    assert_eq!(lines[0]["map"]["rooms"].as_array().unwrap().len(), 26);
    let snapshot = &lines[1];
    assert_eq!(snapshot["tick"], 2);
    assert_eq!(snapshot["player"]["position"][1], 1.6);
    assert!(snapshot["player"]["position"][2].as_f64().unwrap() < -5.0);
}

#[test]
fn headless_builder_spawns_logical_fuses_and_player_inventory() {
    let mut app = AppBuilder::new(Frontend::Headless).build();
    app.insert_resource(FuseSeed(7));
    app.finish();
    app.cleanup();
    app.update();
    let mut pickups = app.world_mut().query::<&PickupKind>();
    assert_eq!(
        pickups
            .iter(app.world())
            .filter(|kind| matches!(kind, PickupKind::Fuse { .. }))
            .count(),
        FUSE_COUNT
    );
    let mut players = app
        .world_mut()
        .query_filtered::<&FuseInventory, With<PlayerController>>();
    assert_eq!(players.single(app.world()).unwrap().0, 0);
}

fn fuse_slots(app: &mut App) -> Vec<usize> {
    let mut slots: Vec<usize> = app
        .world_mut()
        .query::<&PickupKind>()
        .iter(app.world())
        .filter_map(|kind| match kind {
            PickupKind::Fuse { slot } => Some(*slot),
            PickupKind::Flashbang | PickupKind::Detector => None,
        })
        .collect();
    slots.sort_unstable();
    slots
}

#[test]
fn builder_seed_places_fuses_at_the_seeded_tables() {
    for seed in [0, 7, u64::MAX] {
        let mut app = AppBuilder::new(Frontend::Headless).with_seed(seed).build();
        assert_eq!(app.world().resource::<FuseSeed>(), &FuseSeed(seed));
        app.finish();
        app.cleanup();
        app.update();
        assert_eq!(fuse_slots(&mut app), select_fuse_slots(seed));
    }
}

#[test]
fn builder_without_seed_still_seeds_the_run() {
    let mut app = AppBuilder::new(Frontend::Headless).build();
    assert!(app.world().contains_resource::<FuseSeed>());
    app.finish();
    app.cleanup();
    app.update();
    assert_eq!(fuse_slots(&mut app).len(), FUSE_COUNT);
}

#[test]
fn headless_stdio_snapshots_do_not_depend_on_the_fuse_seed() {
    let other = (0..)
        .find(|&seed| select_fuse_slots(seed) != select_fuse_slots(0))
        .unwrap();
    let outputs: Vec<Vec<u8>> = [0, other]
        .into_iter()
        .map(|seed| {
            let mut output = Vec::new();
            let app = AppBuilder::new(Frontend::Headless)
                .with_stdio()
                .with_seed(seed)
                .build();
            let input = Cursor::new("{\"version\":1,\"tick\":1}\n{\"version\":1,\"tick\":30,\"input\":{\"look\":[200,0]}}\n");
            assert_eq!(run::<Specimen>(app, input, &mut output), AppExit::Success);
            output
        })
        .collect();
    assert_eq!(outputs[0], outputs[1]);
    let lines = snapshots(&outputs[0]);
    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[0]["map"]["fuse_candidates"].as_array().unwrap().len(),
        FUSE_TABLES.len()
    );
    assert!(!lines[0]["map"].to_string().contains("has_fuse"));
}

#[test]
fn stdio_builder_keeps_the_seed() {
    let app = AppBuilder::new(Frontend::Headless)
        .with_stdio()
        .with_seed(3)
        .build();
    assert_eq!(app.world().resource::<FuseSeed>(), &FuseSeed(3));
}

#[test]
fn headless_runs_keep_outcomes_queryable_without_menu_screens() {
    for outcome in 0..2 {
        let mut app = AppBuilder::new(Frontend::Headless).with_seed(7).build();
        app.finish();
        app.cleanup();
        app.update();
        app.update();
        assert_eq!(game_state(&app), GameState::Playing);
        let player = player(&mut app);
        if outcome == 0 {
            app.world_mut().entity_mut(player).insert(Escaped);
        } else {
            app.world_mut().entity_mut(player).insert(Caught {
                monster: Entity::PLACEHOLDER,
                remaining: 0.0,
            });
        }
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(common::player(&mut app), player);
        assert_eq!(app.world().get::<Escaped>(player).is_some(), outcome == 0);
        assert_eq!(app.world().get::<Caught>(player).is_some(), outcome == 1);
        assert_eq!(game_state(&app), GameState::Playing);
        assert_eq!(count::<With<Room>>(&mut app), 26);
        assert!(named(&mut app, "Complete screen").is_none());
        assert!(named(&mut app, "Game over screen").is_none());
        assert_eq!(count::<With<MonsterFigure>>(&mut app), 0);
        assert_eq!(count::<With<Camera>>(&mut app), 0);
    }
}

#[test]
fn headless_leaving_playing_resets_power_and_keeps_the_world() {
    let mut app = AppBuilder::new(Frontend::Headless).with_seed(7).build();
    app.finish();
    app.cleanup();
    app.update();
    app.update();
    assert_eq!(game_state(&app), GameState::Playing);
    *app.world_mut().resource_mut::<FacilityPower>() = FacilityPower::blackout(7);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::GameOver);
    app.update();
    assert_eq!(game_state(&app), GameState::GameOver);
    assert_eq!(
        *app.world().resource::<FacilityPower>(),
        FacilityPower::default()
    );
    assert_eq!(count::<With<Room>>(&mut app), 26);
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
}
