use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use bevy::{input::InputPlugin, prelude::*};
use transport::{run, TransportPlugin, TransportTimeline};

fn ready(mut timeline: ResMut<TransportTimeline>) {
    timeline.ready();
}

#[derive(Resource, Clone)]
struct Updates(Arc<AtomicU64>);

fn count(updates: Res<Updates>) {
    updates.0.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn steps_to_absolute_ticks_without_free_running() {
    let mut app = App::new();
    let updates = Arc::new(AtomicU64::new(0));
    app.add_plugins((MinimalPlugins, InputPlugin, TransportPlugin))
        .insert_resource(Updates(updates.clone()))
        .add_systems(Startup, ready)
        .add_systems(Update, count);

    let mut output = Vec::new();
    let exit = run(
        app,
        Cursor::new("{\"tick\":1}\n{\"tick\":4}\n"),
        &mut output,
    );
    assert_eq!(exit, AppExit::Success);
    assert_eq!(
        String::from_utf8(output).unwrap(),
        "{\"tick\":1,\"player\":null}\n{\"tick\":4,\"player\":null}\n"
    );
    assert_eq!(updates.load(Ordering::SeqCst), 5);
}

#[test]
fn readiness_offsets_loading_updates_and_defers_input() {
    let mut app = App::new();
    let updates = Arc::new(AtomicU64::new(0));
    app.add_plugins((MinimalPlugins, InputPlugin, TransportPlugin))
        .insert_resource(Updates(updates.clone()))
        .add_systems(Update, count)
        .add_systems(
            PostUpdate,
            |updates: Res<Updates>,
             mut timeline: ResMut<TransportTimeline>,
             keyboard: Res<ButtonInput<KeyCode>>| {
                if updates.0.load(Ordering::SeqCst) == 3 {
                    assert!(!keyboard.pressed(KeyCode::KeyW));
                    timeline.ready();
                }
            },
        );

    let mut output = Vec::new();
    assert_eq!(
        run(
            app,
            Cursor::new("{\"tick\":2,\"input\":{\"w\":true}}\n"),
            &mut output
        ),
        AppExit::Success
    );
    assert_eq!(updates.load(Ordering::SeqCst), 5);
    assert!(String::from_utf8(output).unwrap().contains("\"tick\":2"));
}

#[test]
fn rejects_invalid_and_past_ticks_without_advancing() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, TransportPlugin))
        .add_systems(Startup, ready);

    let mut output = Vec::new();
    let exit = run(
        app,
        Cursor::new("nope\n{\"tick\":0}\n{\"tick\":2}\n{\"tick\":2}\n{\"tick\":3,\"input\":1}\n{\"tick\":3}\n"),
        &mut output,
    );
    assert_eq!(exit, AppExit::Success);
    let lines: Vec<serde_json::Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(lines[0].get("error").is_some());
    assert!(lines[1].get("error").is_some());
    assert_eq!(lines[2]["tick"], 2);
    assert!(lines[3].get("error").is_some());
    assert!(lines[4].get("error").is_some());
    assert_eq!(lines[2]["player"], serde_json::Value::Null);
    assert_eq!(lines[5]["tick"], 3);
}

#[test]
fn empty_input_exits_without_a_snapshot() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, TransportPlugin))
        .add_systems(Startup, ready);
    let mut output = Vec::new();
    assert_eq!(run(app, Cursor::new(""), &mut output), AppExit::Success);
    assert!(output.is_empty());
}
