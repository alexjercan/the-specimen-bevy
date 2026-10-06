use super::*;

#[test]
fn capture_plugin_initializes_screenshot_and_shared_state() {
    let mut app = App::new();
    app.add_plugins(CapturePlugin::new(30));
    assert!(app.world().contains_resource::<CaptureState>());
    assert!(app.world().contains_resource::<screenshot::CaptureLog>());
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
