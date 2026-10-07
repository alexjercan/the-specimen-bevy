use std::io::Cursor;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::controller::{PlayerController, PlayerControllerPlugin};
use transport::{run, TransportPlugin, TransportTimeline};

fn step(commands: &str) -> Vec<serde_json::Value> {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        TransportPlugin,
    ));
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            timeline.ready();
        },
    );
    let mut output = Vec::new();
    assert_eq!(
        run(app, Cursor::new(commands), &mut output),
        AppExit::Success
    );
    String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn controls_hold_until_explicitly_released() {
    let lines = step("{\"tick\":2,\"input\":{\"w\":true}}\n{\"tick\":4,\"input\":{\"shift\":true}}\n{\"tick\":5}\n{\"tick\":6,\"input\":{\"w\":false,\"shift\":false}}\n");
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
fn look_applies_once_and_invalid_requests_do_not_change_state() {
    let lines = step("{\"tick\":2,\"input\":{\"look\":[100,50]}}\n{\"tick\":2,\"input\":{\"w\":true}}\n{\"tick\":3,\"input\":{\"look\":[1e100,0]}}\n{\"tick\":4}\n");
    assert!(lines[0]["player"]["yaw"].as_f64().unwrap() < -0.19);
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
