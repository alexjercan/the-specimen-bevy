use std::time::Duration;

use bevy::{
    input::{mouse::MouseMotion, InputPlugin},
    prelude::*,
    time::TimeUpdateStrategy,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::controller::{
    PlayerController, PlayerControllerPlugin, PlayerControlsEnabled, PlayerInput, RUN_SPEED,
    WALK_SPEED,
};

fn app(camera: bool) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    if camera {
        app.add_plugins(PlayerControllerPlugin::default());
    } else {
        app.add_plugins(PlayerControllerPlugin::default().without_camera());
    }
    app.finish();
    app.cleanup();
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)))
        .id();
    app.update();
    (app, player)
}

#[test]
fn player_spawns_with_camera_and_wasd_input() {
    let (app, player) = app(true);
    assert!(app.world().get::<Camera3d>(player).is_some());
    assert!(app.world().get::<PlayerInput>(player).is_some());
}

#[test]
fn camera_player_captures_cursor_and_pause_releases_it() {
    let (mut app, player) = app(true);
    let window = app
        .world_mut()
        .spawn((PrimaryWindow, CursorOptions::default()))
        .id();
    app.update();
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert!(!cursor.visible);
    assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    app.update();
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert!(cursor.visible);
    assert_eq!(cursor.grab_mode, CursorGrabMode::None);
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = true;
    app.update();
    assert!(!app.world().get::<CursorOptions>(window).unwrap().visible);
    app.world_mut().despawn(player);
    app.update();
    assert!(app.world().get::<CursorOptions>(window).unwrap().visible);
}

#[test]
fn headless_player_has_input_and_pose_but_no_camera() {
    let (app, player) = app(false);
    assert!(app.world().get::<Camera3d>(player).is_none());
    assert!(app.world().get::<PlayerInput>(player).is_some());
    assert!(app.world().get::<Transform>(player).is_some());
}

#[test]
fn wasd_shift_and_release_move_at_fixed_height() {
    let (mut app, player) = app(false);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.update();
    let transform = app.world().get::<Transform>(player).unwrap();
    assert!((transform.translation.z + WALK_SPEED * 0.1).abs() < 0.01);
    assert_eq!(transform.translation.y, 1.6);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    app.update();
    assert!(
        (app.world().get::<Transform>(player).unwrap().translation.z
            + (WALK_SPEED + RUN_SPEED) * 0.1)
            .abs()
            < 0.01
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyW);
    app.update();
    assert!(
        (app.world().get::<Transform>(player).unwrap().translation.z
            + (WALK_SPEED + RUN_SPEED) * 0.1)
            .abs()
            < 0.01
    );
}

#[test]
fn mouse_look_needs_no_button_and_does_not_change_height() {
    let (mut app, player) = app(false);
    app.world_mut().write_message(MouseMotion {
        delta: Vec2::new(100.0, 50.0),
    });
    app.update();
    let transform = app.world().get::<Transform>(player).unwrap();
    let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
    assert!(yaw < -0.1);
    assert!(pitch < -0.05);
    assert!(roll.abs() < 0.001);
    assert_eq!(transform.translation.y, 1.6);
}
