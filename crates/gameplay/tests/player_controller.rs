use std::time::Duration;

use bevy::{
    audio::SpatialListener,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use game_settings::GameSettingsPlugin;
use gameplay::player::{
    ControllerSettings, Flashlight, Input, PlayerController, PlayerControlsEnabled, PlayerInput,
    PlayerViewPlugin, RUN_SPEED, WALK_SPEED,
};

mod common;

fn app(camera: bool) -> (App, Entity) {
    let mut app = common::player_app(Duration::from_millis(100));
    if camera {
        app.add_plugins((GameSettingsPlugin { persist: false }, PlayerViewPlugin));
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
    let listener = app.world().get::<SpatialListener>(player).unwrap();
    assert!(listener.left_ear_offset.x > 0.0);
    assert!(listener.right_ear_offset.x < 0.0);
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
fn changed_movement_binding_replaces_wasd_action() {
    let (mut app, player) = app(false);
    let mut settings = common::controller_settings();
    settings.keys.forward = KeyCode::ArrowUp;
    app.insert_resource(settings);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.update();
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation.z,
        0.0
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowUp);
    app.update();
    assert!(app.world().get::<Transform>(player).unwrap().translation.z < 0.0);
}

#[test]
fn changing_flashlight_binding_replaces_mouse_action() {
    let (mut app, player) = app(false);
    let mut settings = common::controller_settings();
    settings.keys.flashlight = Input::Key(KeyCode::KeyQ);
    app.insert_resource(settings);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert!(!app.world().get::<Flashlight>(player).unwrap().on);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyQ);
    app.update();
    assert!(app.world().get::<Flashlight>(player).unwrap().on);
}

#[test]
fn mouse_look_uses_controller_sensitivity() {
    let (mut app, player) = app(false);
    app.world_mut()
        .resource_mut::<ControllerSettings>()
        .mouse_sensitivity = 0.004;
    app.world_mut().write_message(MouseMotion {
        delta: Vec2::new(100.0, 0.0),
    });
    app.update();
    let (yaw, _, _) = app
        .world()
        .get::<Transform>(player)
        .unwrap()
        .rotation
        .to_euler(EulerRot::YXZ);
    assert!((yaw + 0.4).abs() < 0.001);
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
