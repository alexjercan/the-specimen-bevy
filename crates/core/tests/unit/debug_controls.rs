use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use debug::DebugSettings;
use gameplay::controller::{PlayerController, PlayerControlsEnabled};

use super::sync_debug_controls;

#[test]
fn inspector_releases_cursor_and_disables_player_until_closed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<DebugSettings>()
        .init_resource::<PlayerControlsEnabled>()
        .add_systems(PostUpdate, sync_debug_controls);
    app.world_mut().spawn(PlayerController);
    let window = app
        .world_mut()
        .spawn((PrimaryWindow, CursorOptions::default()))
        .id();

    app.update();
    assert!(app.world().resource::<PlayerControlsEnabled>().0);
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert!(!cursor.visible);
    assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);

    app.world_mut().resource_mut::<DebugSettings>().inspector = true;
    app.update();
    assert!(!app.world().resource::<PlayerControlsEnabled>().0);
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert!(cursor.visible);
    assert_eq!(cursor.grab_mode, CursorGrabMode::None);

    app.world_mut().resource_mut::<DebugSettings>().inspector = false;
    app.update();
    assert!(app.world().resource::<PlayerControlsEnabled>().0);
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert!(!cursor.visible);
    assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
}
