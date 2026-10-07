use std::time::Duration;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    time::TimeUpdateStrategy,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use gameplay::controller::{
    ControllerPlugin, CursorHold, PlayerController, HOLD_KEY, LOCK_BUTTON, MOVE_SPEED, PITCH_LIMIT,
    RELEASE_KEY,
};

const STEP: Duration = Duration::from_millis(100);
const EYE: Vec3 = Vec3::new(2.0, 1.6, -3.0);

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(STEP))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<AccumulatedMouseMotion>()
        .add_plugins(ControllerPlugin);
    app.world_mut()
        .spawn((PrimaryWindow, CursorOptions::default()));
    let camera = app
        .world_mut()
        .spawn((PlayerController, Transform::from_translation(EYE)))
        .id();
    app.update();
    app.update();
    (app, camera)
}

fn cursor(app: &mut App) -> CursorOptions {
    app.world_mut()
        .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap()
        .clone()
}

fn locked(app: &mut App) -> bool {
    let cursor = cursor(app);
    match cursor.grab_mode {
        CursorGrabMode::Locked => {
            assert!(!cursor.visible);
            true
        }
        CursorGrabMode::None => {
            assert!(cursor.visible);
            false
        }
        CursorGrabMode::Confined => panic!("unexpected confined cursor"),
    }
}

fn click(app: &mut App) {
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.press(LOCK_BUTTON);
    app.update();
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.release(LOCK_BUTTON);
    buttons.clear();
}

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(key);
    keys.clear();
}

fn hold_keys(app: &mut App, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    for key in keys {
        input.press(*key);
    }
    app.update();
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.release_all();
    input.clear();
}

fn turn(app: &mut App, delta: Vec2) {
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = delta;
    app.update();
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = Vec2::ZERO;
}

fn transform(app: &App, camera: Entity) -> Transform {
    *app.world().get::<Transform>(camera).unwrap()
}

#[test]
fn cursor_starts_free_and_click_locks_it() {
    let (mut app, _) = app();
    assert!(!locked(&mut app));
    click(&mut app);
    assert!(locked(&mut app));
}

#[test]
fn click_does_not_lock_without_a_controller() {
    let (mut app, camera) = app();
    app.world_mut().despawn(camera);
    click(&mut app);
    assert!(!locked(&mut app));
}

#[test]
fn escape_releases_and_click_relocks() {
    let (mut app, _) = app();
    click(&mut app);
    tap(&mut app, RELEASE_KEY);
    assert!(!locked(&mut app));
    assert!(!app.world().resource::<CursorHold>().0);
    click(&mut app);
    assert!(locked(&mut app));
}

#[test]
fn f12_hold_releases_and_blocks_click_until_second_f12() {
    let (mut app, _) = app();
    click(&mut app);
    tap(&mut app, HOLD_KEY);
    assert!(app.world().resource::<CursorHold>().0);
    assert!(!locked(&mut app));
    click(&mut app);
    assert!(!locked(&mut app));
    tap(&mut app, HOLD_KEY);
    assert!(!app.world().resource::<CursorHold>().0);
    assert!(!locked(&mut app));
    click(&mut app);
    assert!(locked(&mut app));
}

#[test]
fn input_is_ignored_while_cursor_is_free() {
    let (mut app, camera) = app();
    let before = transform(&app, camera);
    hold_keys(&mut app, &[KeyCode::KeyW, KeyCode::KeyD]);
    turn(&mut app, Vec2::new(200.0, 100.0));
    assert_eq!(transform(&app, camera), before);
}

#[test]
fn w_walks_forward_at_constant_speed() {
    let (mut app, camera) = app();
    click(&mut app);
    hold_keys(&mut app, &[KeyCode::KeyW]);
    let moved = transform(&app, camera).translation - EYE;
    let expected = Vec3::NEG_Z * MOVE_SPEED * STEP.as_secs_f32();
    assert!(moved.abs_diff_eq(expected, 1e-4), "{moved:?}");
}

#[test]
fn diagonal_walk_is_not_faster() {
    let (mut app, camera) = app();
    click(&mut app);
    hold_keys(&mut app, &[KeyCode::KeyS, KeyCode::KeyA]);
    let moved = transform(&app, camera).translation - EYE;
    let expected = Vec3::new(-1.0, 0.0, 1.0).normalize() * MOVE_SPEED * STEP.as_secs_f32();
    assert!(moved.abs_diff_eq(expected, 1e-4), "{moved:?}");
}

#[test]
fn opposite_keys_cancel() {
    let (mut app, camera) = app();
    click(&mut app);
    hold_keys(
        &mut app,
        &[KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD],
    );
    assert_eq!(transform(&app, camera).translation, EYE);
}

#[test]
fn walking_while_looking_down_keeps_eye_height() {
    let (mut app, camera) = app();
    click(&mut app);
    turn(&mut app, Vec2::new(300.0, 400.0));
    for _ in 0..5 {
        hold_keys(&mut app, &[KeyCode::KeyW, KeyCode::KeyD]);
    }
    let after = transform(&app, camera);
    assert_eq!(after.translation.y, EYE.y);
    let travelled = (after.translation - EYE).length();
    let expected = 5.0 * MOVE_SPEED * STEP.as_secs_f32();
    assert!((travelled - expected).abs() < 1e-4, "{travelled}");
}

#[test]
fn mouse_turns_yaw_and_pitch_without_roll() {
    let (mut app, camera) = app();
    click(&mut app);
    turn(&mut app, Vec2::new(100.0, -50.0));
    let (yaw, pitch, roll) = transform(&app, camera).rotation.to_euler(EulerRot::YXZ);
    assert!(yaw < 0.0, "{yaw}");
    assert!(pitch > 0.0, "{pitch}");
    assert!(roll.abs() < 1e-5, "{roll}");
}

#[test]
fn pitch_is_clamped() {
    let (mut app, camera) = app();
    click(&mut app);
    turn(&mut app, Vec2::new(0.0, -100_000.0));
    let (_, pitch, _) = transform(&app, camera).rotation.to_euler(EulerRot::YXZ);
    assert!((pitch - PITCH_LIMIT).abs() < 1e-4, "{pitch}");
    turn(&mut app, Vec2::new(0.0, 100_000.0));
    let (_, pitch, _) = transform(&app, camera).rotation.to_euler(EulerRot::YXZ);
    assert!((pitch + PITCH_LIMIT).abs() < 1e-4, "{pitch}");
}

#[test]
fn redundant_release_does_not_touch_cursor() {
    let (mut app, _) = app();
    let changed = |app: &mut App| {
        app.world_mut()
            .query_filtered::<Ref<CursorOptions>, With<PrimaryWindow>>()
            .single(app.world())
            .unwrap()
            .last_changed()
    };
    let before = changed(&mut app);
    tap(&mut app, RELEASE_KEY);
    tap(&mut app, HOLD_KEY);
    tap(&mut app, HOLD_KEY);
    assert_eq!(changed(&mut app), before);
    click(&mut app);
    assert_ne!(changed(&mut app), before);
}
