mod common;

use std::io::Cursor;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_app::{FrameStep, Specimen};
use gameplay::{player::PlayerController, presets::GamePreset};
use serde_json::Value;
use stdio::{run, HeadlessStdio, StdioTimeline};

#[test]
fn headless_sprint_depletes_then_recovers_without_autorestarting_while_shift_is_held() {
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
        "{\"version\":1,\"tick\":1,\"input\":{\"w\":true,\"shift\":true}}\n\
         {\"version\":1,\"tick\":520}\n\
         {\"version\":1,\"tick\":700}\n\
         {\"version\":1,\"tick\":701,\"input\":{\"shift\":false}}\n\
         {\"version\":1,\"tick\":702,\"input\":{\"shift\":true}}\n",
    );
    let mut output = Vec::new();
    assert!(matches!(
        run::<Specimen>(app, input, &mut output),
        AppExit::Success
    ));
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
