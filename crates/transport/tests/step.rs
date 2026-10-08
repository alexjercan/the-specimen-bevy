use std::{
    io::{self, BufReader, Cursor, Read, Write},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use bevy::{input::InputPlugin, prelude::*};
use serde_json::Value;
use transport::{run, TransportPlugin, TransportTimeline};

fn ready(mut timeline: ResMut<TransportTimeline>) {
    timeline.ready();
}

#[derive(Resource, Clone)]
struct Updates(Arc<AtomicU64>);

fn count(updates: Res<Updates>) {
    updates.0.fetch_add(1, Ordering::SeqCst);
}

fn parse(output: &[u8]) -> Vec<Value> {
    String::from_utf8(output.to_vec())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[derive(Clone, Default)]
struct Flushed {
    pending: Arc<Mutex<Vec<u8>>>,
    flushed: Arc<Mutex<Vec<u8>>>,
}

impl Write for Flushed {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.pending.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut pending = self.pending.lock().unwrap();
        self.flushed.lock().unwrap().append(&mut pending);
        Ok(())
    }
}

struct Watched {
    input: Cursor<Vec<u8>>,
    output: Flushed,
    updates: Arc<AtomicU64>,
    first_read: Arc<Mutex<Option<(Vec<u8>, u64)>>>,
}

impl Read for Watched {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.first_read.lock().unwrap().get_or_insert_with(|| {
            (
                self.output.flushed.lock().unwrap().clone(),
                self.updates.load(Ordering::SeqCst),
            )
        });
        self.input.read(buffer)
    }
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
    let lines = parse(&output);
    assert_eq!(lines.len(), 3);
    for (line, tick) in lines.iter().zip([0, 1, 4]) {
        assert_eq!(line["tick"], tick);
        assert_eq!(line["player"], Value::Null);
        assert_eq!(line["won"], false);
    }
    assert!(lines[0]["map"].is_object());
    assert!(lines[1..].iter().all(|line| line.get("map").is_none()));
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
    let lines = parse(&output);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["tick"], 0);
    assert_eq!(lines[1]["tick"], 2);
}

#[test]
fn initial_snapshot_is_flushed_at_readiness_before_reading_input() {
    let mut app = App::new();
    let updates = Arc::new(AtomicU64::new(0));
    app.add_plugins((MinimalPlugins, InputPlugin, TransportPlugin))
        .insert_resource(Updates(updates.clone()))
        .add_systems(Update, count)
        .add_systems(
            PostUpdate,
            |updates: Res<Updates>, mut timeline: ResMut<TransportTimeline>| {
                if updates.0.load(Ordering::SeqCst) == 2 {
                    timeline.ready();
                }
            },
        );

    let output = Flushed::default();
    let first_read = Arc::new(Mutex::new(None));
    let input = BufReader::new(Watched {
        input: Cursor::new(b"{\"tick\":3}\n".to_vec()),
        output: output.clone(),
        updates: updates.clone(),
        first_read: first_read.clone(),
    });
    assert_eq!(run(app, input, output.clone()), AppExit::Success);

    let (before_input, updates_before_input) = first_read.lock().unwrap().clone().unwrap();
    assert_eq!(updates_before_input, 2);
    let initial = parse(&before_input);
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0]["tick"], 0);
    assert!(initial[0]["map"].is_object());

    assert!(output.pending.lock().unwrap().is_empty());
    let lines = parse(&output.flushed.lock().unwrap());
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1]["tick"], 3);
    assert!(lines[1].get("map").is_none());
    assert_eq!(updates.load(Ordering::SeqCst), 5);
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
    let lines = parse(&output);
    assert_eq!(lines[0]["tick"], 0);
    assert!(lines[1].get("error").is_some());
    assert!(lines[2].get("error").is_some());
    assert_eq!(lines[3]["tick"], 2);
    assert!(lines[4].get("error").is_some());
    assert!(lines[5].get("error").is_some());
    assert_eq!(lines[3]["player"], Value::Null);
    assert_eq!(lines[6]["tick"], 3);
    assert!(lines[1..].iter().all(|line| line.get("map").is_none()));
}

#[test]
fn empty_input_exits_after_the_initial_snapshot() {
    let mut app = App::new();
    let updates = Arc::new(AtomicU64::new(0));
    app.add_plugins((MinimalPlugins, InputPlugin, TransportPlugin))
        .insert_resource(Updates(updates.clone()))
        .add_systems(Startup, ready)
        .add_systems(Update, count);
    let mut output = Vec::new();
    assert_eq!(run(app, Cursor::new(""), &mut output), AppExit::Success);
    let lines = parse(&output);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["tick"], 0);
    assert!(lines[0]["map"].is_object());
    assert_eq!(updates.load(Ordering::SeqCst), 1);
}
