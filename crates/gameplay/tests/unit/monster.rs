use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy};

use super::{
    point, room_at, route, CHASE_SPEED, CHASE_TURN_SPEED, GRID, MODEL_LIFT, MODEL_PIVOT,
    MODEL_SCALE, SEARCH_RADIUS, SEARCH_TIME, TURN_SPEED,
};

#[test]
fn patrol_turns_at_a_bounded_rate_before_moving_in_a_new_direction() {
    let mut transform = Transform::IDENTITY;
    assert!(!super::turn_toward(
        &mut transform,
        Vec2::X,
        0.1,
        TURN_SPEED
    ));
    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    assert!((yaw + TURN_SPEED * 0.1).abs() < 0.001);
    for _ in 0..8 {
        super::turn_toward(&mut transform, Vec2::X, 0.1, TURN_SPEED);
    }
    assert!(super::turn_toward(&mut transform, Vec2::X, 0.1, TURN_SPEED));
    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    assert!((yaw + std::f32::consts::FRAC_PI_2).abs() < 0.001);
}

#[test]
fn patrol_turns_across_the_yaw_wrap_by_the_shortest_path() {
    let mut transform = Transform::from_rotation(Quat::from_rotation_y(2.6));
    let target = Vec2::new(0.5155, 0.8569).normalize();
    assert!(!super::turn_toward(
        &mut transform,
        target,
        0.01,
        TURN_SPEED
    ));
    let expected = Quat::from_rotation_y(2.6 + TURN_SPEED * 0.01);
    assert!(transform.rotation.angle_between(expected) < 0.001);
    for _ in 0..60 {
        super::turn_toward(&mut transform, target, 0.01, TURN_SPEED);
    }
    assert!(
        transform
            .rotation
            .angle_between(Quat::from_rotation_y(-2.6))
            < 0.01
    );
}

#[test]
fn presence_repeats_while_monster_waits_without_a_route() {
    use crate::levels::Monster;
    use game_audio::{PlaySourceSound, Sound};
    use rand_core::SeedableRng;

    #[derive(Resource, Default)]
    struct Heard(Vec<Sound>);

    fn collect(mut sounds: MessageReader<PlaySourceSound>, mut heard: ResMut<Heard>) {
        heard.0.extend(sounds.read().map(|cue| cue.sound));
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .insert_resource(super::PatrolRng(
            bevy_rand::prelude::ChaCha8Rng::seed_from_u64(1),
        ))
        .init_resource::<Heard>()
        .add_plugins(super::MonsterPlugin)
        .add_systems(PostUpdate, collect);
    app.world_mut()
        .spawn((Monster::default(), Transform::IDENTITY));
    for _ in 0..190 {
        app.update();
    }
    assert!(app.world().resource::<Heard>().0.is_empty());
    for _ in 190..500 {
        app.update();
    }
    let heard = &app.world().resource::<Heard>().0;
    assert_eq!(
        heard
            .iter()
            .filter(|&&cue| cue == Sound::MonsterPresence)
            .count(),
        2
    );
    assert!(!heard.iter().any(|cue| matches!(cue, Sound::MonsterStep(_))));
}

fn monster_app(seed: u64) -> App {
    use rand_core::SeedableRng;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .insert_resource(super::PatrolRng(
            bevy_rand::prelude::ChaCha8Rng::seed_from_u64(seed),
        ))
        .add_plugins(super::MonsterPlugin);
    app
}

fn hide(app: &mut App, player: Entity) {
    use crate::levels::{Hidden, HidingPhase};

    let spot = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(player).insert(Hidden {
        spot,
        height: 1.0,
        phase: HidingPhase::Hidden,
    });
}

fn monster_state(app: &App, monster: Entity) -> &crate::levels::Monster {
    app.world().get::<crate::levels::Monster>(monster).unwrap()
}

#[test]
fn chase_runs_one_meter_per_second_faster_than_a_sprint() {
    use crate::controller::{PlayerController, RUN_SPEED};
    use crate::levels::{Monster, Room};

    assert!((CHASE_SPEED - RUN_SPEED - 1.0).abs() < 0.001);
    let mut app = monster_app(7);
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -25.0, 5.0, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    app.world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -11.5)));
    app.update();
    assert!(monster_state(&app, monster).pursuit.is_some());
    let mut previous = app.world().get::<Transform>(monster).unwrap().translation;
    for _ in 0..8 {
        app.update();
        let current = app.world().get::<Transform>(monster).unwrap().translation;
        let travelled = current.distance(previous);
        assert!(
            (travelled - CHASE_SPEED * 0.1).abs() < 0.01,
            "chase must keep full speed on a clear line, travelled {travelled}"
        );
        assert!(travelled > RUN_SPEED * 0.1);
        previous = current;
    }
}

#[test]
fn chase_turns_toward_a_heard_sprint_quickly_but_at_a_bounded_rate() {
    use crate::controller::{PlayerController, PlayerInput, Stamina};
    use crate::levels::{Monster, Room};

    let mut transform = Transform::IDENTITY;
    assert!(!super::turn_toward(
        &mut transform,
        Vec2::X,
        0.1,
        CHASE_TURN_SPEED
    ));
    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    assert!((yaw + CHASE_TURN_SPEED * 0.1).abs() < 0.001);
    assert!(CHASE_TURN_SPEED > TURN_SPEED * 2.0);

    let mut app = monster_app(7);
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -5.0, 5.0, 15.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 7.0)))
        .id();
    app.world_mut()
        .get_mut::<PlayerInput>(player)
        .unwrap()
        .movement = Vec2::Y;
    app.world_mut()
        .get_mut::<Stamina>(player)
        .unwrap()
        .sprinting = true;
    let mut previous = Quat::IDENTITY;
    let mut fastest: f32 = 0.0;
    let mut faced_at = None;
    for frame in 0..8 {
        app.update();
        assert!(monster_state(&app, monster).pursuit.is_some());
        let transform = app.world().get::<Transform>(monster).unwrap();
        let turned = previous.angle_between(transform.rotation);
        assert!(turned <= CHASE_TURN_SPEED * 0.1 + 0.001);
        fastest = fastest.max(turned);
        previous = transform.rotation;
        if faced_at.is_none() && (transform.rotation * Vec3::NEG_Z).z > 0.98 {
            faced_at = Some(frame);
        }
    }
    assert!(fastest > TURN_SPEED * 0.1 + 0.01);
    let faced_at = faced_at.expect("monster must face the sprint it heard");
    assert!(faced_at as f32 * 0.1 <= std::f32::consts::PI / CHASE_TURN_SPEED + 0.1);
    assert!(app.world().get::<Transform>(monster).unwrap().translation.z > 1.0);
}

#[test]
fn search_scans_near_the_last_sensed_position_then_disengages() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Monster, Room};

    let mut app = monster_app(7);
    app.world_mut()
        .spawn(Room(Rect::new(-6.0, -16.0, 6.0, 4.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -6.0)))
        .id();
    app.update();
    let last = monster_state(&app, monster).pursuit.unwrap().last_sensed;
    assert_eq!(last, Vec2::new(0.0, -6.0));
    hide(&mut app, player);
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(5.0, 1.6, 3.0));
    let mut searched_at = None;
    let mut search_targets: Vec<Vec2> = Vec::new();
    let frames = (SEARCH_TIME / 0.1).round() as usize;
    for frame in 0..frames + 20 {
        app.update();
        let state = monster_state(&app, monster);
        let position = app
            .world()
            .get::<Transform>(monster)
            .unwrap()
            .translation
            .xz();
        if frame + 3 < frames {
            assert!(
                state.pursuit.is_some(),
                "search must persist after losing the player"
            );
        }
        if frame > frames + 3 {
            assert!(
                state.pursuit.is_none(),
                "search must end without new evidence"
            );
            continue;
        }
        let Some(pursuit) = state.pursuit else {
            continue;
        };
        assert_eq!(
            pursuit.last_sensed, last,
            "hidden player must not be tracked"
        );
        if pursuit.searching {
            searched_at.get_or_insert(frame);
            assert!(position.distance(last) <= SEARCH_RADIUS + GRID);
            if let Some(&target) = state.route.back() {
                assert!(target.distance(last) <= SEARCH_RADIUS + GRID);
                if search_targets
                    .iter()
                    .all(|seen| seen.distance(target) > GRID)
                {
                    search_targets.push(target);
                }
            }
        }
    }
    let searched_at = searched_at.expect("monster must search near its last sensed position");
    assert!(searched_at < 20);
    assert!(
        search_targets.len() >= 2,
        "search must scan several points around the last sensed position"
    );
    assert!(monster_state(&app, monster).pursuit.is_none());
    assert!(!app.world().entity(player).contains::<Caught>());
}

#[test]
fn player_sensed_again_near_the_lost_position_ends_the_search() {
    use crate::controller::{PlayerController, PlayerInput, Stamina};
    use crate::levels::{Hidden, Monster, Room};

    let mut app = monster_app(7);
    app.world_mut()
        .spawn(Room(Rect::new(-6.0, -16.0, 6.0, 4.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -6.0)))
        .id();
    app.update();
    hide(&mut app, player);
    let mut searching = false;
    for _ in 0..40 {
        app.update();
        let state = monster_state(&app, monster);
        let position = app
            .world()
            .get::<Transform>(monster)
            .unwrap()
            .translation
            .xz();
        if state.pursuit.unwrap().searching && position.distance(Vec2::new(0.0, -6.0)) > 1.0 {
            searching = true;
            break;
        }
    }
    assert!(searching);
    app.world_mut().entity_mut(player).remove::<Hidden>();
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(0.2, 1.6, -6.2));
    app.world_mut()
        .get_mut::<PlayerInput>(player)
        .unwrap()
        .movement = Vec2::Y;
    app.world_mut()
        .get_mut::<Stamina>(player)
        .unwrap()
        .sprinting = true;
    app.update();
    let state = monster_state(&app, monster);
    assert!(!state.pursuit.unwrap().searching);
    assert!(state
        .route
        .back()
        .is_none_or(|goal| goal.distance(Vec2::new(0.2, -6.2)) <= GRID));
    for _ in 0..10 {
        app.update();
    }
    assert!(
        app.world()
            .entity(player)
            .contains::<crate::levels::Caught>(),
        "monster must chase a player it senses again"
    );
}

#[test]
fn hidden_player_in_plain_view_and_reach_is_never_sensed_or_attacked() {
    use crate::controller::{PlayerController, PlayerInput, Stamina};
    use crate::levels::{Caught, Monster, Room};

    let mut app = monster_app(7);
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -0.5)))
        .id();
    hide(&mut app, player);
    app.world_mut()
        .get_mut::<PlayerInput>(player)
        .unwrap()
        .movement = Vec2::Y;
    app.world_mut()
        .get_mut::<Stamina>(player)
        .unwrap()
        .sprinting = true;
    for _ in 0..30 {
        app.update();
        let state = monster_state(&app, monster);
        assert!(state.pursuit.is_none());
        assert!(state.route.is_empty());
        assert!(!app.world().entity(player).contains::<Caught>());
    }
}

#[test]
fn hiding_before_contact_keeps_a_chased_player_safe() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Monster, Room};

    let mut app = monster_app(7);
    app.world_mut().spawn(Room(Rect::new(-5.0, -8.0, 5.0, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -3.0)))
        .id();
    app.update();
    assert!(monster_state(&app, monster).pursuit.is_some());
    hide(&mut app, player);
    let mut reached = false;
    for _ in 0..60 {
        app.update();
        let position = app
            .world()
            .get::<Transform>(monster)
            .unwrap()
            .translation
            .xz();
        reached |= position.distance(Vec2::new(0.0, -3.0)) <= super::ATTACK_REACH;
        assert!(!app.world().entity(player).contains::<Caught>());
    }
    assert!(
        reached,
        "monster must reach the hiding player without a catch"
    );
}

#[test]
fn witnessed_hiding_is_pulled_out_into_the_scripted_catch() {
    use crate::controller::PlayerController;
    use crate::levels::{
        Caught, Hidden, HidingPhase, HidingPlugin, HidingSpot, Monster, Room, UseHidingSpot,
    };

    let mut app = monster_app(7);
    app.add_plugins(HidingPlugin);
    app.world_mut().spawn(Room(Rect::new(-5.0, -8.0, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(0.0, 0.0, -4.0)))
        .id();
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -2.9)))
        .id();
    app.update();
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(app.world().get::<Hidden>(player).is_some());
    assert_eq!(
        app.world()
            .get::<super::WitnessedHiding>(player)
            .unwrap()
            .monster,
        monster
    );
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(!matches!(
        app.world().get::<Hidden>(player).unwrap().phase,
        HidingPhase::Leaving(_)
    ));
    let mut settled = false;
    let mut pulled_out = false;
    for _ in 0..100 {
        app.update();
        settled |= app
            .world()
            .get::<Hidden>(player)
            .is_some_and(Hidden::settled);
        pulled_out |= app
            .world()
            .get::<Hidden>(player)
            .is_some_and(|hidden| matches!(hidden.phase, HidingPhase::Leaving(_)));
        if app.world().get::<Caught>(player).is_some() {
            break;
        }
    }
    assert!(settled, "monster must reach a fully hidden player");
    assert!(
        pulled_out,
        "monster must pull the player out before attacking"
    );
    assert_eq!(app.world().get::<Caught>(player).unwrap().monster, monster);
    assert!(app.world().get::<Hidden>(player).is_none());
    assert!(app.world().get::<super::WitnessedHiding>(player).is_none());
    let exit = HidingSpot::Locker.exit(app.world().get::<Transform>(spot).unwrap());
    assert_eq!(
        app.world()
            .get::<Transform>(player)
            .unwrap()
            .translation
            .xz(),
        exit
    );
}

#[test]
fn unseen_hiding_remains_safe_even_when_monster_later_approaches() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Hidden, HidingPlugin, HidingSpot, Monster, Room, UseHidingSpot};

    let mut app = monster_app(7);
    app.add_plugins(HidingPlugin);
    app.world_mut().spawn(Room(Rect::new(-5.0, -8.0, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(0.0, 0.0, -4.0)))
        .id();
    let monster = app
        .world_mut()
        .spawn((
            Monster::default(),
            Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -2.9)))
        .id();
    app.update();
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(app.world().get::<super::WitnessedHiding>(player).is_none());
    app.world_mut()
        .entity_mut(monster)
        .insert(Transform::from_xyz(0.0, 0.0, -3.2));
    for _ in 0..40 {
        app.update();
    }
    assert!(app
        .world()
        .get::<Hidden>(player)
        .is_some_and(Hidden::settled));
    assert!(app.world().get::<Caught>(player).is_none());
}

#[test]
fn hiding_entry_behind_a_wall_is_not_witnessed() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Hidden, HidingPlugin, HidingSpot, Monster, Room, UseHidingSpot};

    let mut app = monster_app(7);
    app.add_plugins(HidingPlugin);
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -5.0, 5.0, -0.5)));
    app.world_mut().spawn(Room(Rect::new(-5.0, -0.5, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(0.0, 0.0, -2.4)))
        .id();
    app.world_mut()
        .spawn((Monster::default(), Transform::IDENTITY));
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -1.3)))
        .id();
    app.update();
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(app.world().get::<Hidden>(player).is_some());
    assert!(app.world().get::<super::WitnessedHiding>(player).is_none());
    for _ in 0..20 {
        app.update();
    }
    assert!(app.world().get::<Caught>(player).is_none());
}

#[test]
fn caught_player_cannot_move_look_or_hide_and_faces_the_full_attack() {
    use crate::controller::{
        player::apply_input, PlayerController, PlayerControlsEnabled, PlayerInput, SprintExhausted,
    };
    use crate::levels::{Caught, Hidden, HidingPlugin, HidingSpot, Monster, Room, UseHidingSpot};

    let mut app = monster_app(7);
    app.add_plugins(HidingPlugin)
        .insert_resource(PlayerControlsEnabled(true))
        .add_message::<SprintExhausted>()
        .add_message::<game_audio::PlaySound>()
        .add_systems(Update, apply_input);
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(1.0, 0.0, -1.7)))
        .id();
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -1.7)))
        .id();
    let anchor = Vec3::new(0.0, 1.6, -1.7);
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
        let caught = *app
            .world()
            .get::<Caught>(player)
            .expect("contact must catch the player on the same frame");
        assert_eq!(caught.monster, monster);
        let view = *app.world().get::<Transform>(player).unwrap();
        assert_eq!(view.translation, anchor, "caught player must not move");
        assert!(!app.world().entity(player).contains::<Hidden>());
        assert!(
            previous.angle_between(view.rotation) <= super::JUMPSCARE_TURN_SPEED * 0.1 + 0.001,
            "caught view must turn at a bounded rate and ignore look input"
        );
        previous = view.rotation;
        if caught.finished() && finished_at.is_none() {
            finished_at = Some(frame);
        }
    }
    let finished_at = finished_at.expect("scripted attack must finish");
    assert!((super::ATTACK_DURATION - 40.0 / 24.0).abs() < 0.001);
    let frames = super::ATTACK_DURATION / 0.1;
    assert!(finished_at as f32 >= frames - 1.0);
    assert!(finished_at as f32 <= frames + 1.0);
    let view = app.world().get::<Transform>(player).unwrap();
    let pose = app.world().get::<Transform>(monster).unwrap();
    let head = pose.transform_point(super::head_rest_offset());
    assert!(view.forward().dot((head - view.translation).normalize()) > 0.99);
    assert!(
        view.forward().y > 0.3,
        "caught view must look up at the head"
    );
    assert!(head.y > view.translation.y + 0.2);
    let toward_player = (view.translation - pose.translation).xz().normalize();
    assert!(pose.forward().xz().dot(toward_player) > 0.95);
    assert!((head - pose.translation).xz().dot(toward_player) > 1.0);
}

#[test]
fn jumpscare_aims_at_the_live_head_joint_when_the_scene_is_loaded() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Monster, Room};

    let mut app = monster_app(7);
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let joint = Vec3::new(0.3, 2.3, -1.1);
    app.world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .with_children(|children| {
            children.spawn(Transform::default()).with_children(|bones| {
                bones.spawn((
                    Name::new(super::HEAD_JOINT),
                    GlobalTransform::from_translation(joint),
                ));
            });
        });
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -1.7)))
        .id();
    for _ in 0..12 {
        app.update();
    }
    assert!(app.world().entity(player).contains::<Caught>());
    let view = app.world().get::<Transform>(player).unwrap();
    assert!(view.forward().dot((joint - view.translation).normalize()) > 0.99);
}

#[test]
fn contact_reach_covers_the_head_offset_without_reaching_further() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Monster, Room};

    let head = super::head_rest_offset();
    assert!(head.y > 1.8);
    assert!(head.z < -1.2);
    assert!(super::ATTACK_REACH - head.xz().length() >= 0.3);
    for (distance, caught) in [(1.7, true), (1.95, false)] {
        let mut app = monster_app(7);
        app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
        let monster = app
            .world_mut()
            .spawn((Monster::default(), Transform::IDENTITY))
            .id();
        let player = app
            .world_mut()
            .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, distance)))
            .id();
        for _ in 0..10 {
            app.update();
        }
        assert_eq!(
            app.world().entity(player).contains::<Caught>(),
            caught,
            "still player behind the monster at {distance} m"
        );
        assert_eq!(monster_state(&app, monster).pursuit.is_some(), caught);
    }
}

#[test]
fn chase_stops_at_contact_range_so_the_head_stays_in_front_of_the_view() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Monster, Room};

    let mut app = monster_app(7);
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -12.0, 5.0, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -7.0)))
        .id();
    let mut caught = false;
    for _ in 0..30 {
        app.update();
        if app.world().entity(player).contains::<Caught>() {
            caught = true;
            break;
        }
    }
    assert!(caught);
    let pose = app.world().get::<Transform>(monster).unwrap();
    let view = app.world().get::<Transform>(player).unwrap();
    let distance = pose.translation.xz().distance(view.translation.xz());
    assert!(distance <= super::ATTACK_REACH);
    assert!(distance >= super::ATTACK_REACH * 0.85);
    let head = pose.transform_point(super::head_rest_offset());
    let toward_player = (view.translation - pose.translation).xz().normalize();
    assert!(
        (view.translation - head).xz().dot(toward_player) > 0.25,
        "head must stay in front of the camera"
    );
}

#[test]
fn walls_block_sprint_sound_and_a_player_behind_them_is_not_tracked() {
    use crate::controller::{PlayerController, PlayerInput, Stamina};
    use crate::levels::{Monster, Room};

    let mut app = monster_app(7);
    app.world_mut().spawn(Room(Rect::new(-2.5, -5.0, 2.5, 0.0)));
    app.world_mut().spawn(Room(Rect::new(-2.5, 0.0, 2.5, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.0, 0.0, 1.0)))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -1.5)))
        .id();
    app.world_mut()
        .get_mut::<PlayerInput>(player)
        .unwrap()
        .movement = Vec2::Y;
    app.world_mut()
        .get_mut::<Stamina>(player)
        .unwrap()
        .sprinting = true;
    for _ in 0..10 {
        app.update();
        assert!(monster_state(&app, monster).pursuit.is_none());
    }
    app.world_mut()
        .entity_mut(monster)
        .insert(Transform::from_xyz(0.0, 0.0, 4.5));
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(0.0, 1.6, 2.0));
    app.update();
    let last = monster_state(&app, monster).pursuit.unwrap().last_sensed;
    assert_eq!(last, Vec2::new(0.0, 2.0));
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(1.5, 1.6, -3.0));
    for _ in 0..20 {
        app.update();
        let state = monster_state(&app, monster);
        assert_eq!(state.pursuit.unwrap().last_sensed, last);
        assert!(app.world().get::<Transform>(monster).unwrap().translation.z > 0.0);
    }
}

#[test]
fn route_starts_from_a_reachable_cell_when_the_nearest_node_is_blocked() {
    use crate::controller::collision::clear_for_player;

    let rooms = [(
        Entity::from_raw_u32(1).unwrap(),
        Rect::new(-5.0, -8.0, 5.0, 3.0),
    )];
    let blockers = [(Vec2::new(0.0, -0.9), Vec2::new(2.0, 0.2), 0.0)];
    let start = Vec2::new(0.2, -0.44);
    assert!(clear_for_player(start, &blockers));
    assert!(!clear_for_player(point(super::node(start)), &blockers));
    let path = route(start, Vec2::new(0.0, -4.0), &rooms, &blockers)
        .expect("route from an off-grid start beside a prop");
    let mut from = start;
    for &waypoint in &path {
        assert!(super::clear_path(from, waypoint, &blockers));
        from = waypoint;
    }
    assert!(from.distance(Vec2::new(0.0, -4.0)) < 0.01);
}

#[test]
fn chase_routes_around_a_prop_from_an_off_grid_start_without_clipping() {
    use crate::controller::{collision::clear_for_player, PlayerController};
    use crate::levels::{Monster, PropCollider, Room};

    let mut app = monster_app(7);
    let room = Rect::new(-5.0, -8.0, 5.0, 3.0);
    app.world_mut().spawn(Room(room));
    app.world_mut().spawn((
        PropCollider {
            center: Vec2::ZERO,
            half: Vec2::new(2.0, 0.2),
        },
        Transform::from_xyz(0.0, 0.0, -0.9),
    ));
    let blockers = [(Vec2::new(0.0, -0.9), Vec2::new(2.0, 0.2), 0.0)];
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.2, 0.0, -0.44)))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -4.0)))
        .id();
    let mut previous = Vec2::new(0.2, -0.44);
    let mut reached = false;
    for _ in 0..40 {
        app.update();
        let position = app
            .world()
            .get::<Transform>(monster)
            .unwrap()
            .translation
            .xz();
        assert!(clear_for_player(position, &blockers));
        assert!(super::clear_path(previous, position, &blockers));
        assert!(room.contains(position));
        previous = position;
        if app
            .world()
            .entity(player)
            .contains::<crate::levels::Caught>()
        {
            reached = true;
            break;
        }
    }
    assert!(reached, "chase must reach the player around the prop");
}

#[test]
fn blocked_route_replans_around_a_new_obstacle_without_clipping() {
    use crate::controller::collision::clear_for_player;
    use crate::levels::{Monster, PropCollider, Room};

    let mut app = monster_app(7);
    let room = Rect::new(-5.0, -8.0, 5.0, 3.0);
    app.world_mut().spawn(Room(room));
    app.world_mut().spawn((
        PropCollider {
            center: Vec2::ZERO,
            half: Vec2::new(2.0, 0.2),
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
    let blockers = [(Vec2::new(0.0, -1.0), Vec2::new(2.0, 0.2), 0.0)];
    let goal = Vec2::new(0.0, -3.0);
    let monster = app
        .world_mut()
        .spawn((
            Monster {
                route: (1..=6)
                    .map(|step| Vec2::new(0.0, -0.5 * step as f32))
                    .collect(),
                ..Monster::default()
            },
            Transform::IDENTITY,
        ))
        .id();
    let mut previous = Vec2::ZERO;
    let mut reached = false;
    for _ in 0..600 {
        app.update();
        let position = app
            .world()
            .get::<Transform>(monster)
            .unwrap()
            .translation
            .xz();
        assert!(clear_for_player(position, &blockers));
        assert!(super::clear_path(previous, position, &blockers));
        assert!(room.contains(position));
        previous = position;
        if position.distance(goal) < 0.2 {
            reached = true;
            break;
        }
    }
    assert!(reached, "stalled route must replan around the obstacle");
}

#[test]
fn sprint_is_heard_from_behind_but_walking_at_that_distance_is_not() {
    use crate::controller::{PlayerController, PlayerInput, Stamina};
    use crate::levels::{Monster, Room};
    use rand_core::SeedableRng;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .insert_resource(super::PatrolRng(
            bevy_rand::prelude::ChaCha8Rng::seed_from_u64(7),
        ))
        .add_plugins(super::MonsterPlugin);
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -5.0, 5.0, 15.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 7.0)))
        .id();
    app.world_mut()
        .get_mut::<PlayerInput>(player)
        .unwrap()
        .movement = Vec2::Y;
    app.update();
    assert!(app
        .world()
        .get::<Monster>(monster)
        .unwrap()
        .pursuit
        .is_none());
    app.world_mut()
        .get_mut::<Stamina>(player)
        .unwrap()
        .sprinting = true;
    app.update();
    assert!(app
        .world()
        .get::<Monster>(monster)
        .unwrap()
        .pursuit
        .is_some());
}

#[test]
fn walls_block_sight_and_contact_is_a_guaranteed_catch() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Monster, Room};

    let mut app = monster_app(7);
    app.world_mut().spawn(Room(Rect::new(-2.5, -2.5, 2.5, 0.0)));
    app.world_mut().spawn(Room(Rect::new(-2.5, 0.0, 2.5, 2.5)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.0, 0.0, 0.8)))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -0.8)))
        .id();
    app.update();
    assert!(monster_state(&app, monster).pursuit.is_none());
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(0.0, 1.6, 0.25));
    app.update();
    assert_eq!(app.world().get::<Caught>(player).unwrap().monster, monster);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .x = 2.0;
    for _ in 0..20 {
        app.update();
        assert!(app.world().entity(player).contains::<Caught>());
    }
    assert!(app.world().get::<Caught>(player).unwrap().finished());
}

#[test]
fn pursuing_monster_cannot_catch_through_a_wall_within_attack_reach() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Monster, Room};

    let mut app = monster_app(7);
    app.world_mut().spawn(Room(Rect::new(-2.5, 0.0, 2.5, 5.0)));
    app.world_mut().spawn(Room(Rect::new(-2.5, -5.0, 2.5, 0.0)));
    let monster = app
        .world_mut()
        .spawn((
            Monster::default(),
            Transform::from_xyz(0.0, 0.0, 0.25)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 3.0)))
        .id();
    app.update();
    assert!(monster_state(&app, monster).pursuit.is_some());
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .z = -0.8;
    assert!(
        app.world()
            .get::<Transform>(monster)
            .unwrap()
            .translation
            .z
            .abs()
            + 0.8
            < super::ATTACK_REACH
    );
    for _ in 0..3 {
        app.update();
        assert!(app.world().get::<Caught>(player).is_none());
    }
}

#[test]
fn detection_and_attack_emit_one_source_attached_cue_each() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Monster, Room};
    use game_audio::{PlaySourceSound, Sound};
    use rand_core::SeedableRng;

    #[derive(Resource, Default)]
    struct Cues(Vec<(Entity, Sound)>);
    fn collect(mut sounds: MessageReader<PlaySourceSound>, mut cues: ResMut<Cues>) {
        cues.0
            .extend(sounds.read().map(|cue| (cue.source, cue.sound)));
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .insert_resource(super::PatrolRng(
            bevy_rand::prelude::ChaCha8Rng::seed_from_u64(7),
        ))
        .init_resource::<Cues>()
        .add_plugins(super::MonsterPlugin)
        .add_systems(PostUpdate, collect);
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -0.7)))
        .id();
    for _ in 0..12 {
        app.update();
    }
    let cues = &app.world().resource::<Cues>().0;
    assert_eq!(
        cues.iter()
            .filter(|&&(source, sound)| source == monster && sound == Sound::MonsterDetected)
            .count(),
        1
    );
    assert_eq!(
        cues.iter()
            .filter(|&&(source, sound)| source == monster && sound == Sound::MonsterAttack)
            .count(),
        1
    );
    assert!(app.world().entity(player).contains::<Caught>());
}

#[test]
fn patrol_path_uses_opening_and_avoids_solid_props() {
    let rooms = [
        (
            Entity::from_raw_u32(1).unwrap(),
            Rect::new(-4.0, -4.0, 0.0, 4.0),
        ),
        (
            Entity::from_raw_u32(2).unwrap(),
            Rect::new(0.0, -4.0, 4.0, 4.0),
        ),
    ];
    let blockers = [
        (
            Vec2::new(0.0, -2.5),
            Vec2::new(1.5, 0.125),
            std::f32::consts::FRAC_PI_2,
        ),
        (
            Vec2::new(0.0, 2.5),
            Vec2::new(1.5, 0.125),
            std::f32::consts::FRAC_PI_2,
        ),
        (Vec2::new(-1.5, 0.0), Vec2::new(0.35, 0.35), 0.0),
    ];
    let path = route(Vec2::new(-3.0, 0.0), Vec2::new(3.0, 0.0), &rooms, &blockers)
        .expect("path through the clear opening");
    assert!(path.iter().any(|position| position.x >= 0.5));
    assert!(path
        .iter()
        .all(|&position| crate::controller::collision::clear_for_player(position, &blockers)));
    assert!(path
        .iter()
        .all(|&position| room_at(position, &rooms).is_some()));
    assert!(path.iter().any(|position| position.y.abs() >= GRID));
}

#[test]
fn patrol_path_rejects_a_sealed_wall() {
    let rooms = [
        (
            Entity::from_raw_u32(1).unwrap(),
            Rect::new(-4.0, -4.0, 0.0, 4.0),
        ),
        (
            Entity::from_raw_u32(2).unwrap(),
            Rect::new(0.0, -4.0, 4.0, 4.0),
        ),
    ];
    let blockers = [(
        Vec2::ZERO,
        Vec2::new(4.0, 0.125),
        std::f32::consts::FRAC_PI_2,
    )];
    assert!(route(Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0), &rooms, &blockers).is_none());
}

#[test]
fn route_through_an_interior_door_is_independent_of_its_open_state() {
    use crate::levels::{Door, DoorOf, DoorRef, DoorState, DoorSwing, Doors, Passage, Room};

    fn planned_route(
        rooms: Query<(Entity, &Room)>,
        walls: Query<(&Room, Option<&Doors>)>,
        links: Query<(&DoorRef, &DoorOf)>,
        doors: Query<(&Door, &DoorSwing)>,
        passages: Query<&Passage>,
    ) -> Option<std::collections::VecDeque<Vec2>> {
        let rooms: Vec<_> = rooms
            .iter()
            .map(|(entity, room)| (entity, room.0))
            .collect();
        let mut blockers =
            crate::controller::collision::wall_obstacles(&walls, &links, &doors, &passages);
        for (door, _) in &doors {
            blockers.extend(crate::controller::collision::door_frames(door));
        }
        route(Vec2::new(-2.5, 0.0), Vec2::new(2.5, 0.0), &rooms, &blockers)
    }

    let mut app = App::new();
    let door = app
        .world_mut()
        .spawn(crate::levels::builder::door(
            "route door",
            Vec2::ZERO,
            std::f32::consts::FRAC_PI_2,
            "wall_doorway",
            "door_panel",
        ))
        .id();
    app.world_mut()
        .spawn(Room(Rect::new(-5.0, -1.25, 0.0, 1.25)))
        .with_related::<DoorOf>(DoorRef(door));
    app.world_mut()
        .spawn(Room(Rect::new(0.0, -1.25, 5.0, 1.25)))
        .with_related::<DoorOf>(DoorRef(door));
    let closed = app.world_mut().run_system_cached(planned_route).unwrap();
    assert!(closed.is_some());
    app.world_mut().get_mut::<Door>(door).unwrap().state = DoorState::Open;
    app.world_mut().get_mut::<DoorSwing>(door).unwrap().0 = std::f32::consts::FRAC_PI_2;
    let open = app.world_mut().run_system_cached(planned_route).unwrap();
    assert_eq!(closed, open);
}

#[derive(Resource, Default)]
struct DoorToggles(Vec<Entity>);

fn record_door_toggles(
    mut toggles: MessageReader<crate::levels::ToggleDoor>,
    mut doors: Query<&mut crate::levels::Door>,
    mut recorded: ResMut<DoorToggles>,
) {
    use crate::levels::DoorState;

    for toggle in toggles.read() {
        recorded.0.push(toggle.0);
        if let Ok(mut door) = doors.get_mut(toggle.0) {
            door.state = match door.state {
                DoorState::Closed => DoorState::Open,
                DoorState::Open => DoorState::Closed,
            };
        }
    }
}

fn door_app(door_at: Vec2, yaw: f32, rooms: [Rect; 2]) -> (App, Entity) {
    use crate::levels::{DoorOf, DoorRef, Room};

    let mut app = monster_app(7);
    app.init_resource::<DoorToggles>().add_systems(
        Update,
        (record_door_toggles, crate::levels::animation::animate_doors).chain(),
    );
    let door = app
        .world_mut()
        .spawn(crate::levels::builder::door(
            "route door",
            door_at,
            yaw,
            "wall_doorway",
            "door_panel",
        ))
        .id();
    for bounds in rooms {
        app.world_mut()
            .spawn(Room(bounds))
            .with_related::<DoorOf>(DoorRef(door));
    }
    (app, door)
}

fn monster_on_route(app: &mut App, start: Vec2, end: Vec2) -> Entity {
    use crate::levels::Monster;

    let steps = (start.distance(end) / GRID).round() as usize;
    let route = (1..=steps)
        .map(|step| start.lerp(end, step as f32 / steps as f32))
        .collect();
    let facing = Quat::from_rotation_arc(Vec3::NEG_Z, (end - start).normalize().extend(0.0).xzy());
    app.world_mut()
        .spawn((
            Monster {
                route,
                ..Monster::default()
            },
            Transform::from_xyz(start.x, 0.0, start.y).with_rotation(facing),
        ))
        .id()
}

#[test]
fn monster_walking_past_a_side_door_leaves_it_closed_and_keeps_moving() {
    use crate::levels::{Door, DoorState};

    let (mut app, door) = door_app(
        Vec2::new(1.25, 1.25),
        0.0,
        [
            Rect::new(-5.0, -1.25, 5.0, 1.25),
            Rect::new(0.0, 1.25, 2.5, 3.75),
        ],
    );
    let monster = monster_on_route(&mut app, Vec2::new(-1.0, 0.0), Vec2::new(4.0, 0.0));
    let mut closest = f32::MAX;
    let mut furthest = f32::MIN;
    for _ in 0..100 {
        app.update();
        let position = app
            .world()
            .get::<Transform>(monster)
            .unwrap()
            .translation
            .xz();
        closest = closest.min(position.distance(Vec2::new(1.25, 1.25)));
        furthest = furthest.max(position.x);
    }
    assert!(
        closest < super::DOOR_APPROACH,
        "route must pass the side door"
    );
    assert!(app.world().resource::<DoorToggles>().0.is_empty());
    assert_eq!(
        app.world().get::<Door>(door).unwrap().state,
        DoorState::Closed
    );
    assert!(furthest > 3.5, "monster must not wait at a side door");
}

#[test]
fn monster_opens_the_doorway_it_faces_on_its_route_and_crosses_without_snagging() {
    use crate::controller::collision::{clear_for_player, door_frames, door_panel};
    use crate::levels::{Door, DoorState, DoorSwing};

    let (mut app, door) = door_app(
        Vec2::ZERO,
        std::f32::consts::FRAC_PI_2,
        [
            Rect::new(-5.0, -1.25, 0.0, 1.25),
            Rect::new(0.0, -1.25, 5.0, 1.25),
        ],
    );
    let monster = monster_on_route(&mut app, Vec2::new(-2.5, 0.0), Vec2::new(2.5, 0.0));
    let mut crossed = false;
    for _ in 0..200 {
        app.update();
        let world = app.world();
        let position = world.get::<Transform>(monster).unwrap().translation.xz();
        let state = world.get::<Door>(door).unwrap();
        let swing = world.get::<DoorSwing>(door).unwrap();
        let mut blockers = door_frames(state).to_vec();
        blockers.push(door_panel(state, swing));
        assert!(
            clear_for_player(position, &blockers),
            "monster clipped the doorway at {position}"
        );
        if position.x > 2.3 {
            crossed = true;
            break;
        }
    }
    assert!(crossed, "monster must cross the opened doorway");
    assert_eq!(app.world().resource::<DoorToggles>().0, vec![door]);
    for _ in 0..30 {
        app.update();
    }
    assert_eq!(app.world().resource::<DoorToggles>().0, vec![door]);
    assert_eq!(
        app.world().get::<Door>(door).unwrap().state,
        DoorState::Open
    );
}

#[test]
fn monster_turns_to_a_route_door_entered_from_a_corner_and_crosses() {
    use crate::levels::{Door, DoorState, Monster};

    let (mut app, door) = door_app(
        Vec2::ZERO,
        std::f32::consts::FRAC_PI_2,
        [
            Rect::new(-5.0, -1.25, 0.0, 1.25),
            Rect::new(0.0, -1.25, 5.0, 1.25),
        ],
    );
    let start = Vec2::new(-0.9, -0.9);
    assert!(start.length() < super::DOOR_APPROACH);
    let monster = app
        .world_mut()
        .spawn((
            Monster {
                route: [
                    Vec2::new(-1.5, -0.5),
                    Vec2::ZERO,
                    Vec2::new(1.0, 0.0),
                    Vec2::new(2.5, 0.0),
                ]
                .into(),
                ..Monster::default()
            },
            Transform::from_xyz(start.x, 0.0, start.y),
        ))
        .id();
    let mut crossed = false;
    for _ in 0..300 {
        app.update();
        if app.world().get::<Transform>(monster).unwrap().translation.x > 2.0 {
            crossed = true;
            break;
        }
    }
    assert_eq!(app.world().resource::<DoorToggles>().0, vec![door]);
    assert_eq!(
        app.world().get::<Door>(door).unwrap().state,
        DoorState::Open
    );
    assert!(
        crossed,
        "monster must not deadlock at a door entered from a corner"
    );
}

#[test]
fn distant_player_ahead_is_seen_but_cone_and_walls_still_limit_sight() {
    use crate::controller::PlayerController;
    use crate::levels::{Monster, Room};

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
        let mut app = monster_app(7);
        for bounds in rooms {
            app.world_mut().spawn(Room(bounds));
        }
        let monster = app
            .world_mut()
            .spawn((Monster::default(), Transform::IDENTITY))
            .id();
        app.world_mut()
            .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, player_at)));
        app.update();
        let pursuit = monster_state(&app, monster).pursuit;
        assert_eq!(pursuit.is_some(), seen, "player at z {player_at}");
        if seen {
            assert_eq!(pursuit.unwrap().last_sensed, Vec2::new(0.0, player_at));
        }
    }
}

#[test]
fn patrol_opens_traverses_and_leaves_an_unlocked_door_open() {
    use crate::levels::{Door, DoorOf, DoorRef, DoorState, Monster, Room};

    fn toggle(mut toggles: MessageReader<crate::levels::ToggleDoor>, mut doors: Query<&mut Door>) {
        for toggle in toggles.read() {
            if let Ok(mut door) = doors.get_mut(toggle.0) {
                door.state = match door.state {
                    DoorState::Closed => DoorState::Open,
                    DoorState::Open => DoorState::Closed,
                };
            }
        }
    }

    fn setup(mut commands: Commands) {
        let door = commands
            .spawn(crate::levels::builder::door(
                "patrol door",
                Vec2::ZERO,
                std::f32::consts::FRAC_PI_2,
                "wall_doorway",
                "door_panel",
            ))
            .id();
        commands
            .spawn(Room(Rect::new(-5.0, -1.25, 0.0, 1.25)))
            .with_related::<DoorOf>(DoorRef(door));
        commands
            .spawn(Room(Rect::new(0.0, -1.25, 5.0, 1.25)))
            .with_related::<DoorOf>(DoorRef(door));
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )))
        .add_plugins(super::MonsterPlugin)
        .add_systems(Startup, (setup, crate::levels::spawn_first_floor_actors))
        .add_systems(
            Update,
            (toggle, crate::levels::animation::animate_doors).chain(),
        );
    app.update();
    let world = app.world_mut();
    let mut monsters = world.query_filtered::<Entity, With<Monster>>();
    let monster = monsters.single(world).unwrap();
    app.world_mut()
        .entity_mut(monster)
        .insert(Transform::from_xyz(-2.5, 0.0, 0.0));
    let mut opened = false;
    let mut crossed = false;
    for _ in 0..1100 {
        app.update();
        let world = app.world_mut();
        let door = world.query::<&Door>().single(world).unwrap();
        let position = world.get::<Transform>(monster).unwrap().translation.x;
        opened |= door.state == DoorState::Open;
        crossed |= opened && position > 1.8;
        if crossed {
            break;
        }
    }
    assert!(opened, "patrol must open the door");
    assert!(crossed, "patrol must cross the door");
    for _ in 0..100 {
        app.update();
    }
    let world = app.world_mut();
    assert_eq!(
        world.query::<&Door>().single(world).unwrap().state,
        DoorState::Open
    );
}

#[test]
fn locked_exit_is_not_a_patrol_neighbor() {
    use crate::levels::{Door, DoorLock, DoorOf, DoorRef, Monster, Room};

    fn setup(mut commands: Commands) {
        let door = commands
            .spawn((
                crate::levels::builder::door(
                    "locked exit",
                    Vec2::ZERO,
                    std::f32::consts::FRAC_PI_2,
                    "wall_doorway",
                    "door_panel",
                ),
                DoorLock,
            ))
            .id();
        commands
            .spawn(Room(Rect::new(-5.0, -1.25, 0.0, 1.25)))
            .with_related::<DoorOf>(DoorRef(door));
        commands
            .spawn(Room(Rect::new(0.0, -1.25, 5.0, 1.25)))
            .with_related::<DoorOf>(DoorRef(door));
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )))
        .add_plugins(super::MonsterPlugin)
        .add_systems(Startup, (setup, crate::levels::spawn_first_floor_actors));
    for _ in 0..200 {
        app.update();
    }
    let world = app.world_mut();
    let mut monsters = world.query_filtered::<&Transform, With<Monster>>();
    assert!(monsters.single(world).unwrap().translation.x < 0.0);
    assert_eq!(
        world.query::<&Door>().single(world).unwrap().state,
        crate::levels::DoorState::Closed
    );
}

#[test]
fn authored_first_floor_patrol_reaches_another_room() {
    use crate::levels::{Door, DoorState, Monster, Room};
    use game_audio::{PlaySourceSound, Sound};

    #[derive(Resource, Default)]
    struct Heard(Vec<Sound>);

    fn collect(mut sounds: MessageReader<PlaySourceSound>, mut heard: ResMut<Heard>) {
        heard.0.extend(sounds.read().map(|cue| cue.sound));
    }

    fn toggle(mut events: MessageReader<crate::levels::ToggleDoor>, mut doors: Query<&mut Door>) {
        for event in events.read() {
            if let Ok(mut door) = doors.get_mut(event.0) {
                door.state = match door.state {
                    DoorState::Closed => DoorState::Open,
                    DoorState::Open => DoorState::Closed,
                };
            }
        }
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )))
        .insert_resource(crate::levels::FuseSeed(42))
        .init_resource::<Heard>()
        .add_plugins(super::MonsterPlugin)
        .add_observer(crate::controller::collision::attach_prop_collider)
        .add_systems(
            Startup,
            (
                crate::levels::build_first_floor,
                crate::levels::spawn_first_floor_actors,
            ),
        )
        .add_systems(
            Update,
            (toggle, crate::levels::animation::animate_doors).chain(),
        )
        .add_systems(PostUpdate, collect);
    app.update();
    let world = app.world_mut();
    let mut monsters = world.query_filtered::<Entity, With<Monster>>();
    let monster = monsters.single(world).unwrap();
    let mut crossed = false;
    let mut opened = false;
    let mut previous_rotation = app.world().get::<Transform>(monster).unwrap().rotation;
    for _ in 0..1800 {
        app.update();
        let world = app.world_mut();
        let transform = world.get::<Transform>(monster).unwrap();
        assert!(
            previous_rotation.angle_between(transform.rotation) <= TURN_SPEED * 0.016 + 0.001,
            "patrol must not snap its facing at a waypoint or room boundary"
        );
        previous_rotation = transform.rotation;
        let position = transform.translation.xz();
        let lab = world
            .query_filtered::<&Room, With<Room>>()
            .iter(world)
            .find(|room| room.0 == Rect::new(-3.75, -3.75, 3.75, 3.75))
            .unwrap()
            .0;
        crossed |= !lab.contains(position);
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
            .resource::<Heard>()
            .0
            .iter()
            .any(|cue| matches!(cue, Sound::MonsterStep(_))),
        "moving patrol must emit timed footsteps"
    );
}

#[test]
fn rendered_monster_attaches_one_scene_and_animation_graph() {
    use bevy::world_serialization::WorldAssetRoot;
    use game_assets::MonsterAssets;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<AnimationGraph>>()
        .insert_resource(MonsterAssets {
            scene: default(),
            idle: default(),
            walk: default(),
            chase: default(),
            attack: default(),
        })
        .add_plugins((super::MonsterPlugin, super::MonsterRenderPlugin))
        .add_systems(Startup, crate::levels::spawn_first_floor_actors);
    app.update();
    app.update();
    let world = app.world_mut();
    let mut scenes = world.query::<(&WorldAssetRoot, &Transform)>();
    let (_, visual) = scenes.single(world).unwrap();
    assert_eq!(visual.translation.x, -MODEL_PIVOT.x * MODEL_SCALE);
    assert_eq!(visual.translation.y, MODEL_LIFT);
    assert_eq!(visual.translation.z, -MODEL_PIVOT.y * MODEL_SCALE);
    let mut monsters = world.query_filtered::<&Children, With<super::Monster>>();
    assert_eq!(monsters.single(world).unwrap().len(), 1);
    assert!(world.contains_resource::<super::MonsterAnimations>());
}

#[test]
fn first_floor_authors_player_and_monster_only_when_actors_are_spawned() {
    use crate::controller::PlayerController;
    use crate::levels::{
        build_first_floor, spawn_first_floor_actors, Door, ExitDoor, FusePanel, FuseSeed, Monster,
        Room,
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(FuseSeed(42))
        .add_systems(Startup, build_first_floor);
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
    assert_eq!(player.translation, Vec3::new(2.5, 1.6, -27.5));
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
    let mut monsters = world.query_filtered::<&Transform, With<Monster>>();
    assert_eq!(
        monsters.single(world).unwrap().translation,
        Vec3::new(-2.5, 0.0, 0.0)
    );
    assert!(world.contains_resource::<super::PatrolRng>());
}

#[test]
fn figure_holds_the_final_attack_pose_once_without_restarting() {
    use crate::levels::MonsterFigure;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<AnimationGraph>>()
        .add_plugins(super::MonsterRenderPlugin);
    let clip: Handle<AnimationClip> = default();
    let (graph, nodes) =
        AnimationGraph::from_clips([clip.clone(), clip.clone(), clip.clone(), clip]);
    let graph_handle = app
        .world_mut()
        .resource_mut::<Assets<AnimationGraph>>()
        .add(graph);
    app.world_mut().insert_resource(super::MonsterAnimations {
        graph: graph_handle,
        idle: nodes[0],
        walk: nodes[1],
        chase: nodes[2],
        attack: nodes[3],
    });
    let figure = app.world_mut().spawn(MonsterFigure).id();
    let scene_root = app
        .world_mut()
        .spawn((super::MonsterScene, ChildOf(figure)))
        .id();
    let player_entity = app
        .world_mut()
        .spawn((AnimationPlayer::default(), ChildOf(scene_root)))
        .id();
    app.update();
    let attack = nodes[3];
    let paused_at = app
        .world()
        .get::<AnimationPlayer>(player_entity)
        .unwrap()
        .animation(attack)
        .unwrap()
        .seek_time();
    assert!(app
        .world()
        .get::<AnimationPlayer>(player_entity)
        .unwrap()
        .animation(attack)
        .unwrap()
        .is_paused());
    assert_eq!(paused_at, super::ATTACK_DURATION);
    app.update();
    assert_eq!(
        app.world()
            .get::<AnimationPlayer>(player_entity)
            .unwrap()
            .animation(attack)
            .unwrap()
            .seek_time(),
        paused_at,
        "an already-posed figure must not be restarted every frame"
    );
}

#[test]
fn monster_figure_never_gains_monster_ai_behavior() {
    use crate::levels::{MonsterFigure, Room};

    let mut app = monster_app(7);
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let figure = app
        .world_mut()
        .spawn((MonsterFigure, Transform::from_xyz(1.0, 0.0, 1.0)))
        .id();
    for _ in 0..10 {
        app.update();
    }
    assert!(app.world().get::<super::Monster>(figure).is_none());
    assert_eq!(
        app.world().get::<Transform>(figure).unwrap().translation,
        Vec3::new(1.0, 0.0, 1.0)
    );
}

#[test]
fn route_grid_is_half_meter() {
    assert_eq!(
        point(super::node(Vec2::new(-1.1, 2.1))),
        Vec2::new(-1.0, 2.0)
    );
}

fn flash_app() -> App {
    let mut app = monster_app(7);
    app.add_plugins(crate::levels::DevicePlugin);
    app
}

#[test]
fn flashed_player_in_view_and_reach_is_not_sensed_or_caught_until_the_flash_ends() {
    use crate::controller::{PlayerController, PlayerInput, Stamina};
    use crate::levels::{Caught, Flashed, Monster, Room};

    let mut app = flash_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, 1.6, -1.0),
            Flashed::default(),
        ))
        .id();
    app.world_mut()
        .entity_mut(player)
        .insert(super::super::devices::FlashTargets(vec![monster]));
    app.world_mut()
        .get_mut::<PlayerInput>(player)
        .unwrap()
        .movement = Vec2::Y;
    app.world_mut()
        .get_mut::<Stamina>(player)
        .unwrap()
        .sprinting = true;
    for _ in 0..48 {
        app.update();
        assert!(app.world().get::<Flashed>(player).is_some());
        assert!(monster_state(&app, monster).pursuit.is_none());
        assert!(app.world().get::<Caught>(player).is_none());
    }
    for _ in 0..10 {
        app.update();
    }
    assert!(app.world().get::<Flashed>(player).is_none());
    assert_eq!(app.world().get::<Caught>(player).unwrap().monster, monster);
}

#[test]
fn a_flash_only_hides_the_player_from_monsters_hit_by_its_burst() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Flashed, Monster, Room};

    let mut app = flash_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let hit = app
        .world_mut()
        .spawn((Monster::default(), Transform::from_xyz(-1.0, 0.0, 0.0)))
        .id();
    let missed = app
        .world_mut()
        .spawn((Monster::default(), Transform::from_xyz(1.0, 0.0, 0.0)))
        .id();
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(1.0, 1.6, -1.0),
            Flashed::default(),
            super::super::devices::FlashTargets(vec![hit]),
        ))
        .id();

    app.update();
    assert!(monster_state(&app, hit).pursuit.is_none());
    assert!(monster_state(&app, missed).pursuit.is_some());
    assert_eq!(app.world().get::<Caught>(player).unwrap().monster, missed);
}

#[test]
fn a_flash_with_no_monster_hit_does_not_hide_the_player() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Flashed, Monster, Room};

    let mut app = flash_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, 1.6, -1.0),
            Flashed::default(),
        ))
        .id();

    app.update();
    assert!(monster_state(&app, monster).pursuit.is_some());
    assert_eq!(app.world().get::<Caught>(player).unwrap().monster, monster);
}

#[test]
fn flash_mid_chase_keeps_the_last_known_search_without_tracking_or_contact() {
    use crate::controller::{PlayerController, PlayerInput, Stamina};
    use crate::levels::{Caught, Flashed, Monster, Room};

    let mut app = flash_app();
    app.world_mut()
        .spawn(Room(Rect::new(-6.0, -16.0, 6.0, 4.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -6.0)))
        .id();
    app.update();
    let last = monster_state(&app, monster).pursuit.unwrap().last_sensed;
    assert_eq!(last, Vec2::new(0.0, -6.0));
    app.world_mut().entity_mut(player).insert((
        Transform::from_xyz(0.0, 1.6, -8.0),
        Flashed::default(),
        super::super::devices::FlashTargets(vec![monster]),
    ));
    app.world_mut()
        .get_mut::<PlayerInput>(player)
        .unwrap()
        .movement = Vec2::Y;
    app.world_mut()
        .get_mut::<Stamina>(player)
        .unwrap()
        .sprinting = true;
    let mut searched = false;
    loop {
        app.update();
        if app.world().get::<Flashed>(player).is_none() {
            break;
        }
        let pursuit = monster_state(&app, monster)
            .pursuit
            .expect("search must persist through the flash");
        assert_eq!(
            pursuit.last_sensed, last,
            "flashed player must not be tracked"
        );
        searched |= pursuit.searching;
        assert!(app.world().get::<Caught>(player).is_none());
    }
    assert!(searched, "monster must search the last known position");
    let mut tracked = monster_state(&app, monster)
        .pursuit
        .is_some_and(|pursuit| pursuit.last_sensed == Vec2::new(0.0, -8.0));
    for _ in 0..5 {
        app.update();
        tracked |= monster_state(&app, monster)
            .pursuit
            .is_some_and(|pursuit| pursuit.last_sensed == Vec2::new(0.0, -8.0));
    }
    assert!(
        tracked,
        "monster must sense the player again after the flash"
    );
}

#[test]
fn hiding_while_flashed_in_plain_view_is_not_witnessed_and_stays_safe() {
    use crate::controller::PlayerController;
    use crate::levels::{
        Caught, Flashed, Hidden, HidingPlugin, HidingSpot, Monster, Room, UseHidingSpot,
    };

    let mut app = flash_app();
    app.add_plugins(HidingPlugin);
    app.world_mut().spawn(Room(Rect::new(-5.0, -8.0, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(0.0, 0.0, -4.0)))
        .id();
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, 1.6, -2.9),
            Flashed::default(),
            super::super::devices::FlashTargets(vec![monster]),
        ))
        .id();
    app.update();
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(app.world().get::<Hidden>(player).is_some());
    assert!(app.world().get::<super::WitnessedHiding>(player).is_none());
    for _ in 0..120 {
        app.update();
        assert!(app.world().get::<Caught>(player).is_none());
    }
    assert!(app.world().get::<Flashed>(player).is_none());
    assert!(app
        .world()
        .get::<Hidden>(player)
        .is_some_and(Hidden::settled));
}

#[test]
fn witnessed_hiding_is_not_pulled_out_while_flashed_but_is_after_expiry() {
    use crate::controller::PlayerController;
    use crate::levels::{
        Caught, Flashed, Hidden, HidingPhase, HidingPlugin, HidingSpot, Monster, Room,
        UseHidingSpot,
    };

    let mut app = flash_app();
    app.add_plugins(HidingPlugin);
    app.world_mut().spawn(Room(Rect::new(-5.0, -8.0, 5.0, 5.0)));
    let spot = app
        .world_mut()
        .spawn((HidingSpot::Locker, Transform::from_xyz(0.0, 0.0, -4.0)))
        .id();
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -2.9)))
        .id();
    app.update();
    app.world_mut()
        .write_message(UseHidingSpot { player, spot });
    app.update();
    assert!(app.world().get::<super::WitnessedHiding>(player).is_some());
    app.world_mut().entity_mut(player).insert((
        Flashed::default(),
        super::super::devices::FlashTargets(vec![monster]),
    ));
    while app.world().get::<Flashed>(player).is_some() {
        app.update();
        assert!(!app
            .world()
            .get::<Hidden>(player)
            .is_some_and(|hidden| matches!(hidden.phase, HidingPhase::Leaving(_))));
        assert!(app.world().get::<Caught>(player).is_none());
    }
    for _ in 0..100 {
        app.update();
        if app.world().get::<Caught>(player).is_some() {
            break;
        }
    }
    assert_eq!(app.world().get::<Caught>(player).unwrap().monster, monster);
}

#[test]
fn flash_after_contact_does_not_cancel_the_catch() {
    use crate::controller::PlayerController;
    use crate::levels::{Caught, Flashed, Monster, Room};

    let mut app = flash_app();
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, -1.0)))
        .id();
    for _ in 0..3 {
        app.update();
    }
    let started = *app.world().get::<Caught>(player).unwrap();
    app.world_mut()
        .entity_mut(player)
        .insert(Flashed::default());
    for _ in 0..30 {
        app.update();
    }
    let caught = app.world().get::<Caught>(player).unwrap();
    assert_eq!(caught.monster, monster);
    assert!(caught.remaining < started.remaining);
    assert!(caught.finished());
}

#[test]
fn detector_cues_near_the_monster_never_alert_it() {
    use crate::controller::PlayerController;
    use crate::levels::{Detector, Monster, Room};
    use game_audio::{PlaySound, Sound};

    #[derive(Resource, Default)]
    struct Pulses(usize);

    fn count(mut sounds: MessageReader<PlaySound>, mut pulses: ResMut<Pulses>) {
        pulses.0 += sounds
            .read()
            .filter(|cue| cue.sound == Sound::DetectorNearby)
            .count();
    }

    let mut app = flash_app();
    app.init_resource::<Pulses>().add_systems(PostUpdate, count);
    app.world_mut().spawn(Room(Rect::new(-5.0, -5.0, 5.0, 5.0)));
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::IDENTITY))
        .id();
    app.world_mut().spawn((
        PlayerController,
        Detector::default(),
        Transform::from_xyz(0.0, 1.6, 2.5),
    ));
    for _ in 0..60 {
        app.update();
        assert!(monster_state(&app, monster).pursuit.is_none());
    }
    assert!(app.world().resource::<Pulses>().0 >= 15);
}
