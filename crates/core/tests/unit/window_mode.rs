use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use game_settings::{DisplayMode, GameSettings};

use crate::{apply_window_mode, window_mode};

#[test]
fn display_mode_defaults_to_borderless_fullscreen() {
    assert_eq!(
        GameSettings::default().display_mode,
        DisplayMode::Fullscreen
    );
    assert_eq!(
        window_mode(DisplayMode::Fullscreen),
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    );
    assert_eq!(window_mode(DisplayMode::Windowed), WindowMode::Windowed);
}

#[test]
fn changing_display_mode_updates_primary_window_only() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(GameSettings::default())
        .add_systems(Update, apply_window_mode);
    let primary = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    let secondary = app.world_mut().spawn(Window::default()).id();
    app.update();
    assert_eq!(
        app.world().get::<Window>(primary).unwrap().mode,
        window_mode(DisplayMode::Fullscreen)
    );
    app.world_mut().resource_mut::<GameSettings>().display_mode = DisplayMode::Windowed;
    app.update();
    assert_eq!(
        app.world().get::<Window>(primary).unwrap().mode,
        WindowMode::Windowed
    );
    assert_eq!(
        app.world().get::<Window>(secondary).unwrap().mode,
        WindowMode::Windowed
    );
}
