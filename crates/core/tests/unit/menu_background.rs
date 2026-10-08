use bevy::prelude::*;
use gameplay::levels::LevelRoot;

use super::{spawn, GameState};

#[test]
fn menu_room_and_cameras_are_state_scoped() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, |mut commands: Commands| spawn(&mut commands));
    app.update();

    let world = app.world_mut();
    let room = world
        .query_filtered::<Entity, With<LevelRoot>>()
        .single(world)
        .expect("menu room root");
    assert!(world.get::<DespawnOnExit<GameState>>(room).is_some());
    let ui = world
        .query_filtered::<&Camera, (With<Camera2d>, With<IsDefaultUiCamera>)>()
        .single(world)
        .expect("UI camera");
    assert_eq!(ui.order, 1);
    let room_camera = world
        .query_filtered::<&Camera, With<Camera3d>>()
        .single(world)
        .expect("room camera");
    assert_eq!(room_camera.order, 0);
}
