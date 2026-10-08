use std::io::Cursor;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::controller::{PlayerController, PlayerControllerPlugin};
use serde_json::Value;
use transport::{run, TransportPlugin, TransportTimeline};

#[test]
fn headless_sprint_depletes_then_recovers_without_autorestarting_while_shift_is_held() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin));
    app.add_plugins(PlayerControllerPlugin::default().without_camera());
    app.add_plugins(TransportPlugin);
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            timeline.ready();
        },
    );
    let input = Cursor::new(
        "{\"tick\":1,\"input\":{\"w\":true,\"shift\":true}}\n\
         {\"tick\":520}\n\
         {\"tick\":700}\n\
         {\"tick\":701,\"input\":{\"shift\":false}}\n\
         {\"tick\":702,\"input\":{\"shift\":true}}\n",
    );
    let mut output = Vec::new();
    assert!(matches!(run(app, input, &mut output), AppExit::Success));
    let mut replies: Vec<Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(replies.remove(0)["tick"], 0);
    assert_eq!(replies[0]["player"]["running"], true);
    assert_eq!(replies[1]["player"]["stamina_exhausted"], true);
    assert_eq!(replies[1]["player"]["running"], false);
    assert_eq!(replies[2]["player"]["running"], false);
    assert!(replies[2]["player"]["stamina_charge"].as_f64().unwrap() > 0.0);
    assert_eq!(replies[4]["player"]["running"], true);
}
