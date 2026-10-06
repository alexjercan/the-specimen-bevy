use super::*;

#[test]
fn validates_profile_and_resolves_output() {
    assert_eq!(LoopCapturePlugin::default().fps, 30);
    assert!(std::panic::catch_unwind(|| LoopCapturePlugin::new(0)).is_err());
    assert_eq!(loop_path("sample"), PathBuf::from("sample.webm"));
    let mut world = World::new();
    assert!(!loop_written_at("sample")(&world));
    world.init_resource::<Recorder>();
    world.resource_mut::<Recorder>().written = Some(loop_path("sample"));
    assert!(loop_written_at("sample")(&world));
    assert!(!loop_written_at("other")(&world));
    assert_eq!(
        loop_path("/tmp/sample.webm"),
        PathBuf::from("/tmp/sample.webm")
    );
}

#[test]
fn loop_start_accepts_existing_output_and_clears_prior_completion() {
    let mut world = World::new();
    world.insert_resource(Profile(LoopCapturePlugin::new(30)));
    world.init_resource::<Recorder>();
    world.init_resource::<CaptureState>();
    let dir = std::env::temp_dir().join(format!(
        "capture-loop-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir(&dir).expect("create test directory");
    let output = dir.join("existing.webm");
    std::fs::write(&output, b"old video").expect("create old video");
    world.resource_mut::<Recorder>().written = Some(output.clone());
    assert!(loop_written(&world, output.to_str().expect("UTF-8 path")));
    loop_start(&mut world, output.to_str().expect("UTF-8 path"));
    assert!(!loop_written(&world, output.to_str().expect("UTF-8 path")));
    assert_eq!(
        std::fs::read(&output).expect("read old video"),
        b"old video"
    );
    let staging = world
        .resource::<Recorder>()
        .active
        .as_ref()
        .expect("open loop")
        .staging
        .clone();
    std::fs::remove_dir_all(staging).expect("remove test staging");
    std::fs::remove_dir_all(dir).expect("remove test directory");
}

#[test]
fn audio_path_is_exposed_only_while_recording() {
    let mut world = World::new();
    world.insert_resource(Recorder::default());
    assert_eq!(loop_audio_path(&world), None);
    world.resource_mut::<Recorder>().active = Some(Recording {
        output: PathBuf::from("sample.webm"),
        staging: PathBuf::from("/tmp/sample-unique"),
        requested: 0,
        received: 0,
        closed: false,
    });
    assert_eq!(
        loop_audio_path(&world),
        Some(PathBuf::from("/tmp/sample-unique/audio.f32le"))
    );
}

#[test]
fn abort_reports_failure_without_releasing_capture_gate() {
    let mut world = World::new();
    world.init_resource::<CaptureState>();
    world.init_resource::<Recorder>();
    world.init_resource::<Messages<AppExit>>();
    world.resource_mut::<CaptureState>().register();
    abort(&mut world, "ffmpeg failed".into());
    assert_eq!(world.resource::<CaptureState>().pending(), 1);
    assert_eq!(
        world.resource::<CaptureState>().failed(),
        Some("ffmpeg failed")
    );
}
