mod common;

use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_app::{FrameStep, Specimen};
use gameplay::facility::Module;
use gameplay::{
    facility::{Door, DoorState},
    player::PlayerController,
    presets::GamePreset,
};
use stdio::{run, HeadlessStdio, StdioTimeline};

fn step(commands: &str) -> Vec<serde_json::Value> {
    let mut app = App::new();
    app.insert_resource(common::controller_settings())
        .insert_resource(GamePreset::default());
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        common::CueOwners,
        HeadlessStdio::<Specimen>::new(FrameStep::DEFAULT.duration()),
    ));
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<StdioTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            timeline.ready();
        },
    );
    let mut output = Vec::new();
    assert_eq!(
        run::<Specimen>(app, Cursor::new(commands), &mut output),
        AppExit::Success
    );
    let mut lines: Vec<serde_json::Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.remove(0)["tick"], 0);
    lines
}

#[test]
fn controls_hold_until_explicitly_released() {
    let lines = step("{\"version\":1,\"tick\":2,\"input\":{\"w\":true}}\n{\"version\":1,\"tick\":4,\"input\":{\"shift\":true}}\n{\"version\":1,\"tick\":5}\n{\"version\":1,\"tick\":6,\"input\":{\"w\":false,\"shift\":false}}\n");
    assert_eq!(lines[0]["tick"], 2);
    assert_eq!(
        lines[0]["player"]["movement"],
        serde_json::json!([0.0, 1.0])
    );
    assert_eq!(lines[1]["player"]["running"], true);
    assert_eq!(
        lines[2]["player"]["movement"],
        serde_json::json!([0.0, 1.0])
    );
    assert_eq!(lines[2]["player"]["running"], true);
    assert_eq!(
        lines[3]["player"]["movement"],
        serde_json::json!([0.0, 0.0])
    );
    assert_eq!(lines[3]["player"]["running"], false);
    let z = |index: usize| lines[index]["player"]["position"][2].as_f64().unwrap();
    assert!(z(0) < -0.09 && z(0) > -0.11);
    assert!(z(1) < z(0) - 0.19);
    assert!(z(2) < z(1) - 0.09);
    assert!((z(3) - z(2)).abs() < 0.0001);
    assert_eq!(lines[3]["player"]["position"][1], 1.6);
}

#[test]
fn f_toggles_an_aimed_door_once_per_press() {
    let mut app = App::new();
    app.insert_resource(common::controller_settings())
        .insert_resource(GamePreset::default());
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        common::CueOwners,
        HeadlessStdio::<Specimen>::new(FrameStep::DEFAULT.duration()),
    ));
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<StdioTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            commands.spawn(Door {
                position: Vec2::new(0.0, -1.5),
                rotation: Quat::IDENTITY,
                frame: Module::WallDoorway,
                panel: Module::DoorPanel,
                state: DoorState::Closed,
            });
            timeline.ready();
        },
    );
    let observed = Arc::new(Mutex::new(Vec::new()));
    let states = observed.clone();
    app.add_systems(PostUpdate, move |doors: Query<&Door>| {
        states.lock().unwrap().push(doors.single().unwrap().state);
    });
    let mut output = Vec::new();
    assert_eq!(
        run::<Specimen>(
            app,
            Cursor::new("{\"version\":1,\"tick\":1,\"input\":{\"f\":true}}\n{\"version\":1,\"tick\":3}\n{\"version\":1,\"tick\":4,\"input\":{\"f\":false}}\n{\"version\":1,\"tick\":5,\"input\":{\"f\":true}}\n"),
            &mut output,
        ),
        AppExit::Success,
    );
    let states = observed.lock().unwrap();
    assert_eq!(
        &states[1..],
        &[
            DoorState::Open,
            DoorState::Open,
            DoorState::Open,
            DoorState::Open,
            DoorState::Closed,
        ]
    );
    assert_eq!(String::from_utf8(output).unwrap().lines().count(), 5);
}

#[test]
fn look_applies_once_and_invalid_requests_do_not_change_state() {
    let lines = step("{\"version\":1,\"tick\":2,\"input\":{\"look\":[100,50]}}\n{\"version\":1,\"tick\":2,\"input\":{\"w\":true}}\n{\"version\":1,\"tick\":3,\"input\":{\"look\":[1e100,0]}}\n{\"version\":1,\"tick\":4}\n");
    assert!(lines[0]["player"]["yaw"].as_f64().unwrap() < -0.14);
    assert!(lines[1].get("error").is_some());
    assert!(lines[2].get("error").is_some());
    assert_eq!(lines[3]["tick"], 4);
    assert!(
        (lines[3]["player"]["yaw"].as_f64().unwrap() - lines[0]["player"]["yaw"].as_f64().unwrap())
            .abs()
            < 0.0001
    );
    assert_eq!(
        lines[3]["player"]["movement"],
        serde_json::json!([0.0, 0.0])
    );
}
