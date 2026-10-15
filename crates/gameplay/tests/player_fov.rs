use std::time::Duration;

use bevy::prelude::*;
use game_settings::{GameSettings, GameSettingsPlugin};
use gameplay::player::{PlayerController, PlayerViewPlugin};

mod common;

fn app() -> App {
    let mut app = common::player_app(Duration::from_millis(100));
    app.add_plugins((GameSettingsPlugin { persist: false }, PlayerViewPlugin));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn fov(app: &App, player: Entity) -> f32 {
    let Projection::Perspective(projection) = app.world().get::<Projection>(player).unwrap() else {
        panic!("player camera should use a perspective projection");
    };
    projection.fov
}

#[test]
fn new_player_camera_uses_current_fov() {
    let mut app = app();
    app.world_mut().resource_mut::<GameSettings>().fov = 82.0;
    let player = app.world_mut().spawn(PlayerController).id();
    app.update();
    assert!((fov(&app, player) - 82.0_f32.to_radians()).abs() < 0.0001);
}

#[test]
fn external_fov_change_updates_live_player_camera() {
    let mut app = app();
    let player = app.world_mut().spawn(PlayerController).id();
    app.update();
    assert!((fov(&app, player) - 55.0_f32.to_radians()).abs() < 0.0001);
    app.world_mut().resource_mut::<GameSettings>().fov = 96.0;
    app.update();
    assert!((fov(&app, player) - 96.0_f32.to_radians()).abs() < 0.0001);
}
