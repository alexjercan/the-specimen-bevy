use std::{f32::consts::PI, time::Duration};

use bevy::{
    input::{mouse::MouseMotion, InputPlugin},
    prelude::*,
    time::TimeUpdateStrategy,
};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin, PlayerControlsEnabled, PlayerInput},
    levels::{
        build_first_floor, Door, DoorPlugin, DoorState, FuseInventory, FusePickup, FusePlugin,
        FuseSeed, GameplaySound, GameplaySoundKind, Hidden, HidingPhase, HidingPlugin, HidingSpot,
        Prop, PropCollider, Room, HIDING_TRANSITION,
    },
};

const EYE: f32 = 1.6;
const FRAME: f32 = 0.1;
const PLAYER_RADIUS: f32 = 0.25;
const WALL_HALF_DEPTH: f32 = 0.125;

#[derive(Resource, Default)]
struct SoundLog(Vec<(GameplaySoundKind, Vec3)>);

fn collect_sounds(mut events: MessageReader<GameplaySound>, mut log: ResMut<SoundLog>) {
    log.0
        .extend(events.read().map(|sound| (sound.kind, sound.position)));
}

fn plugins(app: &mut App) {
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_plugins(PlayerControllerPlugin::default().without_camera())
        .add_plugins((DoorPlugin, FusePlugin, HidingPlugin));
    app.finish();
    app.cleanup();
}

fn app() -> (App, Entity) {
    let mut app = App::new();
    plugins(&mut app);
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, EYE, 0.0)))
        .id();
    app.update();
    (app, player)
}

fn spot(app: &mut App, kind: HidingSpot, position: Vec3, yaw: f32) -> Entity {
    let module = match kind {
        HidingSpot::Locker => "concept_locker",
        HidingSpot::Table => "concept_table",
    };
    let entity = app
        .world_mut()
        .spawn((
            Prop(module.to_owned()),
            Transform::from_translation(position).with_rotation(Quat::from_rotation_y(yaw)),
        ))
        .id();
    if kind == HidingSpot::Table {
        app.world_mut().entity_mut(entity).insert(kind);
    }
    app.update();
    entity
}

fn door(app: &mut App, z: f32) -> Entity {
    app.world_mut()
        .spawn(Door {
            position: Vec2::new(0.0, z),
            rotation: Quat::IDENTITY,
            frame: String::new(),
            panel: String::new(),
            state: DoorState::Closed,
        })
        .id()
}

fn place(app: &mut App, player: Entity, eye: Vec3, target: Vec3) {
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_translation(eye).looking_at(target, Vec3::Y));
}

fn aim(app: &mut App, player: Entity, target: Vec3) {
    let eye = transform(app, player).translation;
    place(app, player, eye, target);
}

fn press_f(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyF);
    app.update();
}

fn settle(app: &mut App) {
    for _ in 0..=(HIDING_TRANSITION / FRAME).ceil() as usize {
        app.update();
    }
}

fn hold(app: &mut App, keys: &[KeyCode], frames: usize) {
    for &key in keys {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }
    for _ in 0..frames {
        app.update();
    }
    for &key in keys {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(key);
    }
    app.update();
}

fn transform(app: &App, player: Entity) -> Transform {
    *app.world().get::<Transform>(player).unwrap()
}

fn hidden(app: &App, player: Entity) -> Option<Hidden> {
    app.world().get::<Hidden>(player).copied()
}

fn yaw(transform: &Transform) -> f32 {
    transform.rotation.to_euler(EulerRot::YXZ).0
}

fn same_angle(a: f32, b: f32) -> bool {
    let delta = (a - b).rem_euclid(2.0 * PI);
    delta < 0.001 || delta > 2.0 * PI - 0.001
}

fn inside_collider(
    point: Vec2,
    collider: &PropCollider,
    transform: &Transform,
    margin: f32,
) -> bool {
    let (yaw, ..) = transform.rotation.to_euler(EulerRot::YXZ);
    let local = Quat::from_rotation_y(-yaw)
        * (Vec3::new(point.x, 0.0, point.y)
            - Vec3::new(transform.translation.x, 0.0, transform.translation.z));
    let offset = local.xz() - collider.center;
    let outside = (offset.abs() - collider.half).max(Vec2::ZERO);
    outside.length() < margin
}

#[test]
fn f_enters_a_locker_and_f_leaves_to_its_exit() {
    let (mut app, player) = app();
    let locker = spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    settle(&mut app);
    let state = hidden(&app, player).unwrap();
    assert_eq!(state.spot, locker);
    assert_eq!(state.height, EYE);
    assert!(state.settled());
    let inside = transform(&app, player);
    assert!(inside.translation.distance(Vec3::new(0.0, 1.45, -2.0)) < 0.001);
    assert!(same_angle(yaw(&inside), PI));
    assert!((1.15..=1.78).contains(&inside.translation.y));
    let place = transform(&app, locker);
    let front = place.rotation * Vec3::NEG_Z;
    let back_plate = place.transform_point(Vec3::new(0.0, inside.translation.y, 0.3));
    assert!(inside.forward().dot(front) > 0.999);
    assert!(inside.forward().dot(back_plate - inside.translation) < 0.0);

    press_f(&mut app);
    settle(&mut app);
    assert_eq!(hidden(&app, player), None);
    let outside = transform(&app, player);
    assert!(outside.translation.distance(Vec3::new(0.0, EYE, -1.05)) < 0.001);
    assert!(same_angle(yaw(&outside), PI));
}

#[test]
fn f_crawls_under_a_table_and_restores_eye_height_on_leave() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Table, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 0.4, -2.0));
    press_f(&mut app);
    settle(&mut app);
    assert!(hidden(&app, player).is_some());
    assert!((transform(&app, player).translation.y - 0.45).abs() < 0.001);

    press_f(&mut app);
    settle(&mut app);
    assert_eq!(hidden(&app, player), None);
    let outside = transform(&app, player).translation;
    assert!(outside.distance(Vec3::new(0.0, EYE, -0.9)) < 0.001);
}

#[test]
fn hidden_player_cannot_walk_or_run_but_can_look() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    settle(&mut app);
    let before = transform(&app, player);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    for _ in 0..5 {
        app.update();
        let input = app.world().get::<PlayerInput>(player).unwrap();
        assert_eq!(input.movement, Vec2::ZERO);
        assert!(!input.running);
    }
    assert_eq!(transform(&app, player).translation, before.translation);

    app.world_mut().write_message(MouseMotion {
        delta: Vec2::new(200.0, 0.0),
    });
    app.update();
    let after = transform(&app, player);
    assert_eq!(after.translation, before.translation);
    assert!(!same_angle(yaw(&after), yaw(&before)));
    assert!(hidden(&app, player).is_some());
}

#[test]
fn leaving_does_not_trap_the_player_and_keeps_the_prop_solid() {
    let (mut app, player) = app();
    let locker = spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    assert!(app.world().get::<PropCollider>(locker).is_some());
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    settle(&mut app);
    press_f(&mut app);
    settle(&mut app);
    let exit = transform(&app, player).translation;

    hold(&mut app, &[KeyCode::KeyS], 5);
    let collider = *app.world().get::<PropCollider>(locker).unwrap();
    let place = transform(&app, locker);
    let back = transform(&app, player).translation;
    assert!(!inside_collider(
        back.xz(),
        &collider,
        &place,
        PLAYER_RADIUS - 0.01
    ));

    hold(&mut app, &[KeyCode::KeyW], 3);
    let forward = transform(&app, player).translation;
    assert!(forward.z - back.z > 0.5);
    assert!(forward.z > exit.z);
}

#[test]
fn walls_block_hiding() {
    let (mut app, player) = app();
    app.world_mut()
        .spawn(Room(Rect::new(-1.25, -3.75, 1.25, -1.25)));
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(0.0, EYE, -0.6));
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    assert_eq!(hidden(&app, player), None);
}

#[test]
fn disabled_controls_do_not_hide() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    press_f(&mut app);
    assert_eq!(hidden(&app, player), None);
}

#[test]
fn nearer_door_wins_over_a_hiding_spot_behind_it() {
    let (mut app, player) = app();
    let entry = door(&mut app, -0.8);
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    assert_eq!(
        app.world().get::<Door>(entry).unwrap().state,
        DoorState::Open
    );
    assert_eq!(hidden(&app, player), None);
}

#[test]
fn nearer_hiding_spot_wins_over_a_door_behind_it() {
    let (mut app, player) = app();
    let behind = door(&mut app, -2.5);
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -1.2), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -1.2));
    press_f(&mut app);
    assert!(hidden(&app, player).is_some());
    assert_eq!(
        app.world().get::<Door>(behind).unwrap().state,
        DoorState::Closed
    );
}

#[test]
fn leaving_a_hiding_spot_does_not_toggle_the_door_in_view() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    settle(&mut app);
    let ahead = door(&mut app, -0.5);
    press_f(&mut app);
    settle(&mut app);
    assert_eq!(hidden(&app, player), None);
    assert_eq!(
        app.world().get::<Door>(ahead).unwrap().state,
        DoorState::Closed
    );
}

#[test]
fn fuse_on_a_table_wins_over_hiding_under_it() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Table, Vec3::new(0.0, 0.0, -1.5), PI);
    let fuse = app
        .world_mut()
        .spawn((FusePickup { slot: 0 }, Transform::from_xyz(0.0, 0.8, -1.5)))
        .id();
    aim(&mut app, player, Vec3::new(0.0, 0.83, -1.5));
    press_f(&mut app);
    assert!(app.world().get_entity(fuse).is_err());
    assert_eq!(app.world().get::<FuseInventory>(player).unwrap().0, 1);
    assert_eq!(hidden(&app, player), None);

    aim(&mut app, player, Vec3::new(0.0, 0.3, -1.2));
    press_f(&mut app);
    assert!(hidden(&app, player).is_some());
}

#[test]
fn hidden_player_cannot_pick_up_fuses() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Table, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 0.3, -1.6));
    press_f(&mut app);
    settle(&mut app);
    assert!(hidden(&app, player).is_some());
    let eye = transform(&app, player).translation;
    let fuse = app
        .world_mut()
        .spawn((
            FusePickup { slot: 0 },
            Transform::from_translation(eye + Vec3::new(0.0, -0.03, 0.6)),
        ))
        .id();
    aim(&mut app, player, eye + Vec3::new(0.0, 0.0, 0.6));
    press_f(&mut app);
    assert!(app.world().get_entity(fuse).is_ok());
    assert_eq!(app.world().get::<FuseInventory>(player).unwrap().0, 0);
    settle(&mut app);
    assert_eq!(hidden(&app, player), None);
}

#[test]
fn occupied_spot_rejects_a_second_player() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    let other = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.3, EYE, 0.0)))
        .id();
    app.update();
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    aim(&mut app, other, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    let hidden_count = [player, other]
        .into_iter()
        .filter(|&entity| hidden(&app, entity).is_some())
        .count();
    assert_eq!(hidden_count, 1);
}

#[test]
fn first_floor_hiding_spots_have_clear_exits_and_round_trip() {
    let mut app = App::new();
    app.insert_resource(FuseSeed(7));
    plugins(&mut app);
    app.add_systems(Startup, build_first_floor);
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, EYE, -5.0)))
        .id();
    app.update();

    let world = app.world_mut();
    let spots: Vec<(Entity, HidingSpot, Transform, String)> = world
        .query::<(Entity, &HidingSpot, &Transform, &Prop)>()
        .iter(world)
        .map(|(entity, spot, transform, prop)| (entity, *spot, *transform, prop.0.clone()))
        .collect();
    let rooms: Vec<Rect> = world
        .query::<&Room>()
        .iter(world)
        .map(|room| room.0)
        .collect();
    let colliders: Vec<(PropCollider, Transform)> = world
        .query::<(&PropCollider, &Transform)>()
        .iter(world)
        .map(|(collider, transform)| (*collider, *transform))
        .collect();
    let doors: Vec<Vec2> = world
        .query::<&Door>()
        .iter(world)
        .map(|door| door.position)
        .collect();
    assert_eq!(spots.len(), 9);
    let lockers: Vec<_> = spots
        .iter()
        .filter(|(_, kind, ..)| *kind == HidingSpot::Locker)
        .collect();
    assert_eq!(lockers.len(), 6);
    let maintenance_locker = lockers
        .iter()
        .find(|(_, _, transform, _)| transform.translation.xz() == Vec2::new(-13.35, -18.75))
        .expect("maintenance locker");
    assert!(
        maintenance_locker
            .2
            .translation
            .xz()
            .distance(Vec2::new(-12.5, -16.25))
            > 2.0
    );
    let concept_lockers = world
        .query::<&Prop>()
        .iter(world)
        .filter(|prop| prop.0 == "concept_locker")
        .count();
    assert_eq!(lockers.len(), concept_lockers);

    for (entity, kind, spot_transform, module) in spots {
        let expected = match kind {
            HidingSpot::Locker => "concept_locker",
            HidingSpot::Table => "concept_table",
        };
        assert_eq!(module, expected);
        let exit = kind.exit(&spot_transform);
        let margin = PLAYER_RADIUS + WALL_HALF_DEPTH;
        assert!(
            rooms.iter().any(|room| {
                room.min.x + margin < exit.x
                    && exit.x < room.max.x - margin
                    && room.min.y + margin < exit.y
                    && exit.y < room.max.y - margin
            }),
            "exit {exit} is too close to a wall"
        );
        for (collider, transform) in &colliders {
            assert!(
                !inside_collider(exit, collider, transform, PLAYER_RADIUS + 0.05),
                "exit {exit} overlaps a prop"
            );
        }
        for position in &doors {
            assert!(position.distance(exit) > 1.5, "exit {exit} blocks a door");
        }

        let eye = Vec3::new(exit.x, EYE, exit.y);
        let center = spot_transform.transform_point(match kind {
            HidingSpot::Locker => Vec3::new(0.0, 1.0, 0.0),
            HidingSpot::Table => Vec3::new(0.0, 0.4, 0.0),
        });
        place(&mut app, player, eye, center);
        press_f(&mut app);
        settle(&mut app);
        assert_eq!(
            hidden(&app, player).map(|hidden| (hidden.spot, hidden.settled())),
            Some((entity, true)),
            "could not enter spot at {exit}"
        );
        press_f(&mut app);
        settle(&mut app);
        assert_eq!(hidden(&app, player), None);
        let out = transform(&app, player).translation;
        assert!(out.distance(eye) < 0.001);
        hold(&mut app, &[KeyCode::KeyW], 3);
        let walked = transform(&app, player).translation;
        assert!(walked.distance(out) > 0.5, "player is stuck at {exit}");
    }
}

fn walk_to(app: &mut App, player: Entity, target: Vec2) {
    for _ in 0..40 {
        let eye = transform(app, player).translation;
        if eye.xz().distance(target) < 0.2 {
            return;
        }
        place(app, player, eye, Vec3::new(target.x, eye.y, target.y));
        hold(app, &[KeyCode::KeyW], 1);
    }
    panic!(
        "player stopped at {} before {target}",
        transform(app, player).translation
    );
}

#[test]
fn hiding_actions_emit_one_kind_specific_cue_per_transition() {
    let (mut app, player) = app();
    app.init_resource::<SoundLog>()
        .add_systems(PostUpdate, collect_sounds);
    for (kind, position, enter, leave) in [
        (
            HidingSpot::Locker,
            Vec3::new(0.0, 0.0, -2.0),
            GameplaySoundKind::LockerOpen,
            GameplaySoundKind::LockerClose,
        ),
        (
            HidingSpot::Table,
            Vec3::new(2.0, 0.0, -2.0),
            GameplaySoundKind::TableEnter,
            GameplaySoundKind::TableLeave,
        ),
    ] {
        let entity = spot(&mut app, kind, position, PI);
        let eye = kind.eye(&transform(&app, entity));
        place(&mut app, player, Vec3::new(position.x, EYE, 0.0), eye);
        press_f(&mut app);
        settle(&mut app);
        assert_eq!(hidden(&app, player).map(|hidden| hidden.spot), Some(entity));
        press_f(&mut app);
        settle(&mut app);
        assert_eq!(hidden(&app, player), None);
        let cues = &app.world().resource::<SoundLog>().0;
        assert_eq!(
            &cues[cues.len() - 2..],
            &[(enter, position), (leave, position)]
        );
        app.world_mut().despawn(entity);
    }
    assert_eq!(app.world().resource::<SoundLog>().0.len(), 4);
}

#[test]
fn storage_lockers_hide_with_f_after_walking_in_from_the_door() {
    let mut app = App::new();
    app.insert_resource(FuseSeed(7));
    plugins(&mut app);
    app.add_systems(Startup, build_first_floor);
    app.update();
    let world = app.world_mut();
    let lockers: Vec<(Entity, Transform)> = world
        .query::<(Entity, &Prop, &Transform)>()
        .iter(world)
        .filter(|(_, prop, transform)| {
            prop.0 == "concept_locker"
                && Rect::new(6.25, -16.25, 13.75, -8.75).contains(transform.translation.xz())
        })
        .map(|(entity, _, transform)| (entity, *transform))
        .collect();
    assert_eq!(lockers.len(), 2);
    let hall = Vec3::new(4.75, EYE, -12.5);
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_translation(hall)))
        .id();
    app.update();
    aim(&mut app, player, Vec3::new(6.25, 1.0, -12.5));
    press_f(&mut app);
    settle(&mut app);

    for (locker, place_at) in lockers {
        let route: &[Vec2] = if place_at.translation.x > 12.0 {
            &[Vec2::new(9.5, -10.8), Vec2::new(12.2, -11.0)]
        } else {
            &[Vec2::new(9.0, -10.8)]
        };
        place(&mut app, player, hall, Vec3::new(6.25, EYE, -12.5));
        for &waypoint in [Vec2::new(7.0, -12.5)].iter().chain(route) {
            walk_to(&mut app, player, waypoint);
        }
        aim(
            &mut app,
            player,
            place_at.transform_point(Vec3::new(0.0, 1.0, 0.0)),
        );
        press_f(&mut app);
        settle(&mut app);
        assert_eq!(
            hidden(&app, player).map(|hidden| (hidden.spot, hidden.settled())),
            Some((locker, true)),
            "could not hide in storage locker at {}",
            place_at.translation
        );
        press_f(&mut app);
        settle(&mut app);
        assert_eq!(hidden(&app, player), None);
        let exit = HidingSpot::Locker.exit(&place_at);
        assert!(transform(&app, player).translation.xz().distance(exit) < 0.001);
    }
}

#[test]
fn entering_moves_in_small_steps_instead_of_teleporting() {
    let (mut app, player) = app();
    let locker = spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    let start = transform(&app, player).translation;
    let eye = HidingSpot::Locker.eye(&transform(&app, locker));
    let span = start.distance(eye);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    let mut previous = start;
    let mut frames = 0;
    while !hidden(&app, player).is_some_and(|hidden| hidden.settled()) {
        app.update();
        frames += 1;
        assert!(frames <= 10, "transition did not finish");
        let current = transform(&app, player).translation;
        assert!(current.distance(previous) < span * 0.6);
        assert!(matches!(
            hidden(&app, player).unwrap().phase,
            HidingPhase::Entering(_) | HidingPhase::Hidden
        ));
        previous = current;
    }
    assert!(frames >= 3);
    assert!(previous.distance(eye) < 0.001);
}

#[test]
fn look_is_ignored_during_the_transition_and_restored_after() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    assert!(!hidden(&app, player).unwrap().settled());
    app.world_mut().write_message(MouseMotion {
        delta: Vec2::new(300.0, 120.0),
    });
    settle(&mut app);
    let settled = transform(&app, player);
    let (yaw, pitch, _) = settled.rotation.to_euler(EulerRot::YXZ);
    assert!(same_angle(yaw, PI));
    assert!(pitch.abs() < 0.001);

    app.world_mut().write_message(MouseMotion {
        delta: Vec2::new(0.0, 120.0),
    });
    app.update();
    let (_, pitch, _) = transform(&app, player).rotation.to_euler(EulerRot::YXZ);
    assert!(pitch < -0.1);
}

#[test]
fn f_during_entering_reverses_to_the_safe_exit() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    assert!(matches!(
        hidden(&app, player).unwrap().phase,
        HidingPhase::Entering(_)
    ));
    press_f(&mut app);
    assert!(matches!(
        hidden(&app, player).unwrap().phase,
        HidingPhase::Leaving(_)
    ));
    settle(&mut app);
    assert_eq!(hidden(&app, player), None);
    let outside = transform(&app, player);
    assert!(outside.translation.distance(Vec3::new(0.0, EYE, -1.05)) < 0.001);
    assert!(same_angle(yaw(&outside), PI));
}

#[test]
fn f_during_leaving_reverses_back_into_hiding() {
    let (mut app, player) = app();
    let locker = spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    settle(&mut app);
    press_f(&mut app);
    assert!(matches!(
        hidden(&app, player).unwrap().phase,
        HidingPhase::Leaving(_)
    ));
    press_f(&mut app);
    settle(&mut app);
    let state = hidden(&app, player).unwrap();
    assert!(state.settled());
    assert_eq!(state.height, EYE);
    let eye = HidingSpot::Locker.eye(&transform(&app, locker));
    assert!(transform(&app, player).translation.distance(eye) < 0.001);
}

#[test]
fn movement_stays_locked_until_leaving_finishes() {
    let (mut app, player) = app();
    spot(&mut app, HidingSpot::Locker, Vec3::new(0.0, 0.0, -2.0), PI);
    aim(&mut app, player, Vec3::new(0.0, 1.0, -2.0));
    press_f(&mut app);
    settle(&mut app);
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::KeyF);
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::ShiftLeft);
    }
    let mut frames = 0;
    while hidden(&app, player).is_some() {
        app.update();
        frames += 1;
        assert!(frames <= 10, "leaving did not finish");
        if hidden(&app, player).is_some() {
            let input = app.world().get::<PlayerInput>(player).unwrap();
            assert_eq!(input.movement, Vec2::ZERO);
            assert!(!input.running);
            assert!(transform(&app, player).translation.z <= -1.05 + 0.001);
        }
    }
    hold(&mut app, &[KeyCode::KeyW], 2);
    assert!(transform(&app, player).translation.z > -1.05 + 0.3);
}
