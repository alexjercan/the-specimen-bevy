use std::time::Duration;

use bevy::{
    app::{App, Main},
    ecs::schedule::{ScheduleCleanupPolicy, Schedules},
    time::{Time, TimePlugin, TimeUpdateStrategy},
};

#[test]
fn removing_main_runner_keeps_schedules_available_during_initialization() {
    let mut app = App::new();
    let mut main = app
        .world_mut()
        .resource_mut::<Schedules>()
        .remove(Main)
        .unwrap();
    let removed = main
        .remove_systems_in_set(
            Main::run_main,
            app.world_mut(),
            ScheduleCleanupPolicy::RemoveSetAndSystems,
        )
        .unwrap();
    assert_eq!(removed, 1);
    app.world_mut().resource_mut::<Schedules>().reinsert(main);
    assert!(app.world().resource::<Schedules>().contains(Main));
}

#[test]
fn manual_duration_does_not_require_a_render_time_receiver() {
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        50,
    )));
    app.update();
    app.update();
    app.update();
    assert_eq!(
        app.world().resource::<Time>().elapsed(),
        Duration::from_millis(100)
    );
}
