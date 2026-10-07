use bevy::{
    app::{App, Main},
    ecs::schedule::{ScheduleCleanupPolicy, Schedules},
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
