use std::time::Duration;

use bevy::{
    prelude::*,
    state::app::StatesPlugin,
    time::TimeUpdateStrategy,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use debug::DebugSettings;
use gameplay::controller::{PlayerController, PlayerControlsEnabled};

use crate::PauseState;

use super::sync_debug_controls;

#[test]
fn inspector_pauses_game_clock_and_controls_until_closed() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)))
        .init_state::<PauseState>()
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
    assert!(app.world().resource::<Time<Virtual>>().is_paused());
    let elapsed = app.world().resource::<Time<Virtual>>().elapsed();
    app.update();
    assert_eq!(app.world().resource::<Time<Virtual>>().elapsed(), elapsed);
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert!(cursor.visible);
    assert_eq!(cursor.grab_mode, CursorGrabMode::None);

    app.world_mut().resource_mut::<DebugSettings>().inspector = false;
    app.update();
    assert!(app.world().resource::<PlayerControlsEnabled>().0);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert!(!cursor.visible);
    assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
    app.update();
    assert!(app.world().resource::<Time<Virtual>>().elapsed() > elapsed);

    app.world_mut()
        .resource_mut::<NextState<PauseState>>()
        .set(PauseState::Paused);
    app.update();
    assert!(app.world().resource::<Time<Virtual>>().is_paused());
    assert!(!app.world().resource::<PlayerControlsEnabled>().0);
    app.world_mut().resource_mut::<DebugSettings>().inspector = true;
    app.update();
    app.world_mut().resource_mut::<DebugSettings>().inspector = false;
    app.update();
    assert!(app.world().resource::<Time<Virtual>>().is_paused());
    assert!(!app.world().resource::<PlayerControlsEnabled>().0);
}
