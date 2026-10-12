use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bevy::{
    asset::AssetPlugin,
    audio::{PlaybackSettings, Volume},
    prelude::*,
};
use capture::{
    screenshot::{screenshot_start, screenshot_written, screenshot_written_at, CaptureLog},
    video::{
        video_close_at_exit, video_end, video_staging, video_start, video_written,
        video_written_at, AudioEvent, LoggedBus, VideoCapturePlugin, VideoProfile, AUDIO_LOG,
    },
    CapturePlugin, CaptureState,
};
use game_audio::Bus;

// NOTE: a test fixture step; the game step is owned by game_app::FrameStep.
const STEP: Duration = Duration::from_nanos(1_000_000_000 / 30);

fn capture_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        CapturePlugin::new(VideoProfile::PREVIEW, STEP),
    ));
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
fn video_profiles_validate_fps_against_the_frame_step() {
    let (preview, last) = (VideoProfile::PREVIEW, VideoProfile::FINAL);
    assert_eq!(preview.fps, last.fps);
    assert!(preview.crf > last.crf);
    assert!(preview.output_size.is_some());
    assert!(last.output_size.is_none());
    assert!(preview.deadline < last.deadline);
    VideoCapturePlugin::new(VideoProfile::PREVIEW, STEP / 2);
    assert!(std::panic::catch_unwind(|| {
        VideoCapturePlugin::new(VideoProfile::PREVIEW.with_fps(0), STEP)
    })
    .is_err());
    assert!(std::panic::catch_unwind(|| {
        VideoCapturePlugin::new(VideoProfile::PREVIEW.with_fps(45), STEP)
    })
    .is_err());
}

#[test]
fn video_start_keeps_existing_output_and_stages_in_the_profile_directory() {
    let dir = unique_dir("video");
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        CapturePlugin::new(VideoProfile::PREVIEW.with_staging(&dir), STEP),
    ));
    let world = app.world_mut();
    assert_eq!(video_staging(world), None);
    assert!(!video_written(world, "sample"));
    assert!(!video_written_at("sample")(world));

    let output = dir.join("existing.webm");
    std::fs::write(&output, b"old video").expect("create old video");
    let name = output.to_str().expect("UTF-8 path");
    video_start(world, name);

    assert!(!video_written(world, name));
    assert_eq!(world.resource::<CaptureState>().pending(), 1);
    assert_eq!(
        std::fs::read(&output).expect("read old video"),
        b"old video"
    );
    let staging = video_staging(world).expect("open video exposes its staging directory");
    assert_eq!(staging.parent(), Some(dir.as_path()));
    assert!(staging.join("audio.jsonl").is_file());

    std::fs::remove_dir_all(dir).expect("remove test directory");
}

#[test]
fn closing_an_empty_video_reports_failure_without_releasing_capture_gate() {
    let mut app = capture_app();
    video_start(app.world_mut(), "empty-video");
    let staging = video_staging(app.world()).unwrap();
    video_end(app.world_mut());
    app.update();

    let state = app.world().resource::<CaptureState>();
    assert_eq!(state.pending(), 1);
    assert_eq!(state.failed(), Some("video has no frames"));
    assert_eq!(video_staging(app.world()), None);
    assert!(!video_written(app.world(), "empty-video"));
    assert!(!staging.exists());
}

#[test]
fn a_video_aborted_at_the_deadline_leaves_no_staging() {
    let dir = unique_dir("video-deadline");
    let profile = VideoProfile {
        deadline: Duration::ZERO,
        ..VideoProfile::PREVIEW.with_staging(&dir)
    };
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CapturePlugin::new(profile, STEP)));
    video_start(app.world_mut(), "late-video");
    let staging = video_staging(app.world()).unwrap();
    for _ in 0..4 {
        app.update();
    }
    video_end(app.world_mut());
    std::thread::sleep(Duration::from_millis(2));
    app.update();

    let state = app.world().resource::<CaptureState>();
    assert!(state
        .failed()
        .is_some_and(|reason| reason.contains("video frames arrived within")));
    assert_eq!(video_staging(app.world()), None);
    assert!(!video_written(app.world(), "late-video"));
    assert!(!staging.exists());

    std::fs::remove_dir_all(dir).expect("remove test directory");
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

fn audio_events(world: &World) -> Vec<AudioEvent> {
    let log = video_staging(world).unwrap().join(AUDIO_LOG);
    std::fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn audio_players_are_logged_from_spawn_to_despawn_with_their_asset_path() {
    let assets = unique_dir("audio-log-assets");
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: assets.to_string_lossy().into_owned(),
            ..default()
        },
        TransformPlugin,
        CapturePlugin::new(VideoProfile::PREVIEW, STEP),
    ))
    .init_asset::<AudioSource>();
    video_start(app.world_mut(), "logged");
    app.update();
    assert!(audio_events(app.world()).is_empty());

    let handle: Handle<AudioSource> = app.world().resource::<AssetServer>().load("beep.wav");
    let voice = app
        .world_mut()
        .spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.5)),
        ))
        .id();
    app.update();
    app.update();
    let events = audio_events(app.world());
    assert_eq!(events.len(), 1, "{events:?}");
    match &events[0] {
        AudioEvent::Start {
            frame,
            voice: id,
            bus,
            path,
            volume,
            looped,
            spatial,
            ..
        } => {
            assert_eq!(*frame, 1);
            assert_eq!(*bus, LoggedBus::Unrouted);
            assert_eq!(*id, voice.to_bits());
            assert_eq!(path, "beep.wav");
            assert!((volume - 0.5).abs() < 1e-6);
            assert!(*looped);
            assert_eq!(*spatial, None);
        }
        other => panic!("expected a start event, got {other:?}"),
    }

    app.world_mut().despawn(voice);
    app.update();
    let events = audio_events(app.world());
    assert_eq!(events.len(), 2, "{events:?}");
    assert_eq!(
        events[1],
        AudioEvent::Stop {
            frame: 3,
            voice: voice.to_bits(),
            bus: LoggedBus::Unrouted,
        }
    );

    let staging = video_staging(app.world()).unwrap();
    std::fs::remove_dir_all(staging).expect("remove test staging");
    std::fs::remove_dir_all(assets).expect("remove test assets");
}

#[test]
fn audio_players_log_the_bus_they_play_on() {
    let assets = unique_dir("audio-log-bus");
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: assets.to_string_lossy().into_owned(),
            ..default()
        },
        TransformPlugin,
        CapturePlugin::new(VideoProfile::PREVIEW, STEP),
    ))
    .init_asset::<AudioSource>();
    video_start(app.world_mut(), "bus");
    app.update();
    let handle: Handle<AudioSource> = app.world().resource::<AssetServer>().load("beep.wav");
    let world = app
        .world_mut()
        .spawn((
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::ONCE,
            Bus::World,
        ))
        .id();
    let ui = app
        .world_mut()
        .spawn((AudioPlayer::new(handle), PlaybackSettings::ONCE, Bus::Ui))
        .id();
    app.update();
    app.world_mut().despawn(ui);
    app.update();
    let buses: Vec<(u64, &'static str, LoggedBus)> = audio_events(app.world())
        .into_iter()
        .map(|event| match event {
            AudioEvent::Start { voice, bus, .. } => (voice, "start", bus),
            AudioEvent::Retune { voice, bus, .. } => (voice, "retune", bus),
            AudioEvent::Stop { voice, bus, .. } => (voice, "stop", bus),
        })
        .collect();
    assert!(buses.contains(&(world.to_bits(), "start", LoggedBus::World)));
    assert!(buses.contains(&(ui.to_bits(), "start", LoggedBus::Ui)));
    assert!(buses.contains(&(ui.to_bits(), "stop", LoggedBus::Ui)));
    assert_eq!(buses.len(), 3, "{buses:?}");

    let staging = video_staging(app.world()).unwrap();
    std::fs::remove_dir_all(staging).expect("remove test staging");
    std::fs::remove_dir_all(assets).expect("remove test assets");
}

#[test]
fn dropping_an_open_recording_removes_its_staging() {
    let mut app = capture_app();
    video_start(app.world_mut(), "dropped");
    app.update();
    let staging = video_staging(app.world()).unwrap();
    assert!(staging.join(AUDIO_LOG).is_file());
    drop(app);
    assert!(!staging.exists());
}

#[test]
fn a_recording_still_open_at_exit_turns_the_exit_into_an_error() {
    for exit in [AppExit::Success, AppExit::error()] {
        let mut app = capture_app();
        video_start(app.world_mut(), "unfinished");
        app.update();
        let staging = video_staging(app.world()).unwrap();
        app.world_mut().write_message(exit);
        video_close_at_exit(app.world_mut());

        assert_eq!(app.should_exit(), Some(AppExit::error()));
        assert!(app.world().resource::<CaptureState>().failed().is_some());
        assert!(!video_written(app.world(), "unfinished"));
        assert_eq!(video_staging(app.world()), None);
        assert!(!staging.exists());
    }
}

#[test]
fn closing_at_exit_without_an_open_recording_keeps_the_exit() {
    let mut app = capture_app();
    app.world_mut().write_message(AppExit::Success);
    video_close_at_exit(app.world_mut());
    assert_eq!(app.should_exit(), Some(AppExit::Success));
    assert!(app.world().resource::<CaptureState>().failed().is_none());
}
