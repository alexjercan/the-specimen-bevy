use std::time::{Duration, Instant};

use bevy::{
    app::{App, Main},
    ecs::schedule::{ScheduleCleanupPolicy, Schedules},
    input::InputPlugin,
    prelude::{Entity, Local, Rect, Transform, Vec2, Vec3, With, World},
    time::{create_time_channels, Time, TimePlugin, TimeUpdateStrategy},
};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin},
    levels::{
        build_first_floor, Door, DoorLock, DoorPlugin, DoorState, Escaped, ExitDoor, FuseInventory,
        FusePanel, FusePlugin, ObjectivePlugin, Room, FUSE_COUNT,
    },
};

use super::{discard_render_times, pace, run_main, TransportTimeline, FRAME_TIME};
use crate::transport::{snapshot, Command};

fn rendered_frame(world: &mut World, started: Local<bool>) {
    run_main(world, started);
}

#[test]
fn rendered_main_runs_exit_detection_and_advances_one_numbered_tick() {
    let mut app = App::new();
    app.add_plugins((bevy::MinimalPlugins, ObjectivePlugin))
        .init_resource::<TransportTimeline>();
    app.world_mut()
        .spawn(Room(Rect::new(-1.25, -31.25, 8.75, -21.25)));
    app.world_mut().spawn((
        Door {
            position: Vec2::new(0.0, -31.25),
            rotation: Default::default(),
            frame: String::new(),
            panel: String::new(),
            state: DoorState::Open,
        },
        ExitDoor,
    ));
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_translation(Vec3::new(0.0, 1.6, -31.5)),
        ))
        .id();
    let mut main = app
        .world_mut()
        .resource_mut::<Schedules>()
        .remove(Main)
        .unwrap();
    main.remove_systems_in_set(
        Main::run_main,
        app.world_mut(),
        ScheduleCleanupPolicy::RemoveSetAndSystems,
    )
    .unwrap();
    main.add_systems(rendered_frame);
    app.world_mut().resource_mut::<Schedules>().reinsert(main);
    app.world_mut().resource_mut::<TransportTimeline>().ready();

    app.update();
    assert_eq!(app.world().resource::<TransportTimeline>().tick(), Some(0));
    assert!(app.world().entity(player).contains::<Escaped>());
    let mut escaped = app.world_mut().query_filtered::<Entity, With<Escaped>>();
    assert_eq!(escaped.iter(app.world()).count(), 1);
    app.update();
    assert_eq!(app.world().resource::<TransportTimeline>().tick(), Some(1));
}

#[test]
fn rendered_main_installs_fuses_opens_exit_and_reports_win_after_crossing() {
    let mut app = App::new();
    app.add_plugins((
        bevy::MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        DoorPlugin,
        FusePlugin,
        ObjectivePlugin,
    ))
    .init_resource::<TransportTimeline>()
    .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME_TIME));
    app.finish();
    app.cleanup();
    app.world_mut()
        .run_system_cached(build_first_floor)
        .unwrap();
    let world = app.world_mut();
    let exit = world
        .query_filtered::<Entity, With<ExitDoor>>()
        .single(world)
        .unwrap();
    let panel = world
        .query_filtered::<Entity, With<FusePanel>>()
        .single(world)
        .unwrap();
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            FuseInventory(FUSE_COUNT),
            Transform::from_xyz(5.0, 1.6, -29.5).looking_at(Vec3::new(5.0, 1.85, -31.15), Vec3::Y),
        ))
        .id();
    let mut main = app
        .world_mut()
        .resource_mut::<Schedules>()
        .remove(Main)
        .unwrap();
    main.remove_systems_in_set(
        Main::run_main,
        app.world_mut(),
        ScheduleCleanupPolicy::RemoveSetAndSystems,
    )
    .unwrap();
    main.add_systems(rendered_frame);
    app.world_mut().resource_mut::<Schedules>().reinsert(main);
    app.world_mut().resource_mut::<TransportTimeline>().ready();
    app.update();
    assert_eq!(app.world().resource::<TransportTimeline>().tick(), Some(0));
    for (target, controls) in [
        (1, r#"{"f":true}"#),
        (2, r#"{"f":false}"#),
        (3, r#"{"f":true}"#),
        (4, r#"{"f":false}"#),
        (40, r#"{}"#),
        (100, r#"{"w":true}"#),
    ] {
        let command: Command =
            serde_json::from_str(&format!(r#"{{"tick":{target},"input":{controls}}}"#)).unwrap();
        command.input.apply(app.world_mut());
        while app.world().resource::<TransportTimeline>().tick() != Some(target) {
            app.update();
        }
        if target == 2 {
            assert!(app.world().get::<DoorLock>(exit).is_none());
            app.world_mut().entity_mut(player).insert(
                Transform::from_xyz(0.0, 1.6, -30.0)
                    .looking_at(Vec3::new(0.0, 1.1, -31.25), Vec3::Y),
            );
        }
        if target == 4 {
            assert_eq!(
                app.world().get::<FusePanel>(panel).unwrap().installed,
                FUSE_COUNT
            );
        }
        if target == 40 {
            assert_eq!(
                app.world().get::<Door>(exit).unwrap().state,
                DoorState::Open
            );
            assert!(app.world().get::<Escaped>(player).is_none());
            assert!(app.world().get::<Transform>(player).unwrap().translation.z > -31.25);
        }
    }
    assert_eq!(
        app.world().get::<Door>(exit).unwrap().state,
        DoorState::Open
    );
    assert!(
        app.world().get::<Escaped>(player).is_some(),
        "player at {:?}",
        app.world().get::<Transform>(player).unwrap().translation
    );
    let reply: serde_json::Value = serde_json::from_str(&snapshot(app.world_mut(), 100)).unwrap();
    assert_eq!(reply["won"], true);
}

#[test]
fn rendered_requests_do_not_advance_faster_than_frame_time() {
    let mut next = Instant::now() + FRAME_TIME;
    let start = Instant::now();
    pace(&mut next);
    assert!(start.elapsed() >= FRAME_TIME.saturating_sub(Duration::from_millis(1)));
}

#[test]
fn render_times_are_consumed_even_without_main_updates() {
    let (sender, receiver) = create_time_channels();
    let worker = discard_render_times(receiver);
    let deadline = Instant::now() + Duration::from_secs(2);

    for _ in 0..32 {
        loop {
            if sender.0.try_send(Instant::now()).is_ok() {
                break;
            }
            assert!(Instant::now() < deadline, "render times stopped draining");
            std::thread::yield_now();
        }
    }
    drop(sender);
    worker.join().unwrap();
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
