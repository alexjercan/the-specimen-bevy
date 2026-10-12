use std::time::Duration;

use bevy::prelude::*;
use gameplay::{
    devices::{DevicePlugin, Flashbangs, Flashed},
    facility::FacilityPowerPlugin,
    player::{ControllerSettings, Flashlight, Input, PlayerController},
};

mod common;

fn app(flashlight: Input, flashbang: Input) -> (App, Entity) {
    let mut settings = common::controller_settings();
    settings.keys.flashlight = flashlight;
    settings.keys.flashbang = flashbang;
    let mut app = common::player_app(Duration::from_millis(100));
    app.insert_resource(settings)
        .add_plugins((FacilityPowerPlugin, DevicePlugin));
    app.finish();
    app.cleanup();
    app.update();
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, 1.6, 0.0),
            Flashbangs(2),
        ))
        .id();
    app.update();
    (app, player)
}

fn tap<T: Copy + Eq + std::hash::Hash + Send + Sync + 'static>(app: &mut App, input: T) {
    app.world_mut()
        .resource_mut::<ButtonInput<T>>()
        .press(input);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<T>>()
        .release(input);
    app.update();
}

fn flashlight_on(app: &App, player: Entity) -> bool {
    app.world().get::<Flashlight>(player).unwrap().on
}

fn flashbangs(app: &App, player: Entity) -> usize {
    app.world().get::<Flashbangs>(player).unwrap().0
}

#[test]
fn default_bindings_keep_left_and_right_mouse() {
    let (mut app, player) = app(
        Input::Mouse(MouseButton::Left),
        Input::Mouse(MouseButton::Right),
    );
    tap(&mut app, MouseButton::Left);
    assert!(flashlight_on(&app, player));
    tap(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), 1);
    assert!(app.world().get::<Flashed>(player).is_none());
}

#[test]
fn rebound_keys_drive_flashlight_and_flashbang() {
    let (mut app, player) = app(Input::Key(KeyCode::KeyH), Input::Key(KeyCode::KeyG));
    tap(&mut app, MouseButton::Left);
    tap(&mut app, MouseButton::Right);
    assert!(!flashlight_on(&app, player));
    assert_eq!(flashbangs(&app, player), 2);

    tap(&mut app, KeyCode::KeyH);
    assert!(flashlight_on(&app, player));
    tap(&mut app, KeyCode::KeyG);
    assert_eq!(flashbangs(&app, player), 1);
}

#[test]
fn swapped_mouse_buttons_follow_the_settings() {
    let (mut app, player) = app(
        Input::Mouse(MouseButton::Right),
        Input::Mouse(MouseButton::Left),
    );
    tap(&mut app, MouseButton::Left);
    assert!(!flashlight_on(&app, player));
    assert_eq!(flashbangs(&app, player), 1);
    tap(&mut app, MouseButton::Right);
    assert!(flashlight_on(&app, player));
    assert_eq!(flashbangs(&app, player), 1);
}

#[test]
fn changing_settings_rebinds_at_runtime() {
    let (mut app, player) = app(
        Input::Mouse(MouseButton::Left),
        Input::Mouse(MouseButton::Right),
    );
    app.world_mut()
        .resource_mut::<ControllerSettings>()
        .keys
        .flashbang = Input::Key(KeyCode::KeyG);
    app.update();
    tap(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), 2);
    tap(&mut app, KeyCode::KeyG);
    assert_eq!(flashbangs(&app, player), 1);
}

#[test]
fn held_rebound_flashlight_key_does_not_toggle_on_spawn() {
    let mut settings = common::controller_settings();
    settings.keys.flashlight = Input::Key(KeyCode::KeyH);
    let mut app = common::player_app(Duration::from_millis(100));
    app.insert_resource(settings);
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyH);
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)))
        .id();
    app.update();
    app.update();
    assert!(!flashlight_on(&app, player));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyH);
    app.update();
    tap(&mut app, KeyCode::KeyH);
    assert!(flashlight_on(&app, player));
}
