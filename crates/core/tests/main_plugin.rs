use bevy::prelude::*;
use game_core::{AppBuilder, CoreState, GameState};
use gameplay::levels::Room;
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
