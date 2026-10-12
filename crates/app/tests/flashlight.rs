mod common;

use std::io::Cursor;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_app::{FrameStep, Specimen};
use gameplay::{player::PlayerController, presets::GamePreset};
use serde_json::Value;
use stdio::{run, HeadlessStdio, StdioTimeline};

#[test]
fn headless_stdio_click_persists_until_explicit_release_and_reports_charge() {
    let mut app = App::new();
    app.insert_resource(common::controller_settings())
        .insert_resource(GamePreset::default());
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin));
    app.add_plugins(common::CueOwners);
    app.add_plugins(HeadlessStdio::<Specimen>::new(
        FrameStep::DEFAULT.duration(),
    ));
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<StdioTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            timeline.ready();
        },
    );

    let input = Cursor::new(
        "{\"version\":1,\"tick\":1,\"input\":{\"flashlight\":true}}\n\
         {\"version\":1,\"tick\":30}\n\
         {\"version\":1,\"tick\":31,\"input\":{\"flashlight\":false}}\n\
         {\"version\":1,\"tick\":32,\"input\":{\"flashlight\":true}}\n",
    );
    let mut output = Vec::new();
    let result = run::<Specimen>(app, input, &mut output);
    assert!(matches!(result, AppExit::Success));
    let mut replies: Vec<Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(replies.remove(0)["tick"], 0);
    assert_eq!(replies.len(), 4);
    assert_eq!(replies[0]["player"]["flashlight_on"], true);
    assert_eq!(replies[1]["player"]["flashlight_on"], true);
    assert!(replies[1]["player"]["flashlight_charge"].as_f64().unwrap() < 1.0);
    assert_eq!(replies[3]["player"]["flashlight_on"], false);
}
