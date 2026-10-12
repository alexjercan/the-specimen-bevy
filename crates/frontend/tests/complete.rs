use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy, window::CursorGrabMode};
use game_flow::GameState;
use gameplay::{
    facility::{DoorLock, Escaped, ExitDoor, FacilityPower, LevelRoot, Room},
    fuses::{FuseInventory, FusePanel, InstallFuses, FUSE_COUNT},
    player::PlayerController,
};

mod common;

use common::*;

fn escape_facility(app: &mut App) {
    let player = player(app);
    let panel = single::<With<FusePanel>>(app);
    app.world_mut()
        .entity_mut(player)
        .insert(FuseInventory(FUSE_COUNT));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.world_mut()
        .write_message(InstallFuses { player, panel });
    app.update();
    let exit = single::<With<ExitDoor>>(app);
    assert!(app.world().get::<DoorLock>(exit).is_some());
    for _ in 0..15 {
        app.update();
    }
    assert!(app.world().get::<DoorLock>(exit).is_none());
    app.insert_resource(TimeUpdateStrategy::Automatic);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(0.0, 1.6, -31.6);
    for _ in 0..3 {
        app.update();
    }
}

fn settle(app: &mut App) {
    run_for(app, REVEAL_AT + 0.5);
}

fn buttons(app: &mut App) -> usize {
    count::<With<Button>>(app)
}

fn assert_completion_scene(app: &mut App) {
    assert_eq!(game_state(app), GameState::Complete);
    assert_eq!(count::<With<PlayerController>>(app), 0);
    assert_eq!(count::<With<FusePanel>>(app), 0);
    assert_eq!(count::<With<ExitDoor>>(app), 0);
    assert_eq!(count::<With<LevelRoot>>(app), 1);
    assert_eq!(count::<With<Room>>(app), 2);
    assert_eq!(count::<With<Camera2d>>(app), 1);
    assert_eq!(count::<With<Camera3d>>(app), 1);
    assert_eq!(buttons(app), 0);
}

fn assert_fresh_run(app: &mut App) {
    assert_eq!(game_state(app), GameState::Playing);
    let power = app.world().resource::<FacilityPower>();
    assert!(power.is_on() && power.outage_in().is_some());
    assert_eq!(count::<With<Room>>(app), 26);
    let player = player(app);
    assert_eq!(app.world().get::<FuseInventory>(player).unwrap().0, 0);
    assert!(app.world().get::<Escaped>(player).is_none());
    let exit = single::<With<ExitDoor>>(app);
    assert!(app.world().get::<DoorLock>(exit).is_some());
    let panel = single::<With<FusePanel>>(app);
    assert_eq!(app.world().get::<FusePanel>(panel).unwrap().installed, 0);
}

#[test]
fn escaping_shows_the_completion_cinematic_then_credits_and_retry_starts_a_fresh_run() {
    let mut app = ready_menu_app();
    press(&mut app, "Play button");
    assert_fresh_run(&mut app);

    escape_facility(&mut app);
    assert_completion_scene(&mut app);
    assert_eq!(named_count(&mut app, "Complete screen"), 1);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));

    settle(&mut app);
    assert_eq!(game_state(&app), GameState::Credits);
    assert_eq!(buttons(&mut app), 3);
    press(&mut app, "Main menu button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 1);

    press(&mut app, "Play button");
    assert_fresh_run(&mut app);
    escape_facility(&mut app);
    assert_completion_scene(&mut app);
    settle(&mut app);
    assert_eq!(game_state(&app), GameState::Credits);
    press(&mut app, "Retry button");
    assert_fresh_run(&mut app);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
    assert_eq!(count::<With<Camera2d>>(&mut app), 0);
    assert_eq!(buttons(&mut app), 0);

    escape_facility(&mut app);
    assert_completion_scene(&mut app);
    settle(&mut app);
    press(&mut app, "Main menu button");
    press(&mut app, "Play button");
    assert_fresh_run(&mut app);
    app.update();
    assert_eq!(game_state(&app), GameState::Playing);
}

#[test]
fn quit_from_the_ending_credits_requests_app_exit() {
    let mut app = ready_menu_app();
    press(&mut app, "Play button");
    escape_facility(&mut app);
    assert_eq!(game_state(&app), GameState::Complete);
    settle(&mut app);
    press(&mut app, "Quit button");
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}
