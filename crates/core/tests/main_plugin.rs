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
    let snapshot: serde_json::Value = serde_json::from_slice(&output).unwrap();
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
    let snapshot: serde_json::Value = serde_json::from_slice(&output).unwrap();
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
        assert_eq!(
            fuse_slots(&mut app),
            select_fuse_slots(seed, FUSE_TABLES.len())
        );
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
fn transport_builder_keeps_the_seed() {
    let app = AppBuilder::headless().with_transport().with_seed(3).build();
    assert_eq!(app.world().resource::<FuseSeed>(), &FuseSeed(3));
}
