use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin, PlayerControlsEnabled},
    levels::{
        panel_center, panel_top, Door, DoorPanel, DoorPlugin, DoorState, DoorSwing, ToggleDoor,
    },
};

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.add_plugins(PlayerControllerPlugin::default().without_camera());
    app.add_plugins(DoorPlugin);
    app.finish();
    app.cleanup();
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)))
        .id();
    app.update();
    (app, player)
}

fn door(app: &mut App, x: f32, z: f32) -> Entity {
    app.world_mut()
        .spawn(Door {
            position: Vec2::new(x, z),
            rotation: Quat::IDENTITY,
            frame: String::new(),
            panel: String::new(),
            state: DoorState::Closed,
        })
        .id()
}

fn press_f(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();
}

fn release_f(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyF);
    app.update();
}

#[test]
fn toggle_message_animates_hinge_and_reverses_from_current_pose() {
    let (mut app, _) = app();
    let entity = door(&mut app, 0.0, -1.5);
    let panel = app
        .world_mut()
        .spawn((
            DoorPanel,
            Transform::from_xyz(-0.59, 0.0, 0.0),
            ChildOf(entity),
        ))
        .id();
    app.world_mut().write_message(ToggleDoor(entity));
    app.update();
    assert_eq!(
        app.world().get::<Door>(entity).unwrap().state,
        DoorState::Open
    );
    let angle = app.world().get::<DoorSwing>(entity).unwrap().0;
    assert!(angle > 0.0 && angle < FRAC_PI_2);
    assert!(
        (app.world().get::<Transform>(panel).unwrap().rotation - Quat::from_rotation_y(angle))
            .length()
            < 0.001
    );

    app.world_mut().write_message(ToggleDoor(entity));
    app.update();
    assert_eq!(
        app.world().get::<Door>(entity).unwrap().state,
        DoorState::Closed
    );
    let closing = app.world().get::<DoorSwing>(entity).unwrap().0;
    assert!(closing < angle && closing >= 0.0);
    assert!((app.world().get::<Transform>(panel).unwrap().translation.x + 0.59).abs() < 0.001);
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(app.world().get::<DoorSwing>(entity).unwrap().0, 0.0);
    assert_eq!(
        app.world().get::<Transform>(panel).unwrap().rotation,
        Quat::IDENTITY
    );
}

#[test]
fn f_targets_nearest_panel_and_open_panel_not_old_opening() {
    let (mut app, player) = app();
    let near = door(&mut app, 0.0, -1.5);
    let far = door(&mut app, 0.0, -2.0);
    let side = door(&mut app, 1.5, -1.5);
    let behind = door(&mut app, 0.0, 1.0);
    press_f(&mut app);
    assert_eq!(
        app.world().get::<Door>(near).unwrap().state,
        DoorState::Open
    );
    for entity in [far, side, behind] {
        assert_eq!(
            app.world().get::<Door>(entity).unwrap().state,
            DoorState::Closed
        );
    }
    app.world_mut().despawn(far);
    for _ in 0..5 {
        app.update();
    }
    let swing = app.world().get::<DoorSwing>(near).unwrap();
    assert!((swing.0 - FRAC_PI_2).abs() < 0.001);
    let top = panel_top(app.world().get::<Door>(near).unwrap(), swing);
    let center = panel_center(app.world().get::<Door>(near).unwrap(), swing);
    assert!((center.y - 1.1).abs() < 0.001);
    assert!((center.x + 0.59).abs() < 0.001);
    assert!((center.z + 2.09).abs() < 0.001);
    assert!((top.x + 0.59).abs() < 0.001);
    assert!((top.z + 2.09).abs() < 0.001);
    release_f(&mut app);
    press_f(&mut app);
    assert_eq!(
        app.world().get::<Door>(near).unwrap().state,
        DoorState::Open
    );
    release_f(&mut app);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .x = -0.59;
    press_f(&mut app);
    assert_eq!(
        app.world().get::<Door>(near).unwrap().state,
        DoorState::Closed
    );
}

#[test]
fn paused_player_cannot_toggle_a_door() {
    let (mut app, _) = app();
    let entity = door(&mut app, 0.0, -1.5);
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    press_f(&mut app);
    assert_eq!(
        app.world().get::<Door>(entity).unwrap().state,
        DoorState::Closed
    );
    release_f(&mut app);
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = true;
    press_f(&mut app);
    assert_eq!(
        app.world().get::<Door>(entity).unwrap().state,
        DoorState::Open
    );
}

#[test]
fn interaction_respects_door_yaw() {
    let (mut app, player) = app();
    let entity = door(&mut app, 0.0, -1.5);
    app.world_mut().get_mut::<Door>(entity).unwrap().rotation = Quat::from_rotation_y(FRAC_PI_2);
    {
        let mut player_transform = app.world_mut().get_mut::<Transform>(player).unwrap();
        player_transform.translation = Vec3::new(2.0, 1.6, -1.5);
        player_transform.rotation = Quat::from_rotation_y(FRAC_PI_2);
    }
    press_f(&mut app);
    assert_eq!(
        app.world().get::<Door>(entity).unwrap().state,
        DoorState::Open
    );
}
