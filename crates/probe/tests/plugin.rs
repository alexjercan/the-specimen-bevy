use std::{path::PathBuf, time::Duration};

use bevy::{
    prelude::*,
    window::{PresentMode, PrimaryWindow},
    world_serialization::{WorldAsset, WorldAssetRoot},
};
use probe::{
    world_instances_ready, Output, Phase, ProbeConfig, ProbeError, ProbePlugin, ProbeState,
    Readiness,
};

fn config() -> ProbeConfig {
    ProbeConfig {
        warmup: 2,
        frames: 5,
        resolution: UVec2::new(640, 360),
        ..default()
    }
}

fn app(config: ProbeConfig, ready: fn(&World) -> Readiness) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.add_plugins(ProbePlugin::new(config, ready).unwrap());
    app.finish();
    app.cleanup();
    app
}

fn run(app: &mut App, limit: usize) -> Option<AppExit> {
    for _ in 0..limit {
        std::thread::sleep(Duration::from_millis(1));
        app.update();
        if let Some(exit) = app.should_exit() {
            return Some(exit);
        }
    }
    None
}

fn state(app: &App) -> &ProbeState {
    app.world().resource::<ProbeState>()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("probe-plugin-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn build_pins_the_primary_window() {
    let app = app(config(), |_| Readiness::Waiting);
    let mut windows = app
        .world()
        .try_query_filtered::<&Window, With<PrimaryWindow>>()
        .unwrap();
    let window = windows.single(app.world()).unwrap();
    assert_eq!(window.resolution.width(), 640.0);
    assert_eq!(window.resolution.height(), 360.0);
    assert_eq!(window.present_mode, PresentMode::AutoNoVsync);
    assert!(!window.resizable);
}

#[test]
fn ready_scene_is_measured_and_written() {
    let dir = scratch("success");
    let path = dir.join("run.json");
    let mut app = app(
        ProbeConfig {
            out: Some(Output::Json(path.clone())),
            ..config()
        },
        |_| Readiness::Ready,
    );
    assert_eq!(run(&mut app, 50), Some(AppExit::Success));
    let state = state(&app);
    assert_eq!(state.phase(), Phase::Done);
    assert_eq!(state.samples().len(), 5);
    let report = state.report().unwrap();
    assert_eq!(report.stats.frames, 5);
    assert_eq!((report.width, report.height), (640, 360));
    assert!(report.stats.p99_ms >= report.stats.p50_ms);
    assert!(std::fs::read_to_string(&path)
        .unwrap()
        .contains("\"frames\": 5"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn nothing_is_measured_before_ready() {
    let mut app = app(config(), |_| Readiness::Waiting);
    assert_eq!(run(&mut app, 20), None);
    assert_eq!(state(&app).phase(), Phase::WaitReady);
    assert!(state(&app).samples().is_empty());
}

#[test]
fn failed_scene_exits_with_error() {
    let mut app = app(config(), |_| Readiness::Failed("assets failed".into()));
    assert_eq!(run(&mut app, 5), Some(AppExit::error()));
    assert_eq!(state(&app).phase(), Phase::Failed);
    assert!(state(&app).failure().unwrap().contains("assets failed"));
    assert!(state(&app).report().is_none());
}

#[test]
fn readiness_times_out() {
    let mut app = app(
        ProbeConfig {
            timeout: Duration::from_millis(5),
            ..config()
        },
        |_| Readiness::Waiting,
    );
    app.update();
    std::thread::sleep(Duration::from_millis(10));
    assert_eq!(run(&mut app, 5), Some(AppExit::error()));
    assert!(state(&app).failure().unwrap().contains("not ready"));
}

#[test]
fn resized_window_aborts_the_capture() {
    let mut app = app(config(), |_| Readiness::Ready);
    run(&mut app, 4);
    let mut windows = app
        .world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>();
    windows
        .single_mut(app.world_mut())
        .unwrap()
        .resolution
        .set(800.0, 600.0);
    assert_eq!(run(&mut app, 10), Some(AppExit::error()));
    assert!(state(&app).failure().unwrap().contains("800x600"));
}

#[test]
fn missing_window_aborts_the_capture() {
    let mut app = app(config(), |_| Readiness::Ready);
    let mut windows = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>();
    let window = windows.single(app.world()).unwrap();
    app.world_mut().despawn(window);
    assert_eq!(run(&mut app, 10), Some(AppExit::error()));
    assert!(state(&app).failure().unwrap().contains("missing"));
}

#[test]
fn extra_primary_window_aborts_the_capture() {
    let mut app = app(config(), |_| Readiness::Ready);
    app.world_mut().spawn((
        Window {
            resolution: (640, 360).into(),
            ..default()
        },
        PrimaryWindow,
    ));
    assert_eq!(run(&mut app, 10), Some(AppExit::error()));
    assert!(state(&app).failure().unwrap().contains("more than one"));
}

#[test]
#[should_panic(expected = "needs exactly one primary window")]
fn build_without_a_window_panics() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(ProbePlugin::new(config(), |_| Readiness::Ready).unwrap());
}

#[test]
fn early_exit_is_an_error() {
    let mut app = app(config(), |_| Readiness::Waiting);
    app.update();
    app.world_mut().write_message(AppExit::Success);
    app.update();
    assert_eq!(app.should_exit(), Some(AppExit::error()));
    assert!(state(&app).failure().unwrap().contains("exited before"));
}

#[test]
fn invalid_config_is_rejected() {
    let ready = |_: &World| Readiness::Ready;
    assert!(matches!(
        ProbePlugin::new(
            ProbeConfig {
                frames: 0,
                ..config()
            },
            ready
        ),
        Err(ProbeError::ZeroFrames)
    ));
    assert!(matches!(
        ProbePlugin::new(
            ProbeConfig {
                timeout: Duration::ZERO,
                ..config()
            },
            ready
        ),
        Err(ProbeError::ZeroTimeout)
    ));
    assert!(matches!(
        ProbePlugin::new(
            ProbeConfig {
                resolution: UVec2::new(0, 720),
                ..config()
            },
            ready
        ),
        Err(ProbeError::InvalidResolution(_))
    ));
}

#[test]
fn world_instances_wait_for_spawned_roots() {
    let mut world = World::new();
    assert!(world_instances_ready(&world));
    world.spawn(WorldAssetRoot(Handle::<WorldAsset>::default()));
    assert!(!world_instances_ready(&world));
}
