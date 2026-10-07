use std::time::Duration;

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin, PlayerControlsEnabled},
    levels::{Door, DoorOf, DoorRef, DoorState, DoorSwing, Passage, Room},
};

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_plugins(PlayerControllerPlugin::default().without_camera());
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

fn position(app: &App, player: Entity) -> Vec3 {
    app.world().get::<Transform>(player).unwrap().translation
}

fn walk(app: &mut App, key: KeyCode, frames: usize) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    for _ in 0..frames {
        app.update();
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
    app.update();
}

fn room(app: &mut App, opening: Option<Entity>) {
    let mut room = app
        .world_mut()
        .spawn(Room(Rect::new(-1.25, -1.25, 1.25, 1.25)));
    if let Some(opening) = opening {
        room.with_related::<DoorOf>(DoorRef(opening));
    }
}

#[test]
fn solid_walls_stop_large_steps_and_allow_sliding() {
    let (mut app, player) = app();
    room(&mut app, None);
    walk(&mut app, KeyCode::KeyW, 30);
    assert!(position(&app, player).z >= -0.88);
    assert_eq!(position(&app, player).y, 1.6);
    walk(&mut app, KeyCode::KeyD, 20);
    assert!(position(&app, player).x > 0.5);
    assert!(position(&app, player).x <= 0.88);
}

#[test]
fn sliding_across_horizontal_wall_tiles_does_not_snag_at_the_seam() {
    let (mut app, player) = app();
    app.world_mut().spawn(Room(Rect::new(-2.5, -2.5, 2.5, 0.0)));
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(-1.0, 1.6, -0.375);
    walk(&mut app, KeyCode::KeyD, 7);
    let at = position(&app, player);
    assert!(
        at.x > 0.5,
        "horizontal wall seam stopped the player: {at:?}"
    );
    assert!((at.z + 0.375).abs() < 0.001);
}

#[test]
fn sliding_across_vertical_wall_tiles_does_not_snag_at_the_seam() {
    let (mut app, player) = app();
    app.world_mut().spawn(Room(Rect::new(-2.5, -2.5, 0.0, 2.5)));
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(-0.375, 1.6, 1.0);
    walk(&mut app, KeyCode::KeyW, 7);
    let at = position(&app, player);
    assert!(at.z < -0.5, "vertical wall seam stopped the player: {at:?}");
    assert!((at.x + 0.375).abs() < 0.001);
}

#[test]
fn glancing_past_joined_wall_ends_does_not_hit_a_square_corner() {
    let (mut app, player) = app();
    room(&mut app, None);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(-1.7, 1.6, -1.48);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.update();
    let at = position(&app, player);
    assert!(at.x > -1.55, "wall corner snagged the player: {at:?}");
    assert!(at.z < -1.48);
}

#[test]
fn model_free_passage_is_walkable() {
    let (mut app, player) = app();
    let passage = app.world_mut().spawn(Passage(Vec2::new(0.0, -1.25))).id();
    room(&mut app, Some(passage));
    walk(&mut app, KeyCode::KeyW, 12);
    assert!(position(&app, player).z < -2.0);
}

#[test]
fn door_blocks_when_closed_and_allows_crossing_when_open() {
    let (mut app, player) = app();
    let door = app
        .world_mut()
        .spawn(Door {
            position: Vec2::new(0.0, -1.25),
            rotation: Quat::IDENTITY,
            frame: "wall_doorway".to_owned(),
            panel: "door_panel".to_owned(),
            state: DoorState::Closed,
        })
        .id();
    room(&mut app, Some(door));
    walk(&mut app, KeyCode::KeyW, 12);
    assert!(position(&app, player).z > -1.0);
    app.world_mut().get_mut::<DoorSwing>(door).unwrap().0 = std::f32::consts::FRAC_PI_2;
    app.world_mut().get_mut::<Door>(door).unwrap().state = DoorState::Open;
    walk(&mut app, KeyCode::KeyW, 12);
    assert!(position(&app, player).z < -2.0);
}

#[test]
fn swinging_leaf_blocks_the_current_pose_not_only_the_closed_pose() {
    let (mut app, player) = app();
    let door = app
        .world_mut()
        .spawn(Door {
            position: Vec2::new(0.0, -1.25),
            rotation: Quat::IDENTITY,
            frame: "wall_doorway".to_owned(),
            panel: "door_panel".to_owned(),
            state: DoorState::Open,
        })
        .id();
    app.world_mut().get_mut::<DoorSwing>(door).unwrap().0 = std::f32::consts::FRAC_PI_4;
    room(&mut app, Some(door));
    walk(&mut app, KeyCode::KeyW, 12);
    let z = position(&app, player).z;
    assert!(
        z < -1.0 && z > -1.8,
        "player crossed the swinging panel: {z}"
    );
}

#[test]
fn door_frame_blocks_sideways_motion_through_open_leaf() {
    let (mut app, player) = app();
    let door = app
        .world_mut()
        .spawn(Door {
            position: Vec2::new(0.0, -1.25),
            rotation: Quat::IDENTITY,
            frame: "wall_doorway".to_owned(),
            panel: "door_panel".to_owned(),
            state: DoorState::Open,
        })
        .id();
    app.world_mut().get_mut::<DoorSwing>(door).unwrap().0 = std::f32::consts::FRAC_PI_2;
    room(&mut app, Some(door));
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .x = 0.95;
    walk(&mut app, KeyCode::KeyW, 15);
    assert!(position(&app, player).z > -1.0);
}

#[test]
fn a_diagonal_step_slides_along_the_wall_without_tunneling() {
    let (mut app, player) = app();
    room(&mut app, None);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    for _ in 0..8 {
        app.update();
    }
    let at = position(&app, player);
    assert!(at.z >= -0.88);
    assert!(at.x > 0.4 && at.x <= 0.88);
}

#[test]
fn player_at_resting_contact_can_move_away() {
    let (mut app, player) = app();
    room(&mut app, None);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .z = -0.875;
    walk(&mut app, KeyCode::KeyS, 3);
    assert!(position(&app, player).z > -0.1);
}

#[test]
fn initial_overlap_can_move_out_of_a_wall() {
    let (mut app, player) = app();
    room(&mut app, None);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .z = -1.2;
    walk(&mut app, KeyCode::KeyS, 5);
    assert!(position(&app, player).z > -0.9);
}

#[test]
fn disabled_controls_do_not_turn_walk_or_interact() {
    let (mut app, player) = app();
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    walk(&mut app, KeyCode::KeyW, 4);
    assert_eq!(position(&app, player), Vec3::new(0.0, 1.6, 0.0));
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = true;
    app.update();
    assert_eq!(position(&app, player), Vec3::new(0.0, 1.6, 0.0));
}
