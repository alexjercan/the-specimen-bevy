use bevy::prelude::*;
use game_core::{AppBuilder, CoreState, GameState};
use gameplay::{
    controller::PlayerController,
    levels::{
        select_fuse_slots, FuseInventory, FusePickup, FuseSeed, Room, FUSE_COUNT, FUSE_TABLES,
    },
};
use std::io::Cursor;
use transport::{run, TransportPlugin};

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
    let mut app = AppBuilder::headless().build();
    assert_eq!(
        *app.world().resource::<State<CoreState>>().get(),
        CoreState::Ready
    );
    app.finish();
    app.cleanup();
    app.update();
    let mut rooms = app.world_mut().query::<&Room>();
    assert_eq!(rooms.iter(app.world()).count(), 18);
}

#[test]
fn custom_plugin_replaces_default_game_plugin() {
    let mut app = AppBuilder::headless().with_main_plugin(MainPlugin).build();
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
fn headless_without_transport_does_not_install_transport() {
    let app = AppBuilder::headless().build();
    assert!(!app.is_plugin_added::<TransportPlugin>());
}

#[test]
fn headless_transport_reports_the_spawned_player() {
    let mut output = Vec::new();
    let app = AppBuilder::headless().with_transport().build();
    assert!(app.is_plugin_added::<TransportPlugin>());
    let exit = run(
        app,
        Cursor::new("{\"tick\":2,\"input\":{\"w\":true}}\n"),
        &mut output,
    );
    assert_eq!(exit, AppExit::Success);
    let lines = snapshots(&output);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["tick"], 0);
    assert_eq!(lines[0]["map"]["rooms"].as_array().unwrap().len(), 18);
    let snapshot = &lines[1];
    assert_eq!(snapshot["tick"], 2);
    assert_eq!(snapshot["player"]["position"][1], 1.6);
    assert!(snapshot["player"]["position"][2].as_f64().unwrap() < -5.0);
}

#[test]
fn headless_ignores_menu_and_builds_first_floor() {
    let mut app = AppBuilder::headless().with_menu().build();
    app.finish();
    app.cleanup();
    app.update();
    assert!(!app.world().contains_resource::<State<GameState>>());
    let mut rooms = app.world_mut().query::<&Room>();
    assert_eq!(rooms.iter(app.world()).count(), 18);
}

#[test]
fn headless_transport_ignores_menu_and_reports_the_spawned_player() {
    let mut output = Vec::new();
    let app = AppBuilder::headless().with_menu().with_transport().build();
    let exit = run(app, Cursor::new("{\"tick\":1,\"input\":{}}\n"), &mut output);
    assert_eq!(exit, AppExit::Success);
    let lines = snapshots(&output);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["tick"], 0);
    let snapshot = &lines[1];
    assert_eq!(snapshot["tick"], 1);
    assert_eq!(snapshot["player"]["position"][1], 1.6);
}

#[test]
fn headless_builder_spawns_logical_fuses_and_player_inventory() {
    let mut app = AppBuilder::headless().build();
    app.insert_resource(FuseSeed(7));
    app.finish();
    app.cleanup();
    app.update();
    let mut fuses = app.world_mut().query::<&FusePickup>();
    assert_eq!(fuses.iter(app.world()).count(), FUSE_COUNT);
    let mut players = app
        .world_mut()
        .query_filtered::<&FuseInventory, With<PlayerController>>();
    assert_eq!(players.single(app.world()).unwrap().0, 0);
}

fn fuse_slots(app: &mut App) -> Vec<usize> {
    let mut slots: Vec<usize> = app
        .world_mut()
        .query::<&FusePickup>()
        .iter(app.world())
        .map(|fuse| fuse.slot)
        .collect();
    slots.sort_unstable();
    slots
}

#[test]
fn builder_seed_places_fuses_at_the_seeded_tables() {
    for seed in [0, 7, u64::MAX] {
        let mut app = AppBuilder::headless().with_seed(seed).build();
        assert_eq!(app.world().resource::<FuseSeed>(), &FuseSeed(seed));
        app.finish();
        app.cleanup();
        app.update();
        assert_eq!(fuse_slots(&mut app), select_fuse_slots(seed));
    }
}

#[test]
fn builder_without_seed_leaves_fuse_seed_unset() {
    let mut app = AppBuilder::headless().build();
    assert!(!app.world().contains_resource::<FuseSeed>());
    app.finish();
    app.cleanup();
    app.update();
    assert_eq!(fuse_slots(&mut app).len(), FUSE_COUNT);
}

#[test]
fn headless_transport_snapshots_do_not_depend_on_the_fuse_seed() {
    let other = (0..)
        .find(|&seed| select_fuse_slots(seed) != select_fuse_slots(0))
        .unwrap();
    let outputs: Vec<Vec<u8>> = [0, other]
        .into_iter()
        .map(|seed| {
            let mut output = Vec::new();
            let app = AppBuilder::headless()
                .with_transport()
                .with_seed(seed)
                .build();
            let input = Cursor::new("{\"tick\":1}\n{\"tick\":30,\"input\":{\"look\":[200,0]}}\n");
            assert_eq!(run(app, input, &mut output), AppExit::Success);
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
fn transport_builder_keeps_the_seed() {
    let app = AppBuilder::headless().with_transport().with_seed(3).build();
    assert_eq!(app.world().resource::<FuseSeed>(), &FuseSeed(3));
}
