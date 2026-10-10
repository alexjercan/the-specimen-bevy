use std::time::Duration;

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_settings::GameSettings;
use gameplay::{
    controller::{Flashlight, PlayerController, PlayerControllerPlugin},
    levels::{DevicePlugin, Flashbangs, Flashed},
};

fn app(flashlight: &str, flashbang: &str) -> (App, Entity) {
    let mut settings = GameSettings::default();
    settings.keys.flashlight = flashlight.into();
    settings.keys.flashbang = flashbang.into();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .insert_resource(settings)
        .add_plugins(PlayerControllerPlugin::default().without_camera())
        .add_plugins(DevicePlugin);
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
    let (mut app, player) = app("MouseLeft", "MouseRight");
    tap(&mut app, MouseButton::Left);
    assert!(flashlight_on(&app, player));
    tap(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), 1);
    assert!(app.world().get::<Flashed>(player).is_none());
}

#[test]
fn rebound_keys_drive_flashlight_and_flashbang() {
    let (mut app, player) = app("KeyH", "KeyG");
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
    let (mut app, player) = app("MouseRight", "MouseLeft");
    tap(&mut app, MouseButton::Left);
    assert!(!flashlight_on(&app, player));
    assert_eq!(flashbangs(&app, player), 1);
    tap(&mut app, MouseButton::Right);
    assert!(flashlight_on(&app, player));
    assert_eq!(flashbangs(&app, player), 1);
}

#[test]
fn changing_settings_rebinds_at_runtime() {
    let (mut app, player) = app("MouseLeft", "MouseRight");
    app.world_mut()
        .resource_mut::<GameSettings>()
        .keys
        .flashbang = "KeyG".into();
    app.update();
    tap(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), 2);
    tap(&mut app, KeyCode::KeyG);
    assert_eq!(flashbangs(&app, player), 1);
}

#[test]
fn held_rebound_flashlight_key_does_not_toggle_on_spawn() {
    let mut settings = GameSettings::default();
    settings.keys.flashlight = "KeyH".into();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(settings)
        .add_plugins(PlayerControllerPlugin::default().without_camera());
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
