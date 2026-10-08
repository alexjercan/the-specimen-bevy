use std::io::Cursor;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::controller::{PlayerController, PlayerControllerPlugin};
use serde_json::Value;
use transport::{run, TransportPlugin, TransportTimeline};

#[test]
fn headless_transport_click_persists_until_explicit_release_and_reports_charge() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin));
    app.add_plugins(PlayerControllerPlugin::default().without_camera());
    app.add_plugins(TransportPlugin);
    app.add_systems(Startup, |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
        commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
        timeline.ready();
    });

    let input = Cursor::new(
        "{\"tick\":1,\"input\":{\"flashlight\":true}}\n\
         {\"tick\":30}\n\
         {\"tick\":31,\"input\":{\"flashlight\":false}}\n\
         {\"tick\":32,\"input\":{\"flashlight\":true}}\n",
    );
    let mut output = Vec::new();
    let result = run(app, input, &mut output);
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
