use std::{
    f32::consts::{FRAC_PI_2, PI},
    time::Duration,
};

use bevy::{prelude::*, time::TimeUpdateStrategy, world_serialization::WorldAssetRoot};
use gameplay::facility::Module;
use gameplay::{
    devices::{Detector, DeviceCue, DevicePlugin, Flashbangs, Flashed},
    facility::{
        build_first_floor, spawn_first_floor_actors, Door, DoorLock, DoorOf, DoorPlugin, DoorRef,
        DoorState, DoorSwing, ExitDoor, Passage, PropCollider, Room, ToggleDoor,
    },
    fuses::FusePanel,
    hiding::{Hidden, HidingPhase, HidingPlugin, HidingSpot, UseHidingSpot},
    monster::{
        Caught, Monster, MonsterAssets, MonsterCue, MonsterCueKind, MonsterFigure,
        MonsterNavigation, MonsterPlugin, MonsterViewPlugin, PatrolRng, MONSTER_HEAD_JOINT,
    },
    player::{
        door_frames, door_panel, PlayerController, PlayerControlsEnabled, PlayerInput, Stamina,
        StaminaState, RUN_SPEED,
    },
    presets::{GameMode, GamePreset},
};

mod common;

const FRAME: f32 = 0.1;
const EYE: f32 = 1.6;

fn monster_app_with(frame: Duration) -> App {
    let mut app = common::level_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(frame))
        .insert_resource(PlayerControlsEnabled::default())
        .add_plugins((MonsterPlugin, DoorPlugin));
    app
}

fn monster_app() -> App {
    monster_app_with(Duration::from_millis(100))
}

fn spawn_monster(app: &mut App, transform: Transform) -> Entity {
    app.world_mut()
        .spawn((Monster::default(), PatrolRng::new(common::SEED), transform))
        .id()
}

fn spawn_player(app: &mut App, position: Vec3) -> Entity {
    app.world_mut()
        .spawn((PlayerController, Transform::from_translation(position)))
        .id()
}

fn navigation(app: &App, monster: Entity) -> MonsterNavigation {
    app.world()
        .get::<Monster>(monster)
        .unwrap()
        .debug_navigation()
}

fn position(app: &App, entity: Entity) -> Vec2 {
    app.world()
        .get::<Transform>(entity)
        .unwrap()
        .translation
        .xz()
}

fn caught(app: &App, player: Entity) -> Option<Caught> {
    app.world().get::<Caught>(player).copied()
}

fn hidden(app: &App, player: Entity) -> Option<Hidden> {
    app.world().get::<Hidden>(player).copied()
}

fn leaving(app: &App, player: Entity) -> bool {
    hidden(app, player).is_some_and(|hidden| matches!(hidden.phase, HidingPhase::Leaving(_)))
}

fn teleport(app: &mut App, entity: Entity, position: Vec3) {
    app.world_mut()
        .get_mut::<Transform>(entity)
        .unwrap()
        .translation = position;
}

fn hide(app: &mut App, player: Entity) {
    let spot = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(player).insert(Hidden {
        spot,
        height: 1.0,
        phase: HidingPhase::Hidden,
    });
}

fn walk(app: &mut App, player: Entity) {
    app.world_mut()
        .get_mut::<PlayerInput>(player)
        .unwrap()
        .movement = Vec2::Y;
}

fn sprint(app: &mut App, player: Entity) {
    walk(app, player);
    app.world_mut().get_mut::<Stamina>(player).unwrap().state = StaminaState::Sprinting;
}

#[derive(Resource, Default)]
struct Cues(Vec<(Entity, MonsterCueKind)>);

fn collect_cues(mut sounds: MessageReader<MonsterCue>, mut cues: ResMut<Cues>) {
    cues.0
        .extend(sounds.read().map(|cue| (cue.monster, cue.kind)));
}

fn count(app: &App, source: Entity, sound: MonsterCueKind) -> usize {
    app.world()
        .resource::<Cues>()
        .0
        .iter()
        .filter(|&&(from, cue)| from == source && cue == sound)
        .count()
}

#[test]
fn debug_navigation_reports_the_route_and_pursuit_without_changing_them() {
    let mut app = monster_app();
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -25.0, 5.0, 5.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    assert_eq!(
        navigation(&app, monster),
        MonsterNavigation {
            waypoints: vec![],
            destination: None,
            last_sensed: None,
            long_range: false,
            running: false,
        }
    );
    spawn_player(&mut app, Vec3::new(0.0, EYE, -15.0));
    app.update();
    let chase = navigation(&app, monster);
    assert_eq!(chase.last_sensed, Some(Vec2::new(0.0, -15.0)));
    assert!(chase.running);
    assert!(!chase.long_range);
    assert_eq!(chase.destination, chase.waypoints.last().copied());
    assert!(chase
        .destination
        .is_some_and(|goal| goal.distance(Vec2::new(0.0, -15.0)) < 0.5));
    assert_eq!(navigation(&app, monster), chase);
}

#[test]
fn presence_repeats_while_monster_waits_without_a_route() {
    let mut app = monster_app();
    app.init_resource::<Cues>()
        .add_systems(PostUpdate, collect_cues);
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    for _ in 0..190 {
        app.update();
    }
    assert!(app.world().resource::<Cues>().0.is_empty());
    for _ in 190..500 {
        app.update();
    }
    assert_eq!(count(&app, monster, MonsterCueKind::Presence), 2);
    assert!(!app
        .world()
        .resource::<Cues>()
        .0
        .iter()
        .any(|(_, cue)| matches!(cue, MonsterCueKind::Step(_))));
}

fn measure_chase(mode: GameMode) -> Vec<f32> {
    let mut app = monster_app();
    app.insert_resource(GamePreset::of(mode));
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -45.0, 5.0, 5.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    spawn_player(&mut app, Vec3::new(0.0, EYE, -35.0));
    app.update();
    assert!(navigation(&app, monster).last_sensed.is_some());
    assert!(navigation(&app, monster).running);
    assert!(!navigation(&app, monster).long_range);
    let mut previous = position(&app, monster);
    let mut steps = Vec::new();
    for _ in 0..8 {
        app.update();
        let current = position(&app, monster);
        steps.push(current.distance(previous));
        previous = current;
    }
    steps
}

#[test]
fn chase_runs_one_meter_per_second_faster_than_a_sprint() {
    let chase = GamePreset::of(GameMode::Normal)
        .rules()
        .monster
        .chase_speed
        .value();
    assert!((chase - RUN_SPEED - 1.0).abs() < 0.001);
    for travelled in measure_chase(GameMode::Normal) {
        assert!(
            (travelled - chase * FRAME).abs() < 0.01,
            "chase must keep full speed on a clear line, travelled {travelled}"
        );
        assert!(travelled > RUN_SPEED * FRAME);
    }
}

#[test]
fn insane_chase_runs_at_double_sprint_speed() {
    let chase = GamePreset::of(GameMode::Insane)
        .rules()
        .monster
        .chase_speed
        .value();
    assert!((chase - RUN_SPEED * 2.0).abs() < 0.001);
    for travelled in measure_chase(GameMode::Insane) {
        assert!(
            (travelled - chase * FRAME).abs() < 0.01,
            "insane chase must keep full speed on a clear line, travelled {travelled}"
        );
    }
}

#[test]
fn chase_turns_toward_a_heard_sprint_quickly_but_without_snapping() {
    let mut app = monster_app();
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -5.0, 5.0, 15.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, 7.0));
    sprint(&mut app, player);
    let mut previous = Quat::IDENTITY;
    let mut faced_at = None;
    for frame in 0..8 {
        app.update();
        assert!(navigation(&app, monster).last_sensed.is_some());
        let transform = app.world().get::<Transform>(monster).unwrap();
        let turned = previous.angle_between(transform.rotation);
        assert!(turned <= FRAC_PI_2, "turn must be rate limited, not a snap");
        previous = transform.rotation;
        if faced_at.is_none() && (transform.rotation * Vec3::NEG_Z).z > 0.98 {
            faced_at = Some(frame);
        }
    }
    let faced_at = faced_at.expect("monster must face the sprint it heard");
    assert!(faced_at <= 5);
    assert!(app.world().get::<Transform>(monster).unwrap().translation.z > 1.0);
}

#[test]
fn search_scans_near_the_last_sensed_position_then_disengages() {
    let mut app = monster_app();
    app.world_mut()
        .spawn(Room(Rect::new(-6.0, -16.0, 6.0, 4.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -6.0));
    app.update();
    let last = navigation(&app, monster).last_sensed.unwrap();
    assert_eq!(last, Vec2::new(0.0, -6.0));
    hide(&mut app, player);
    teleport(&mut app, player, Vec3::new(5.0, EYE, 3.0));
    let mut pursuing_frames = 0;
    let mut search_targets: Vec<Vec2> = Vec::new();
    for frame in 0..140 {
        app.update();
        let state = navigation(&app, monster);
        match state.last_sensed {
            Some(sensed) => {
                assert_eq!(sensed, last, "hidden player must not be tracked");
                pursuing_frames = frame + 1;
                if let Some(target) = state.destination {
                    if target.distance(last) > 0.5
                        && search_targets
                            .iter()
                            .all(|seen| seen.distance(target) > 0.5)
                    {
                        search_targets.push(target);
                    }
                }
            }
            None => assert!(state.waypoints.is_empty()),
        }
        assert!(caught(&app, player).is_none());
    }
    assert!(
        (60..=100).contains(&pursuing_frames),
        "search must persist for a while, then end without new evidence: {pursuing_frames}"
    );
    assert!(
        search_targets.len() >= 2,
        "search must scan several points around the last sensed position"
    );
    assert!(search_targets
        .iter()
        .all(|target| target.distance(last) <= 3.5));
    assert!(navigation(&app, monster).last_sensed.is_none());
}

#[test]
fn player_sensed_again_near_the_lost_position_ends_the_search() {
    let mut app = monster_app();
    app.world_mut()
        .spawn(Room(Rect::new(-6.0, -16.0, 6.0, 4.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -6.0));
    app.update();
    let last = navigation(&app, monster).last_sensed.unwrap();
    hide(&mut app, player);
    let mut searching = false;
    for _ in 0..40 {
        app.update();
        let state = navigation(&app, monster);
        assert_eq!(state.last_sensed, Some(last));
        if state
            .destination
            .is_some_and(|target| target.distance(last) > 0.5)
        {
            searching = true;
            break;
        }
    }
    assert!(searching);
    app.world_mut().entity_mut(player).remove::<Hidden>();
    teleport(&mut app, player, Vec3::new(0.2, EYE, -6.2));
    sprint(&mut app, player);
    app.update();
    let state = navigation(&app, monster);
    assert_eq!(state.last_sensed, Some(Vec2::new(0.2, -6.2)));
    assert!(state
        .destination
        .is_none_or(|goal| goal.distance(Vec2::new(0.2, -6.2)) <= 0.5));
    for _ in 0..10 {
        app.update();
    }
    assert!(
        caught(&app, player).is_some(),
        "monster must chase a player it senses again"
    );
}

#[test]
fn hidden_player_in_plain_view_and_reach_is_never_sensed_or_attacked() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -0.5));
    hide(&mut app, player);
    sprint(&mut app, player);
    for _ in 0..30 {
        app.update();
        let state = navigation(&app, monster);
        assert!(state.last_sensed.is_none());
        assert!(state.waypoints.is_empty());
        assert!(caught(&app, player).is_none());
    }
}

#[test]
fn hiding_before_contact_keeps_a_chased_player_safe() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -8.0, 5.0, 5.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -3.0));
    app.update();
    assert!(navigation(&app, monster).last_sensed.is_some());
    hide(&mut app, player);
    let mut reached = false;
    for _ in 0..60 {
        app.update();
        reached |= position(&app, monster).distance(Vec2::new(0.0, -3.0)) <= 2.0;
        assert!(caught(&app, player).is_none());
    }
    assert!(
        reached,
        "monster must reach the hiding player without a catch"
    );
}

#[test]
fn witnessed_hiding_is_pulled_out_into_the_scripted_catch() {
    let mut app = monster_app();
    app.add_plugins(HidingPlugin);
    app.world_mut().spawn(Room(Rect::new(-5.0, -8.0, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(0.0, 0.0, -4.0)))
        .id();
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -2.9));
    app.update();
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(hidden(&app, player).is_some());
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(
        !leaving(&app, player),
        "a witnessed player cannot leave on their own"
    );
    let mut settled = false;
    let mut pulled_out = false;
    for _ in 0..100 {
        app.update();
        settled |= hidden(&app, player).is_some_and(|hidden| hidden.settled());
        pulled_out |= leaving(&app, player);
        if caught(&app, player).is_some() {
            break;
        }
    }
    assert!(settled, "monster must reach a fully hidden player");
    assert!(
        pulled_out,
        "monster must pull the player out before attacking"
    );
    assert_eq!(caught(&app, player).unwrap().monster, monster);
    assert!(hidden(&app, player).is_none());
    let exit = HidingSpot::Locker.exit(app.world().get::<Transform>(spot).unwrap());
    assert_eq!(position(&app, player), exit);
}

#[test]
fn unseen_hiding_remains_safe_even_when_monster_later_approaches() {
    let mut app = monster_app();
    app.add_plugins(HidingPlugin);
    app.world_mut().spawn(Room(Rect::new(-5.0, -8.0, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(0.0, 0.0, -4.0)))
        .id();
    let monster = spawn_monster(
        &mut app,
        Transform::from_rotation(Quat::from_rotation_y(PI)),
    );
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -2.9));
    app.update();
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(hidden(&app, player).is_some());
    teleport(&mut app, monster, Vec3::new(0.0, 0.0, -3.2));
    for _ in 0..40 {
        app.update();
    }
    assert!(hidden(&app, player).is_some_and(|hidden| hidden.settled()));
    assert!(caught(&app, player).is_none());
}

#[test]
fn hiding_entry_behind_a_wall_is_not_witnessed() {
    let mut app = monster_app();
    app.add_plugins(HidingPlugin);
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -5.0, 5.0, -0.5)));
    app.world_mut().spawn(Room(Rect::new(-5.0, -0.5, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(0.0, 0.0, -2.4)))
        .id();
    spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -1.3));
    app.update();
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(hidden(&app, player).is_some());
    for _ in 0..20 {
        app.update();
    }
    assert!(hidden(&app, player).is_some());
    assert!(caught(&app, player).is_none());
}

#[test]
fn caught_player_cannot_move_look_or_hide_and_faces_the_full_attack() {
    let mut app = common::player_app(Duration::from_millis(100));
    app.add_plugins((MonsterPlugin, DoorPlugin, HidingPlugin));
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(1.0, 0.0, -1.7)))
        .id();
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let anchor = Vec3::new(0.0, EYE, -1.7);
    let player = spawn_player(&mut app, anchor);
    let attack = Caught::new(monster).remaining;
    let mut previous = Quat::IDENTITY;
    let mut finished_at = None;
    for frame in 0..30 {
        {
            let mut input = app.world_mut().get_mut::<PlayerInput>(player).unwrap();
            input.movement = Vec2::Y;
            input.look = Vec2::new(40.0, 10.0);
        }
        app.world_mut()
            .write_message(UseHidingSpot { player, spot });
        app.update();
        let caught = caught(&app, player).expect("contact must catch the player on the same frame");
        assert_eq!(caught.monster, monster);
        let view = *app.world().get::<Transform>(player).unwrap();
        assert_eq!(view.translation, anchor, "caught player must not move");
        assert!(hidden(&app, player).is_none());
        assert!(
            previous.angle_between(view.rotation) <= 1.0,
            "caught view must turn at a bounded rate and ignore look input"
        );
        previous = view.rotation;
        if caught.finished() && finished_at.is_none() {
            finished_at = Some(frame);
        }
    }
    let finished_at = finished_at.expect("scripted attack must finish") as f32;
    let frames = attack / FRAME;
    assert!(finished_at >= frames - 1.0);
    assert!(finished_at <= frames + 1.0);
    let view = app.world().get::<Transform>(player).unwrap();
    let pose = app.world().get::<Transform>(monster).unwrap();
    assert!(
        view.forward().y > 0.3,
        "caught view must look up at the head"
    );
    let toward_player = (view.translation - pose.translation).xz().normalize();
    assert!(pose.forward().xz().dot(toward_player) > 0.95);
}

#[test]
fn jumpscare_aims_at_the_live_head_joint_when_the_scene_is_loaded() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let joint = Vec3::new(0.3, 2.3, -1.1);
    app.world_mut()
        .spawn((
            Monster::default(),
            PatrolRng::default(),
            Transform::IDENTITY,
        ))
        .with_children(|children| {
            children.spawn(Transform::default()).with_children(|bones| {
                bones.spawn((
                    Name::new(MONSTER_HEAD_JOINT),
                    GlobalTransform::from_translation(joint),
                ));
            });
        });
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -1.7));
    for _ in 0..12 {
        app.update();
    }
    assert!(caught(&app, player).is_some());
    let view = app.world().get::<Transform>(player).unwrap();
    assert!(view.forward().dot((joint - view.translation).normalize()) > 0.99);
}

#[test]
fn still_player_behind_the_monster_is_caught_only_within_reach() {
    for (distance, expected) in [(1.7, true), (1.95, false)] {
        let mut app = monster_app();
        app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
        let monster = spawn_monster(&mut app, Transform::IDENTITY);
        let player = spawn_player(&mut app, Vec3::new(0.0, EYE, distance));
        for _ in 0..10 {
            app.update();
        }
        assert_eq!(
            caught(&app, player).is_some(),
            expected,
            "still player behind the monster at {distance} m"
        );
        assert_eq!(navigation(&app, monster).last_sensed.is_some(), expected);
    }
}

#[test]
fn chase_stops_short_of_the_player_at_contact_range() {
    let mut app = monster_app();
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -12.0, 5.0, 5.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -7.0));
    let mut caught_at = None;
    for frame in 0..30 {
        app.update();
        if caught(&app, player).is_some() {
            caught_at = Some(frame);
            break;
        }
    }
    assert!(caught_at.is_some());
    let distance = position(&app, monster).distance(position(&app, player));
    assert!(distance > 1.0, "monster must not walk into the player");
    assert!(distance <= 2.0);
}

#[test]
fn sprint_is_heard_through_walls_within_range_but_not_outside_it() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-2.5, -5.0, 2.5, 0.0)));
    app.world_mut().spawn(Room(Rect::new(-2.5, 0.0, 2.5, 5.0)));
    let monster = spawn_monster(&mut app, Transform::from_xyz(0.0, 0.0, 1.0));
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -1.5));
    sprint(&mut app, player);
    app.update();
    assert_eq!(
        navigation(&app, monster).last_sensed,
        Some(Vec2::new(0.0, -1.5))
    );
    teleport(&mut app, player, Vec3::new(0.0, EYE, -4.0));
    app.update();
    assert_eq!(
        navigation(&app, monster).last_sensed,
        Some(Vec2::new(0.0, -4.0))
    );
    teleport(&mut app, player, Vec3::new(0.0, EYE, -9.0));
    app.update();
    assert_eq!(
        navigation(&app, monster).last_sensed,
        Some(Vec2::new(0.0, -4.0))
    );
}

#[test]
fn walking_is_heard_through_walls_only_while_moving_within_walk_range() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-2.5, -5.0, 2.5, 0.0)));
    app.world_mut().spawn(Room(Rect::new(-2.5, 0.0, 2.5, 5.0)));
    let monster = spawn_monster(&mut app, Transform::from_xyz(0.0, 0.0, 1.0));
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -1.5));
    app.update();
    assert!(navigation(&app, monster).last_sensed.is_none());
    walk(&mut app, player);
    app.update();
    assert_eq!(
        navigation(&app, monster).last_sensed,
        Some(Vec2::new(0.0, -1.5))
    );
    teleport(&mut app, player, Vec3::new(0.0, EYE, -3.0));
    app.update();
    assert_eq!(
        navigation(&app, monster).last_sensed,
        Some(Vec2::new(0.0, -1.5))
    );
}

fn inside(position: Vec2, (center, half, yaw): (Vec2, Vec2, f32)) -> bool {
    let delta = position - center;
    let (s, c) = yaw.sin_cos();
    let local = Vec2::new(c * delta.x - s * delta.y, s * delta.x + c * delta.y);
    local.x.abs() < half.x && local.y.abs() < half.y
}

#[test]
fn chase_routes_around_a_prop_from_an_off_grid_start_without_clipping() {
    let mut app = monster_app();
    let room = Rect::new(-5.0, -8.0, 5.0, 3.0);
    app.world_mut().spawn(Room(room));
    let prop = (Vec2::new(0.0, -0.9), Vec2::new(2.0, 0.2), 0.0);
    app.world_mut().spawn((
        PropCollider {
            center: Vec2::ZERO,
            half: prop.1,
        },
        Transform::from_xyz(prop.0.x, 0.0, prop.0.y),
    ));
    let monster = spawn_monster(&mut app, Transform::from_xyz(0.2, 0.0, -0.44));
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -4.0));
    let mut reached = false;
    for _ in 0..40 {
        app.update();
        let position = position(&app, monster);
        assert!(
            !inside(position, prop),
            "monster clipped the prop at {position}"
        );
        assert!(room.contains(position));
        if caught(&app, player).is_some() {
            reached = true;
            break;
        }
    }
    assert!(reached, "chase must reach the player around the prop");
}

#[test]
fn blocked_route_replans_around_a_new_obstacle_without_clipping() {
    let mut app = monster_app();
    let room = Rect::new(-5.0, -8.0, 5.0, 3.0);
    app.world_mut().spawn(Room(room));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -4.0));
    app.update();
    assert!(navigation(&app, monster)
        .destination
        .is_some_and(|goal| goal.distance(Vec2::new(0.0, -4.0)) < 0.5));
    let prop = (Vec2::new(0.0, -1.0), Vec2::new(2.0, 0.2), 0.0);
    app.world_mut().spawn((
        PropCollider {
            center: Vec2::ZERO,
            half: prop.1,
        },
        Transform::from_xyz(prop.0.x, 0.0, prop.0.y),
    ));
    let mut reached = false;
    for _ in 0..600 {
        app.update();
        let position = position(&app, monster);
        assert!(
            !inside(position, prop),
            "monster clipped the prop at {position}"
        );
        assert!(room.contains(position));
        if caught(&app, player).is_some() {
            reached = true;
            break;
        }
    }
    assert!(reached, "stalled route must replan around the obstacle");
}

#[test]
fn sprint_is_heard_from_behind_but_walking_at_that_distance_is_not() {
    let mut app = monster_app();
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -5.0, 5.0, 15.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, 7.0));
    walk(&mut app, player);
    app.update();
    assert!(navigation(&app, monster).last_sensed.is_none());
    sprint(&mut app, player);
    app.update();
    assert!(navigation(&app, monster).last_sensed.is_some());
}

#[test]
fn insane_hears_walking_where_normal_does_not() {
    for (mode, heard) in [(GameMode::Normal, false), (GameMode::Insane, true)] {
        let mut app = monster_app();
        app.insert_resource(GamePreset::of(mode));
        app.world_mut()
            .spawn(Room(Rect::new(-5.0, -5.0, 5.0, 15.0)));
        let monster = spawn_monster(&mut app, Transform::IDENTITY);
        let player = spawn_player(&mut app, Vec3::new(0.0, EYE, 5.0));
        walk(&mut app, player);
        app.update();
        assert_eq!(
            navigation(&app, monster).last_sensed.is_some(),
            heard,
            "{mode:?}"
        );
    }
}

#[test]
fn walls_block_sight_and_contact_is_a_guaranteed_catch() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-2.5, -2.5, 2.5, 0.0)));
    app.world_mut().spawn(Room(Rect::new(-2.5, 0.0, 2.5, 2.5)));
    let monster = spawn_monster(&mut app, Transform::from_xyz(0.0, 0.0, 0.8));
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -0.8));
    app.update();
    assert!(navigation(&app, monster).last_sensed.is_none());
    teleport(&mut app, player, Vec3::new(0.0, EYE, 0.25));
    app.update();
    assert_eq!(caught(&app, player).unwrap().monster, monster);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .x = 2.0;
    for _ in 0..20 {
        app.update();
        assert!(caught(&app, player).is_some());
    }
    assert!(caught(&app, player).unwrap().finished());
}

#[test]
fn pursuing_monster_cannot_catch_through_a_wall_within_attack_reach() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-2.5, 0.0, 2.5, 5.0)));
    app.world_mut().spawn(Room(Rect::new(-2.5, -5.0, 2.5, 0.0)));
    let monster = spawn_monster(
        &mut app,
        Transform::from_xyz(0.0, 0.0, 0.25).with_rotation(Quat::from_rotation_y(PI)),
    );
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, 3.0));
    app.update();
    assert!(navigation(&app, monster).last_sensed.is_some());
    teleport(&mut app, player, Vec3::new(0.0, EYE, -0.8));
    assert!(position(&app, monster).distance(position(&app, player)) < 1.7);
    for _ in 0..3 {
        app.update();
        assert!(caught(&app, player).is_none());
    }
}

#[test]
fn detection_and_attack_emit_one_source_attached_cue_each() {
    let mut app = monster_app();
    app.init_resource::<Cues>()
        .add_systems(PostUpdate, collect_cues);
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -0.7));
    for _ in 0..12 {
        app.update();
    }
    assert_eq!(count(&app, monster, MonsterCueKind::Detected), 1);
    assert_eq!(count(&app, monster, MonsterCueKind::Attack), 1);
    assert!(caught(&app, player).is_some());
}

#[test]
fn patrol_schedules_a_distant_room_after_short_legs() {
    let mut running = Vec::new();
    let mut walking = Vec::new();
    for seed in 0..16 {
        let mut app = monster_app();
        app.insert_resource(GamePreset::of(GameMode::Insane));
        let passage = app.world_mut().spawn(Passage(Vec2::ZERO)).id();
        app.world_mut()
            .spawn(Room(Rect::new(-30.0, -1.25, 0.0, 1.25)))
            .with_related::<DoorOf>(DoorRef(passage));
        app.world_mut()
            .spawn(Room(Rect::new(0.0, -1.25, 30.0, 1.25)))
            .with_related::<DoorOf>(DoorRef(passage));
        let monster = app
            .world_mut()
            .spawn((
                Monster::default(),
                PatrolRng::new(seed),
                Transform::from_xyz(-15.0, 0.0, 0.0),
            ))
            .id();
        let mut short_legs = 0;
        let mut previous_destination = None;
        let mut planned = None;
        for _ in 0..2000 {
            app.update();
            let state = navigation(&app, monster);
            if state.destination != previous_destination && state.destination.is_some() {
                short_legs += 1;
            }
            previous_destination = state.destination;
            if state.long_range {
                planned = Some((position(&app, monster), state));
                break;
            }
        }
        let (from, state) = planned.expect("patrol must schedule a long leg");
        assert!(short_legs > 1, "short legs must come first: {short_legs}");
        assert!(state.destination.unwrap().distance(from) >= 24.0);
        let runs = state.running;
        for _ in 0..40 {
            app.update();
        }
        let travelled = position(&app, monster).distance(from);
        if runs {
            running.push(travelled);
        } else {
            walking.push(travelled);
        }
    }
    let walk = GamePreset::of(GameMode::Insane)
        .rules()
        .monster
        .walk_speed
        .value()
        * 4.0;
    assert!(!running.is_empty() && !walking.is_empty());
    assert!(running.iter().all(|distance| *distance > walk));
    assert!(walking.iter().all(|distance| *distance < walk));
}

fn spawn_door(app: &mut App, position: Vec2, yaw: f32, locked: bool) -> Entity {
    let mut door = app.world_mut().spawn((
        Name::new("route door"),
        Door {
            position,
            rotation: Quat::from_rotation_y(yaw),
            frame: Module::WallDoorway,
            panel: Module::DoorPanel,
            state: DoorState::Closed,
        },
    ));
    if locked {
        door.insert(DoorLock);
    }
    door.id()
}

fn link_rooms(app: &mut App, door: Entity, rooms: impl IntoIterator<Item = Rect>) {
    for bounds in rooms {
        app.world_mut()
            .spawn(Room(bounds))
            .with_related::<DoorOf>(DoorRef(door));
    }
}

fn planned_waypoints(open: bool) -> Vec<Vec2> {
    let mut app = monster_app();
    let door = spawn_door(&mut app, Vec2::ZERO, FRAC_PI_2, false);
    if open {
        app.world_mut().get_mut::<Door>(door).unwrap().state = DoorState::Open;
        app.world_mut().get_mut::<DoorSwing>(door).unwrap().0 = FRAC_PI_2;
    }
    link_rooms(
        &mut app,
        door,
        [
            Rect::new(-5.0, -1.25, 0.0, 1.25),
            Rect::new(0.0, -1.25, 5.0, 1.25),
        ],
    );
    let monster = spawn_monster(&mut app, Transform::from_xyz(-2.5, 0.0, 0.0));
    let player = spawn_player(&mut app, Vec3::new(2.5, EYE, 0.0));
    sprint(&mut app, player);
    app.update();
    let state = navigation(&app, monster);
    assert_eq!(state.last_sensed, Some(Vec2::new(2.5, 0.0)));
    state.waypoints
}

#[test]
fn route_through_an_interior_door_reaches_the_other_room_whether_open_or_closed() {
    for open in [false, true] {
        let waypoints = planned_waypoints(open);
        assert_eq!(waypoints.last().copied(), Some(Vec2::new(2.5, 0.0)));
        assert!(waypoints.iter().all(|waypoint| waypoint.y.abs() < 1.25));
    }
}

#[derive(Resource, Default)]
struct DoorToggles(Vec<Entity>);

fn record_door_toggles(mut toggles: MessageReader<ToggleDoor>, mut recorded: ResMut<DoorToggles>) {
    recorded.0.extend(toggles.read().map(|toggle| toggle.0));
}

fn door_app(door_at: Vec2, yaw: f32, rooms: [Rect; 2]) -> (App, Entity) {
    let mut app = monster_app();
    app.init_resource::<DoorToggles>()
        .add_systems(PostUpdate, record_door_toggles);
    let door = spawn_door(&mut app, door_at, yaw, false);
    link_rooms(&mut app, door, rooms);
    (app, door)
}

fn toggles(app: &App) -> Vec<Entity> {
    app.world().resource::<DoorToggles>().0.clone()
}

fn door_state(app: &App, door: Entity) -> DoorState {
    app.world().get::<Door>(door).unwrap().state
}

#[test]
fn monster_walking_past_a_side_door_leaves_it_closed_and_keeps_moving() {
    let (mut app, door) = door_app(
        Vec2::new(1.25, 1.25),
        0.0,
        [
            Rect::new(-5.0, -1.25, 10.0, 1.25),
            Rect::new(0.0, 1.25, 2.5, 3.75),
        ],
    );
    let monster = spawn_monster(
        &mut app,
        Transform::from_xyz(-1.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    );
    spawn_player(&mut app, Vec3::new(8.0, EYE, 0.0));
    let mut closest = f32::MAX;
    let mut furthest = f32::MIN;
    for _ in 0..100 {
        app.update();
        let position = position(&app, monster);
        closest = closest.min(position.distance(Vec2::new(1.25, 1.25)));
        furthest = furthest.max(position.x);
    }
    assert!(closest < 1.5, "route must pass the side door");
    assert!(toggles(&app).is_empty());
    assert_eq!(door_state(&app, door), DoorState::Closed);
    assert!(furthest > 3.5, "monster must not wait at a side door");
}

#[test]
fn monster_opens_the_doorway_it_faces_on_its_route_and_crosses_without_snagging() {
    let (mut app, door) = door_app(
        Vec2::ZERO,
        FRAC_PI_2,
        [
            Rect::new(-5.0, -1.25, 0.0, 1.25),
            Rect::new(0.0, -1.25, 5.0, 1.25),
        ],
    );
    let monster = spawn_monster(
        &mut app,
        Transform::from_xyz(-2.5, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    );
    let player = spawn_player(&mut app, Vec3::new(4.5, EYE, 0.0));
    sprint(&mut app, player);
    let mut crossed = false;
    for _ in 0..200 {
        app.update();
        let position = position(&app, monster);
        let state = app.world().get::<Door>(door).unwrap();
        let swing = app.world().get::<DoorSwing>(door).unwrap();
        let mut blockers = door_frames(state).to_vec();
        blockers.push(door_panel(state, swing));
        assert!(
            !blockers.iter().any(|blocker| inside(position, *blocker)),
            "monster clipped the doorway at {position}"
        );
        if position.x > 2.3 {
            crossed = true;
            break;
        }
    }
    assert!(crossed, "monster must cross the opened doorway");
    assert_eq!(toggles(&app), vec![door]);
    for _ in 0..30 {
        app.update();
    }
    assert_eq!(toggles(&app), vec![door]);
    assert_eq!(door_state(&app, door), DoorState::Open);
}

#[test]
fn monster_turns_to_a_route_door_entered_from_a_corner_and_crosses() {
    let (mut app, door) = door_app(
        Vec2::ZERO,
        FRAC_PI_2,
        [
            Rect::new(-5.0, -1.25, 0.0, 1.25),
            Rect::new(0.0, -1.25, 5.0, 1.25),
        ],
    );
    let monster = spawn_monster(&mut app, Transform::from_xyz(-0.9, 0.0, -0.9));
    let player = spawn_player(&mut app, Vec3::new(4.5, EYE, 0.0));
    sprint(&mut app, player);
    let mut crossed = false;
    for _ in 0..300 {
        app.update();
        if position(&app, monster).x > 2.0 {
            crossed = true;
            break;
        }
    }
    assert_eq!(toggles(&app), vec![door]);
    assert_eq!(door_state(&app, door), DoorState::Open);
    assert!(
        crossed,
        "monster must not deadlock at a door entered from a corner"
    );
}

#[test]
fn distant_player_ahead_is_seen_but_cone_and_walls_still_limit_sight() {
    for (rooms, player_at, seen) in [
        (vec![Rect::new(-3.0, -60.0, 3.0, 3.0)], -50.0, true),
        (vec![Rect::new(-3.0, -60.0, 3.0, 3.0)], 2.5, false),
        (
            vec![
                Rect::new(-3.0, -60.0, 3.0, -10.0),
                Rect::new(-3.0, -10.0, 3.0, 3.0),
            ],
            -50.0,
            false,
        ),
    ] {
        let mut app = monster_app();
        for bounds in rooms {
            app.world_mut().spawn(Room(bounds));
        }
        let monster = spawn_monster(&mut app, Transform::IDENTITY);
        spawn_player(&mut app, Vec3::new(0.0, EYE, player_at));
        app.update();
        let sensed = navigation(&app, monster).last_sensed;
        assert_eq!(sensed.is_some(), seen, "player at z {player_at}");
        if seen {
            assert_eq!(sensed, Some(Vec2::new(0.0, player_at)));
        }
    }
}

#[test]
fn patrol_opens_traverses_and_leaves_an_unlocked_door_open() {
    let (mut app, door) = door_app(
        Vec2::ZERO,
        FRAC_PI_2,
        [
            Rect::new(-5.0, -1.25, 0.0, 1.25),
            Rect::new(0.0, -1.25, 5.0, 1.25),
        ],
    );
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )));
    let monster = spawn_monster(&mut app, Transform::from_xyz(-2.5, 0.0, 0.0));
    let mut opened = false;
    let mut crossed = false;
    for _ in 0..1100 {
        app.update();
        opened |= door_state(&app, door) == DoorState::Open;
        crossed |= opened && position(&app, monster).x > 1.8;
        if crossed {
            break;
        }
    }
    assert!(opened, "patrol must open the door");
    assert!(crossed, "patrol must cross the door");
    for _ in 0..100 {
        app.update();
    }
    assert_eq!(door_state(&app, door), DoorState::Open);
}

#[test]
fn locked_exit_is_not_a_patrol_neighbor() {
    let mut app = monster_app_with(Duration::from_millis(16));
    let door = spawn_door(&mut app, Vec2::ZERO, FRAC_PI_2, true);
    link_rooms(
        &mut app,
        door,
        [
            Rect::new(-5.0, -1.25, 0.0, 1.25),
            Rect::new(0.0, -1.25, 5.0, 1.25),
        ],
    );
    let monster = spawn_monster(&mut app, Transform::from_xyz(-2.5, 0.0, 0.0));
    for _ in 0..200 {
        app.update();
    }
    assert!(position(&app, monster).x < 0.0);
    assert!(navigation(&app, monster).destination.is_none());
    assert_eq!(door_state(&app, door), DoorState::Closed);
}

#[test]
fn authored_floor_schedules_a_reachable_long_patrol_destination() {
    let mut app = monster_app();
    app.add_systems(Startup, build_first_floor);
    app.update();
    let monster = spawn_monster(&mut app, Transform::from_xyz(-2.5, 0.0, 0.0));
    let mut planned = None;
    for _ in 0..3000 {
        let before = position(&app, monster);
        app.update();
        let state = navigation(&app, monster);
        if state.long_range {
            planned = Some((before, state));
            break;
        }
    }
    let (from, state) = planned.expect("authored patrol must schedule a long leg");
    assert!(state.destination.unwrap().distance(from) >= 23.5);
    assert!(!state.waypoints.is_empty());
}

#[test]
fn monster_crosses_office_east_door_to_reach_a_witnessed_table() {
    let mut app = common::player_app(Duration::from_millis(100));
    app.add_plugins((DoorPlugin, HidingPlugin, MonsterPlugin))
        .add_systems(Startup, build_first_floor);
    app.finish();
    app.cleanup();
    app.update();
    let world = app.world_mut();
    let table = world
        .query::<(Entity, &HidingSpot, &Transform)>()
        .iter(world)
        .find(|(_, kind, transform)| {
            *kind == &HidingSpot::Table
                && transform.translation.xz().distance(Vec2::new(3.75, -18.75)) < 0.1
        })
        .map(|(entity, _, _)| entity)
        .expect("office table");
    let place = *world.get::<Transform>(table).unwrap();
    let exit = HidingSpot::Table.exit(&place);
    let door = world
        .query::<(Entity, &Door)>()
        .iter(world)
        .find(|(_, door)| door.position == Vec2::new(5.0, -16.25))
        .map(|(entity, _)| entity)
        .expect("office east door");
    let player = spawn_player(&mut app, Vec3::new(exit.x, EYE, exit.y));
    let monster = spawn_monster(
        &mut app,
        Transform::from_xyz(exit.x + 3.0, 0.0, exit.y)
            .with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    );
    app.world_mut().write_message(UseHidingSpot {
        player,
        spot: table,
    });
    app.update();
    assert!(hidden(&app, player).is_some());
    assert_eq!(navigation(&app, monster).last_sensed, Some(exit));
    teleport(&mut app, monster, Vec3::new(5.0, 0.0, -12.5));
    let mut closest = f32::MAX;
    for _ in 0..180 {
        app.update();
        closest = closest.min(position(&app, monster).distance(exit));
        if caught(&app, player).is_some() {
            break;
        }
    }
    assert!(
        caught(&app, player).is_some(),
        "monster did not reach the witnessed office table, closest {closest}, door {:?}, route {:?}",
        door_state(&app, door),
        navigation(&app, monster).waypoints
    );
    assert_eq!(door_state(&app, door), DoorState::Open);
    assert!(hidden(&app, player).is_none());
}

#[test]
fn authored_first_floor_patrol_reaches_another_room() {
    let mut app = monster_app_with(Duration::from_millis(16));
    app.init_resource::<Cues>()
        .add_systems(Startup, build_first_floor)
        .add_systems(PostUpdate, collect_cues);
    app.update();
    let monster = spawn_monster(&mut app, Transform::from_xyz(-2.5, 0.0, 0.0));
    let world = app.world_mut();
    let lab = world
        .query::<&Room>()
        .iter(world)
        .find(|room| room.0.contains(Vec2::new(-2.5, 0.0)))
        .expect("monster spawn room")
        .0;
    let mut crossed = false;
    let mut opened = false;
    let mut previous_rotation = app.world().get::<Transform>(monster).unwrap().rotation;
    for _ in 0..1800 {
        app.update();
        let transform = *app.world().get::<Transform>(monster).unwrap();
        assert!(
            previous_rotation.angle_between(transform.rotation) <= 0.1,
            "patrol must not snap its facing at a waypoint or room boundary"
        );
        previous_rotation = transform.rotation;
        crossed |= !lab.contains(transform.translation.xz());
        let world = app.world_mut();
        opened |= world
            .query::<&Door>()
            .iter(world)
            .any(|door| door.state == DoorState::Open);
        if crossed {
            break;
        }
    }
    assert!(opened, "authored patrol must open a door");
    assert!(crossed, "authored patrol must reach another room");
    assert!(
        app.world()
            .resource::<Cues>()
            .0
            .iter()
            .any(|(_, cue)| matches!(cue, MonsterCueKind::Step(_))),
        "moving patrol must emit timed footsteps"
    );
}

fn monster_assets() -> MonsterAssets {
    MonsterAssets {
        scene: default(),
        idle: default(),
        walk: default(),
        chase: default(),
        attack: default(),
    }
}

fn view_app() -> App {
    let mut app = common::level_app();
    app.init_resource::<Assets<AnimationGraph>>()
        .insert_resource(PlayerControlsEnabled::default())
        .add_plugins(MonsterViewPlugin);
    app
}

fn render_app() -> App {
    let mut app = view_app();
    app.insert_resource(monster_assets());
    app
}

fn scene_roots(app: &mut App, parent: Entity) -> Vec<Entity> {
    let world = app.world_mut();
    let mut scenes = world.query_filtered::<(Entity, &ChildOf), With<WorldAssetRoot>>();
    scenes
        .iter(world)
        .filter(|(_, child)| child.parent() == parent)
        .map(|(entity, _)| entity)
        .collect()
}

#[test]
fn rendered_monster_attaches_one_scene_once() {
    let mut app = render_app();
    app.add_plugins((MonsterPlugin, DoorPlugin))
        .add_systems(Startup, spawn_first_floor_actors);
    app.update();
    app.update();
    let world = app.world_mut();
    let monster = world
        .query_filtered::<Entity, With<Monster>>()
        .single(world)
        .unwrap();
    assert_eq!(scene_roots(&mut app, monster).len(), 1);
    assert_eq!(app.world().get::<Children>(monster).unwrap().len(), 1);
    let world = app.world_mut();
    assert_eq!(world.query::<&WorldAssetRoot>().iter(world).count(), 1);
}

#[test]
fn view_waits_for_monster_assets_before_attaching_a_scene() {
    let mut app = view_app();
    let figure = app.world_mut().spawn(MonsterFigure).id();
    app.update();
    app.update();
    assert!(scene_roots(&mut app, figure).is_empty());
    app.insert_resource(monster_assets());
    app.update();
    assert_eq!(scene_roots(&mut app, figure).len(), 1);
}

#[test]
fn first_floor_authors_player_and_monster_only_when_actors_are_spawned() {
    let mut app = common::level_app();
    app.add_systems(Startup, build_first_floor);
    app.update();
    let world = app.world_mut();
    assert_eq!(
        world
            .query_filtered::<Entity, With<PlayerController>>()
            .iter(world)
            .count(),
        0
    );
    assert_eq!(
        world
            .query_filtered::<Entity, With<Monster>>()
            .iter(world)
            .count(),
        0
    );
    app.world_mut()
        .run_system_cached(spawn_first_floor_actors)
        .unwrap();
    let world = app.world_mut();
    let mut players = world.query_filtered::<&Transform, With<PlayerController>>();
    let player = players.single(world).unwrap();
    assert_eq!(player.translation, Vec3::new(2.5, EYE, -27.5));
    let spawn = player.translation.xz();
    let mut exit_rooms = world.query::<(&Name, &Room)>();
    let exit_room = exit_rooms
        .iter(world)
        .find(|(name, _)| name.as_str() == "exit")
        .unwrap()
        .1;
    assert!(exit_room.0.contains(spawn));
    let mut outside_doors = world.query_filtered::<&Door, With<ExitDoor>>();
    let outside = outside_doors.single(world).unwrap().position;
    assert!((outside - spawn).length() < 5.0);
    assert!((outside - spawn).normalize().dot(Vec2::NEG_Y) > 0.8);
    let mut panels = world.query_filtered::<&Transform, With<FusePanel>>();
    let panel = panels.single(world).unwrap().translation.xz();
    assert!((panel - spawn).length() < 5.0);
    assert!((panel - spawn).normalize().dot(Vec2::NEG_Y) > 0.8);
    let mut monsters = world.query_filtered::<(Entity, &Transform), With<Monster>>();
    let (monster, pose) = monsters.single(world).unwrap();
    assert_eq!(pose.translation, Vec3::new(-2.5, 0.0, 0.0));
    assert!(world.get::<PatrolRng>(monster).is_some());
}

#[test]
fn figure_holds_the_final_attack_pose_once_without_restarting() {
    let mut app = render_app();
    let figure = app.world_mut().spawn(MonsterFigure).id();
    app.update();
    let roots = scene_roots(&mut app, figure);
    assert_eq!(roots.len(), 1);
    let animation_player = app
        .world_mut()
        .spawn((AnimationPlayer::default(), ChildOf(roots[0])))
        .id();
    app.update();
    let posed = app
        .world()
        .get::<AnimationPlayer>(animation_player)
        .unwrap();
    let playing: Vec<_> = posed.playing_animations().collect();
    assert_eq!(playing.len(), 1);
    let (_, attack) = playing[0];
    assert!(attack.is_paused());
    let paused_at = attack.seek_time();
    assert_eq!(paused_at, Caught::new(figure).remaining);
    app.update();
    let posed = app
        .world()
        .get::<AnimationPlayer>(animation_player)
        .unwrap();
    let playing: Vec<_> = posed.playing_animations().collect();
    assert_eq!(playing.len(), 1);
    assert_eq!(
        playing[0].1.seek_time(),
        paused_at,
        "an already-posed figure must not be restarted every frame"
    );
}

#[test]
fn monster_figure_never_gains_monster_ai_behavior() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let figure = app
        .world_mut()
        .spawn((MonsterFigure, Transform::from_xyz(1.0, 0.0, 1.0)))
        .id();
    spawn_player(&mut app, Vec3::new(1.0, EYE, -1.0));
    for _ in 0..10 {
        app.update();
    }
    assert!(app.world().get::<Monster>(figure).is_none());
    assert_eq!(
        app.world().get::<Transform>(figure).unwrap().translation,
        Vec3::new(1.0, 0.0, 1.0)
    );
}

fn flash_app() -> App {
    flash_app_with(|app| {
        app.add_plugins((DevicePlugin, MonsterPlugin, DoorPlugin));
    })
}

fn flash_app_with_hiding() -> App {
    flash_app_with(|app| {
        app.add_plugins((DevicePlugin, MonsterPlugin, DoorPlugin, HidingPlugin));
    })
}

fn flash_app_with(plugins: impl FnOnce(&mut App)) -> App {
    let mut app = common::player_app(Duration::from_millis(100));
    plugins(&mut app);
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn barrier(app: &mut App, z: f32) -> Entity {
    app.world_mut()
        .spawn((
            PropCollider {
                center: Vec2::ZERO,
                half: Vec2::new(5.0, 0.1),
            },
            Transform::from_xyz(0.0, 0.0, z),
        ))
        .id()
}

fn throw(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Right);
}

fn flashed(app: &App, player: Entity) -> bool {
    app.world().get::<Flashed>(player).is_some()
}

fn flash_outlasts_next_frame(app: &App, player: Entity) -> bool {
    app.world()
        .get::<Flashed>(player)
        .is_some_and(|flashed| flashed.remaining > FRAME * 1.5)
}

fn flash(app: &mut App, player: Entity) {
    throw(app);
    for _ in 0..10 {
        if flashed(app, player) {
            return;
        }
        app.update();
    }
    panic!("burst must blind the monster in view");
}

struct FlashScene {
    player: Entity,
    monster: Entity,
    barrier: Entity,
}

fn flash_scene(app: &mut App) -> FlashScene {
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -10.0, 5.0, 10.0)));
    let barrier = barrier(app, -5.0);
    let monster = spawn_monster(
        app,
        Transform::from_xyz(0.0, 0.0, -6.0).with_rotation(Quat::from_rotation_y(PI)),
    );
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, EYE, 0.0),
            Flashbangs(1),
        ))
        .id();
    app.update();
    assert_eq!(navigation(app, monster).last_sensed, Some(Vec2::ZERO));
    FlashScene {
        player,
        monster,
        barrier,
    }
}

#[test]
fn flashed_player_in_view_and_reach_is_not_sensed_or_caught_until_the_flash_ends() {
    let mut app = flash_app();
    let scene = flash_scene(&mut app);
    flash(&mut app, scene.player);
    app.world_mut().entity_mut(scene.barrier).despawn();
    let mut frames = 0;
    while flash_outlasts_next_frame(&app, scene.player) {
        let pose = *app.world().get::<Transform>(scene.monster).unwrap();
        let ahead = pose.translation + pose.forward() * 1.0 + Vec3::Y * EYE;
        teleport(&mut app, scene.player, ahead);
        app.update();
        frames += 1;
        assert!(frames < 80, "flash must expire");
        assert_eq!(
            navigation(&app, scene.monster).last_sensed,
            Some(Vec2::ZERO),
            "flashed player must not be tracked"
        );
        assert!(caught(&app, scene.player).is_none());
    }
    for _ in 0..10 {
        app.update();
        if caught(&app, scene.player).is_some() {
            break;
        }
    }
    assert_eq!(caught(&app, scene.player).unwrap().monster, scene.monster);
}

#[test]
fn a_flash_only_hides_the_player_from_monsters_hit_by_its_burst() {
    let mut app = flash_app();
    let scene = flash_scene(&mut app);
    let far_barrier = barrier(&mut app, 5.0);
    let missed = spawn_monster(&mut app, Transform::from_xyz(0.0, 0.0, 6.0));
    app.update();
    assert_eq!(navigation(&app, missed).last_sensed, Some(Vec2::ZERO));
    flash(&mut app, scene.player);
    app.world_mut().entity_mut(scene.barrier).despawn();
    app.world_mut().entity_mut(far_barrier).despawn();
    let moved = Vec3::new(1.5, EYE, 1.0);
    teleport(&mut app, scene.player, moved);
    for _ in 0..40 {
        app.update();
        if caught(&app, scene.player).is_some() {
            break;
        }
    }
    assert_eq!(caught(&app, scene.player).unwrap().monster, missed);
    assert_eq!(navigation(&app, missed).last_sensed, Some(moved.xz()));
    assert_eq!(
        navigation(&app, scene.monster).last_sensed,
        Some(Vec2::ZERO)
    );
}

#[test]
fn a_flash_with_no_monster_hit_does_not_hide_the_player() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, EYE, -1.0),
            Flashed::default(),
        ))
        .id();
    app.update();
    assert!(navigation(&app, monster).last_sensed.is_some());
    assert_eq!(caught(&app, player).unwrap().monster, monster);
}

#[test]
fn flash_mid_chase_keeps_the_last_known_search_without_tracking_or_contact() {
    let mut app = flash_app();
    let scene = flash_scene(&mut app);
    flash(&mut app, scene.player);
    app.world_mut().entity_mut(scene.barrier).despawn();
    teleport(&mut app, scene.player, Vec3::new(0.0, EYE, 6.0));
    let mut searched = false;
    let mut frames = 0;
    while flash_outlasts_next_frame(&app, scene.player) {
        app.update();
        frames += 1;
        assert!(frames < 80, "flash must expire");
        let state = navigation(&app, scene.monster);
        assert_eq!(
            state.last_sensed,
            Some(Vec2::ZERO),
            "search must persist through the flash without tracking"
        );
        searched |= state
            .destination
            .is_some_and(|target| target.distance(Vec2::ZERO) > 0.5);
        assert!(caught(&app, scene.player).is_none());
    }
    assert!(searched, "monster must search the last known position");
    {
        let mut input = app
            .world_mut()
            .get_mut::<PlayerInput>(scene.player)
            .unwrap();
        input.movement = Vec2::Y;
        input.running = true;
    }
    let mut tracked = false;
    for _ in 0..10 {
        app.update();
        tracked |= navigation(&app, scene.monster)
            .last_sensed
            .is_some_and(|sensed| sensed.distance(Vec2::ZERO) > 3.0);
    }
    assert!(
        tracked,
        "monster must sense the player again after the flash"
    );
}

#[test]
fn hiding_while_flashed_in_plain_view_is_not_witnessed_and_stays_safe() {
    let mut app = flash_app_with_hiding();
    let scene = flash_scene(&mut app);
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(2.0, 0.0, 0.0)))
        .id();
    flash(&mut app, scene.player);
    app.world_mut().entity_mut(scene.barrier).despawn();
    app.world_mut().write_message(UseHidingSpot {
        player: scene.player,
        spot,
    });
    app.update();
    assert!(hidden(&app, scene.player).is_some());
    for _ in 0..120 {
        app.update();
        assert!(caught(&app, scene.player).is_none());
        assert!(!leaving(&app, scene.player));
    }
    assert!(!flashed(&app, scene.player));
    assert!(hidden(&app, scene.player).is_some_and(|hidden| hidden.settled()));
}

#[test]
fn witnessed_hiding_is_not_pulled_out_while_flashed_but_is_after_expiry() {
    let mut app = flash_app_with_hiding();
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -10.0, 5.0, 10.0)));
    let barrier = barrier(&mut app, -5.0);
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::IDENTITY))
        .id();
    let monster = spawn_monster(
        &mut app,
        Transform::from_xyz(0.0, 0.0, -6.0).with_rotation(Quat::from_rotation_y(PI)),
    );
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, EYE, -2.0),
            Flashbangs(1),
        ))
        .id();
    app.update();
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(navigation(&app, monster).last_sensed.is_some());
    for _ in 0..6 {
        app.update();
    }
    assert!(hidden(&app, player).is_some_and(|hidden| hidden.settled()));
    flash(&mut app, player);
    app.world_mut().entity_mut(barrier).despawn();
    let mut frames = 0;
    while flash_outlasts_next_frame(&app, player) {
        app.update();
        frames += 1;
        assert!(frames < 80, "flash must expire");
        assert!(!leaving(&app, player));
        assert!(caught(&app, player).is_none());
    }
    for _ in 0..100 {
        app.update();
        if caught(&app, player).is_some() {
            break;
        }
    }
    assert_eq!(caught(&app, player).unwrap().monster, monster);
}

#[test]
fn flash_after_contact_does_not_cancel_the_catch() {
    let mut app = monster_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    let player = spawn_player(&mut app, Vec3::new(0.0, EYE, -1.0));
    for _ in 0..3 {
        app.update();
    }
    let started = caught(&app, player).unwrap();
    app.world_mut()
        .entity_mut(player)
        .insert(Flashed::default());
    for _ in 0..30 {
        app.update();
    }
    let caught = caught(&app, player).unwrap();
    assert_eq!(caught.monster, monster);
    assert!(caught.remaining < started.remaining);
    assert!(caught.finished());
}

#[derive(Resource, Default)]
struct Pulses(usize);

fn count_pulses(mut sounds: MessageReader<DeviceCue>, mut pulses: ResMut<Pulses>) {
    pulses.0 += sounds
        .read()
        .filter(|&&cue| cue == DeviceCue::DetectorNearby)
        .count();
}

#[test]
fn detector_cues_near_the_monster_never_alert_it() {
    let mut app = monster_app();
    app.add_plugins(DevicePlugin)
        .init_resource::<Pulses>()
        .add_systems(PostUpdate, count_pulses);
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = spawn_monster(&mut app, Transform::IDENTITY);
    app.world_mut().spawn((
        PlayerController,
        Detector::default(),
        Transform::from_xyz(0.0, EYE, 2.5),
    ));
    for _ in 0..60 {
        app.update();
        assert!(navigation(&app, monster).last_sensed.is_none());
    }
    assert!(app.world().resource::<Pulses>().0 >= 15);
}
