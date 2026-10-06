use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use autopilot::{frames, resource_where, AutopilotClock, AutopilotPlugin, AutopilotStatus};
use bevy::prelude::*;

#[derive(Resource, Default)]
struct Counter(u32);

fn app(plugin: AutopilotPlugin) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, plugin));
    app
}

#[test]
fn condition_is_checked_once_per_frame_after_actions() {
    let calls = Arc::new(AtomicUsize::new(0));
    let checks = Arc::clone(&calls);
    let mut app = app(AutopilotPlugin::new()
        .step("wait for update")
        .act(|world| world.insert_resource(Counter(1)))
        .until(
            Arc::new(move |_| {
                checks.fetch_add(1, Ordering::SeqCst);
                false
            }),
            30.0,
        )
        .add());
    app.update();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        *app.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Running
    );
    app.update();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        *app.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Running
    );
}

#[test]
fn next_step_resets_clock_without_actions() {
    let mut app = app(AutopilotPlugin::new()
        .step("first")
        .until(frames(1), 30.0)
        .add()
        .step("second")
        .until(frames(1), 30.0)
        .add());
    app.update();
    app.update();
    assert_eq!(app.world().resource::<AutopilotClock>().step_frames, 1);
    app.update();
    assert_eq!(app.world().resource::<AutopilotClock>().step_frames, 0);
    assert_eq!(
        *app.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Running
    );
    app.update();
    assert_eq!(
        *app.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Success
    );
}

#[test]
fn run_timeout_fails_pending_step() {
    let mut app = app(AutopilotPlugin::new()
        .with_deadline_secs(0.000001)
        .step("pending step")
        .until(Arc::new(|_| false), 30.0)
        .add());
    app.update();
    std::thread::sleep(std::time::Duration::from_millis(2));
    app.update();
    assert!(matches!(app.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Failed(reason) if reason.contains("run deadline expired at `pending step`")));
}

#[test]
fn completion_on_deadline_frame_does_not_run_timeout() {
    let mut app = app(AutopilotPlugin::new()
        .with_deadline_secs(0.000001)
        .step("finish")
        .until(frames(1), 30.0)
        .add());
    app.update();
    std::thread::sleep(std::time::Duration::from_millis(2));
    app.update();
    assert_eq!(
        *app.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Success
    );
}

#[test]
fn status_changes_emit_exit_in_update() {
    let mut successful = app(AutopilotPlugin::new().step("done").add());
    let mut success_cursor = successful
        .world()
        .resource::<Messages<AppExit>>()
        .get_cursor_current();
    successful.update();
    assert_eq!(
        success_cursor
            .read(successful.world().resource::<Messages<AppExit>>())
            .cloned()
            .collect::<Vec<_>>(),
        vec![AppExit::Success]
    );
    successful.update();
    assert_eq!(
        success_cursor
            .read(successful.world().resource::<Messages<AppExit>>())
            .count(),
        0
    );

    let mut failed = app(AutopilotPlugin::new()
        .with_deadline_secs(0.000001)
        .step("pending")
        .until(Arc::new(|_| false), 30.0)
        .add());
    let mut failure_cursor = failed
        .world()
        .resource::<Messages<AppExit>>()
        .get_cursor_current();
    failed.update();
    std::thread::sleep(std::time::Duration::from_millis(2));
    failed.update();
    assert!(matches!(
        failed.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Failed(_)
    ));
    assert_eq!(
        failure_cursor
            .read(failed.world().resource::<Messages<AppExit>>())
            .count(),
        0
    );
    failed.update();
    assert_eq!(
        failure_cursor
            .read(failed.world().resource::<Messages<AppExit>>())
            .cloned()
            .collect::<Vec<_>>(),
        vec![AppExit::error()]
    );
}

#[test]
fn step_contract_and_failure_paths() {
    assert!(std::panic::catch_unwind(|| app(AutopilotPlugin::new())).is_err());
    assert!(std::panic::catch_unwind(|| app(AutopilotPlugin::new().step(" ").add())).is_err());
    assert!(std::panic::catch_unwind(|| app(AutopilotPlugin::new()
        .step("bad wait")
        .until(frames(1), 0.0)
        .add()))
    .is_err());

    let mut successful = app(AutopilotPlugin::new()
        .with_deadline_secs(30.0)
        .step("change resource")
        .act(|world| {
            world.insert_resource(Counter(1));
        })
        .until(resource_where::<Counter>(|counter| counter.0 == 1), 5.0)
        .add()
        .step("assert resource")
        .expect(
            resource_where::<Counter>(|counter| counter.0 == 1),
            "counter should be one",
        )
        .add());
    for _ in 0..4 {
        successful.update();
    }
    assert_eq!(
        successful.world().resource::<AutopilotStatus>(),
        &AutopilotStatus::Success
    );

    let mut failed = app(AutopilotPlugin::new()
        .step("bad score")
        .expect(resource_where::<Counter>(|_| true), "counter should exist")
        .add());
    failed.update();
    assert!(matches!(failed.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Failed(reason) if reason.contains("bad score") && reason.contains("counter should exist")));

    let mut timed_out = app(AutopilotPlugin::new()
        .step("never")
        .until(Arc::new(|_| false), 0.000001)
        .add());
    timed_out.update();
    std::thread::sleep(std::time::Duration::from_millis(2));
    timed_out.update();
    assert!(matches!(timed_out.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Failed(reason) if reason.contains("never") && reason.contains("timed out")));

    for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(
            std::panic::catch_unwind(|| AutopilotPlugin::new().with_deadline_secs(invalid))
                .is_err()
        );
    }
}
