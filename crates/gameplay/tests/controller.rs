use std::time::Duration;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    time::TimeUpdateStrategy,
    window::{CursorOptions, PrimaryWindow},
};
use gameplay::controller::{
    ControllerPlugin, ControllerState, PlayerController, DOWN_KEY, HOLD_KEY, LOOK_BUTTON,
    MOVE_SPEED, PITCH_LIMIT, RELEASE_KEY, RESUME_BUTTON, UP_KEY,
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
    let camera = app
        .world_mut()
        .spawn((PlayerController, Transform::from_translation(EYE)))
        .id();
    app.update();
    app.update();
    (app, camera)
}

fn step() -> f32 {
    MOVE_SPEED * STEP.as_secs_f32()
}

fn state(app: &App) -> ControllerState {
    *app.world().resource::<ControllerState>()
}

fn click(app: &mut App, button: MouseButton) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(button);
    app.update();
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.release(button);
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

fn sweep(app: &mut App, delta: Vec2, buttons: &[MouseButton]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    for button in buttons {
        input.press(*button);
    }
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = delta;
    app.update();
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = Vec2::ZERO;
    let mut input = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    input.release_all();
    input.clear();
}

fn turn(app: &mut App, delta: Vec2) {
    sweep(app, delta, &[LOOK_BUTTON]);
}

fn transform(app: &App, camera: Entity) -> Transform {
    *app.world().get::<Transform>(camera).unwrap()
}

fn moved(app: &App, camera: Entity) -> Vec3 {
    transform(app, camera).translation - EYE
}

#[test]
fn w_flies_forward_before_any_click() {
    let (mut app, camera) = app();
    assert_eq!(state(&app), ControllerState::Active);
    hold_keys(&mut app, &[KeyCode::KeyW]);
    let moved = moved(&app, camera);
    assert!(moved.abs_diff_eq(Vec3::NEG_Z * step(), 1e-4), "{moved:?}");
}

#[test]
fn every_axis_moves_at_constant_speed() {
    for (key, direction) in [
        (KeyCode::KeyW, Vec3::NEG_Z),
        (KeyCode::KeyS, Vec3::Z),
        (KeyCode::KeyA, Vec3::NEG_X),
        (KeyCode::KeyD, Vec3::X),
        (UP_KEY, Vec3::Y),
        (DOWN_KEY, Vec3::NEG_Y),
    ] {
        let (mut app, camera) = app();
        for _ in 0..3 {
            hold_keys(&mut app, &[key]);
        }
        let moved = moved(&app, camera);
        let expected = direction * 3.0 * step();
        assert!(moved.abs_diff_eq(expected, 1e-4), "{key:?} {moved:?}");
    }
}

#[test]
fn diagonal_flight_is_not_faster() {
    let (mut app, camera) = app();
    hold_keys(&mut app, &[KeyCode::KeyS, KeyCode::KeyA, UP_KEY]);
    let moved = moved(&app, camera);
    let expected = Vec3::new(-1.0, 1.0, 1.0).normalize() * step();
    assert!(moved.abs_diff_eq(expected, 1e-4), "{moved:?}");
}

#[test]
fn opposite_keys_cancel() {
    let (mut app, camera) = app();
    hold_keys(
        &mut app,
        &[
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyA,
            KeyCode::KeyD,
            UP_KEY,
            DOWN_KEY,
        ],
    );
    assert_eq!(transform(&app, camera).translation, EYE);
}

#[test]
fn forward_follows_pitch_and_strafe_stays_level() {
    let (mut app, camera) = app();
    turn(&mut app, Vec2::new(0.0, 400.0));
    let rotation = transform(&app, camera).rotation;
    hold_keys(&mut app, &[KeyCode::KeyW]);
    let forward = moved(&app, camera);
    let expected = rotation * Vec3::NEG_Z * step();
    assert!(forward.abs_diff_eq(expected, 1e-4), "{forward:?}");
    assert!(forward.y < 0.0, "{forward:?}");
    hold_keys(&mut app, &[KeyCode::KeyD]);
    let strafe = moved(&app, camera) - forward;
    assert!(strafe.abs_diff_eq(Vec3::X * step(), 1e-4), "{strafe:?}");
}

#[test]
fn mouse_without_right_button_does_not_turn() {
    let (mut app, camera) = app();
    let before = transform(&app, camera);
    sweep(&mut app, Vec2::new(200.0, 100.0), &[]);
    sweep(&mut app, Vec2::new(200.0, 100.0), &[RESUME_BUTTON]);
    assert_eq!(transform(&app, camera), before);
}

#[test]
fn right_button_turns_yaw_and_pitch_without_roll() {
    let (mut app, camera) = app();
    turn(&mut app, Vec2::new(100.0, -50.0));
    let (yaw, pitch, roll) = transform(&app, camera).rotation.to_euler(EulerRot::YXZ);
    assert!(yaw < 0.0, "{yaw}");
    assert!(pitch > 0.0, "{pitch}");
    assert!(roll.abs() < 1e-5, "{roll}");
}

#[test]
fn pitch_is_clamped() {
    let (mut app, camera) = app();
    turn(&mut app, Vec2::new(0.0, -100_000.0));
    let (_, pitch, _) = transform(&app, camera).rotation.to_euler(EulerRot::YXZ);
    assert!((pitch - PITCH_LIMIT).abs() < 1e-4, "{pitch}");
    turn(&mut app, Vec2::new(0.0, 100_000.0));
    let (_, pitch, _) = transform(&app, camera).rotation.to_euler(EulerRot::YXZ);
    assert!((pitch + PITCH_LIMIT).abs() < 1e-4, "{pitch}");
}

#[test]
fn escape_pauses_until_left_click() {
    let (mut app, camera) = app();
    tap(&mut app, RELEASE_KEY);
    assert_eq!(state(&app), ControllerState::Released);
    let before = transform(&app, camera);
    hold_keys(&mut app, &[KeyCode::KeyW, UP_KEY]);
    turn(&mut app, Vec2::new(200.0, 100.0));
    assert_eq!(transform(&app, camera), before);
    click(&mut app, LOOK_BUTTON);
    assert_eq!(state(&app), ControllerState::Released);
    click(&mut app, RESUME_BUTTON);
    assert_eq!(state(&app), ControllerState::Active);
    hold_keys(&mut app, &[KeyCode::KeyW]);
    assert_ne!(transform(&app, camera).translation, before.translation);
}

#[test]
fn f12_hold_blocks_click_until_second_f12() {
    let (mut app, camera) = app();
    tap(&mut app, HOLD_KEY);
    assert_eq!(state(&app), ControllerState::Held);
    let before = transform(&app, camera);
    click(&mut app, RESUME_BUTTON);
    tap(&mut app, RELEASE_KEY);
    assert_eq!(state(&app), ControllerState::Held);
    hold_keys(&mut app, &[KeyCode::KeyW]);
    turn(&mut app, Vec2::new(200.0, 100.0));
    assert_eq!(transform(&app, camera), before);
    tap(&mut app, HOLD_KEY);
    assert_eq!(state(&app), ControllerState::Released);
    click(&mut app, RESUME_BUTTON);
    assert_eq!(state(&app), ControllerState::Active);
}

#[test]
fn f12_while_released_holds() {
    let (mut app, _) = app();
    tap(&mut app, RELEASE_KEY);
    tap(&mut app, HOLD_KEY);
    assert_eq!(state(&app), ControllerState::Held);
}

#[test]
fn controller_never_touches_the_cursor() {
    let (mut app, _) = app();
    let window = app
        .world_mut()
        .spawn((PrimaryWindow, CursorOptions::default()))
        .id();
    app.update();
    let changed = |app: &App| {
        app.world()
            .entity(window)
            .get_ref::<CursorOptions>()
            .unwrap()
            .last_changed()
    };
    let before = changed(&app);
    click(&mut app, RESUME_BUTTON);
    turn(&mut app, Vec2::new(50.0, 20.0));
    tap(&mut app, RELEASE_KEY);
    click(&mut app, RESUME_BUTTON);
    tap(&mut app, HOLD_KEY);
    tap(&mut app, HOLD_KEY);
    assert_eq!(changed(&app), before);
}
