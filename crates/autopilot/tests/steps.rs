use std::sync::Arc;

use autopilot::{frames, resource_where, AutopilotPlugin, AutopilotStatus, CompletionGates};
use bevy::prelude::*;

#[derive(Resource, Default)]
struct Counter(u32);

fn app(plugin: AutopilotPlugin) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, plugin));
    app
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
        .expect(resource_where::<Counter>(|counter| counter.0 == 1))
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
        .expect(resource_where::<Counter>(|_| true))
        .add());
    failed.update();
    assert!(matches!(failed.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Failed(reason) if reason.contains("bad score")));

    let mut timed_out = app(AutopilotPlugin::new()
        .step("never")
        .until(Arc::new(|_| false), 0.000001)
        .add());
    timed_out.update();
    std::thread::sleep(std::time::Duration::from_millis(2));
    timed_out.update();
    assert!(matches!(timed_out.world().resource::<AutopilotStatus>(),
        AutopilotStatus::Failed(reason) if reason.contains("never") && reason.contains("timed out")));

    let mut gated = app(AutopilotPlugin::new().step("generic gate").add());
    gated
        .world_mut()
        .resource_mut::<CompletionGates>()
        .register();
    gated.update();
    assert_eq!(
        gated.world().resource::<AutopilotStatus>(),
        &AutopilotStatus::WaitingForCollectors
    );
    gated.world_mut().resource_mut::<CompletionGates>().finish();
    gated.update();
    assert_eq!(
        gated.world().resource::<AutopilotStatus>(),
        &AutopilotStatus::Success
    );

    for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(
            std::panic::catch_unwind(|| AutopilotPlugin::new().with_deadline_secs(invalid))
                .is_err()
        );
    }
}
