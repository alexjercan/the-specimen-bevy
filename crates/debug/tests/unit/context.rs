use bevy::prelude::*;
use bevy_egui::{EguiContext, PrimaryEguiContext};

use super::keep_inspector_on_window_camera;

#[test]
fn inspector_context_moves_to_new_camera_after_menu_camera_despawns() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, keep_inspector_on_window_camera);

    let menu_room = app.world_mut().spawn(Camera::default()).id();
    let menu_ui = app
        .world_mut()
        .spawn(Camera {
            order: 1,
            ..default()
        })
        .id();
    app.update();
    assert!(app.world().get::<PrimaryEguiContext>(menu_ui).is_some());
    assert!(app.world().get::<EguiContext>(menu_ui).is_some());
    assert!(app.world().get::<PrimaryEguiContext>(menu_room).is_none());

    app.world_mut().despawn(menu_ui);
    app.world_mut().despawn(menu_room);
    let player = app.world_mut().spawn(Camera::default()).id();
    app.update();
    assert!(app.world().get::<PrimaryEguiContext>(player).is_some());
    assert!(app.world().get::<EguiContext>(player).is_some());
}
