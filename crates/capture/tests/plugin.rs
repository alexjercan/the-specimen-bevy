use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use capture::{
    loops::{
        loop_audio_path, loop_end, loop_start, loop_written, loop_written_at, LoopCapturePlugin,
    },
    screenshot::{screenshot_start, screenshot_written, screenshot_written_at, CaptureLog},
    CapturePlugin, CaptureState,
};

fn capture_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CapturePlugin::new(30)));
    app
}

fn unique_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "capture-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir(&dir).expect("create test directory");
    dir
}

#[test]
fn capture_plugin_initializes_screenshot_and_shared_state() {
    let app = capture_app();
    assert!(app.world().contains_resource::<CaptureState>());
    assert!(app.world().contains_resource::<CaptureLog>());
}

#[test]
fn collector_lifecycle() {
    let mut state = CaptureState::default();
    state.register();
    assert_eq!(state.pending(), 1);
    state.finish();
    assert_eq!(state.pending(), 0);
    state.fail("encode failed".into());
    assert_eq!(state.failed(), Some("encode failed"));
}

#[test]
fn loop_profile_validates_fps() {
    assert_eq!(LoopCapturePlugin::default().fps, 30);
    assert!(std::panic::catch_unwind(|| LoopCapturePlugin::new(0)).is_err());
}

#[test]
fn loop_start_keeps_existing_output_and_exposes_audio_path_while_recording() {
    let mut app = capture_app();
    let world = app.world_mut();
    assert_eq!(loop_audio_path(world), None);
    assert!(!loop_written(world, "sample"));
    assert!(!loop_written_at("sample")(world));

    let dir = unique_dir("loop");
    let output = dir.join("existing.webm");
    std::fs::write(&output, b"old video").expect("create old video");
    let name = output.to_str().expect("UTF-8 path");
    loop_start(world, name);

    assert!(!loop_written(world, name));
    assert_eq!(world.resource::<CaptureState>().pending(), 1);
    assert_eq!(
        std::fs::read(&output).expect("read old video"),
        b"old video"
    );
    let audio = loop_audio_path(world).expect("open loop exposes its audio path");
    assert_eq!(audio.file_name().unwrap(), "audio.f32le");
    let staging = audio.parent().unwrap().to_path_buf();
    assert!(staging.is_dir());

    std::fs::remove_dir_all(staging).expect("remove test staging");
    std::fs::remove_dir_all(dir).expect("remove test directory");
}

#[test]
fn closing_an_empty_loop_reports_failure_without_releasing_capture_gate() {
    let mut app = capture_app();
    loop_start(app.world_mut(), "empty-loop");
    let staging = loop_audio_path(app.world())
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    loop_end(app.world_mut());
    app.update();

    let state = app.world().resource::<CaptureState>();
    assert_eq!(state.pending(), 1);
    assert_eq!(state.failed(), Some("loop has no frames"));
    assert_eq!(loop_audio_path(app.world()), None);
    assert!(!loop_written(app.world(), "empty-loop"));

    std::fs::remove_dir_all(staging).expect("remove test staging");
}

#[test]
fn screenshot_written_tracks_exact_paths_and_start_registers_a_collector() {
    let mut app = capture_app();
    let world = app.world_mut();
    let path = PathBuf::from("art/visuals/screenshots/a.png");
    assert!(!screenshot_written(world, &path));
    assert!(!screenshot_written_at(path.clone())(world));
    assert!(!screenshot_written(world, "a.png"));

    screenshot_start(world, &path);
    assert_eq!(world.resource::<CaptureState>().pending(), 1);
    assert!(!screenshot_written(world, &path));
    assert!(!screenshot_written_at("a.png")(world));
}
