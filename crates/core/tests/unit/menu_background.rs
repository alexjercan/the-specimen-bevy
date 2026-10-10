use bevy::prelude::*;
use gameplay::levels::{DoorRef, LevelRoot};

use super::{GameState, spawn};

#[test]
fn menu_room_and_cameras_are_state_scoped() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, |mut commands: Commands| {
            spawn(&mut commands, GameState::MainMenu)
        });
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

#[test]
fn menu_room_links_are_removed_on_each_visit() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, |mut commands: Commands| {
            spawn(&mut commands, GameState::MainMenu)
        });

    for _ in 0..3 {
        app.update();
        let world = app.world_mut();
        let root = world
            .query_filtered::<Entity, With<LevelRoot>>()
            .single(world)
            .expect("menu room root");
        assert_eq!(world.query::<&DoorRef>().iter(world).count(), 2);
        world.entity_mut(root).despawn();
        assert_eq!(world.query::<&DoorRef>().iter(world).count(), 0);
    }
}
