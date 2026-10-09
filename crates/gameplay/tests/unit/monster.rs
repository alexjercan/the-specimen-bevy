use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy};

use super::{point, room_at, route, GRID, MODEL_LIFT, MODEL_PIVOT, MODEL_SCALE, TURN_SPEED};

#[test]
fn patrol_turns_at_a_bounded_rate_before_moving_in_a_new_direction() {
    let mut transform = Transform::IDENTITY;
    assert!(!super::turn_toward(&mut transform, Vec2::X, 0.1));
    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    assert!((yaw + TURN_SPEED * 0.1).abs() < 0.001);
    for _ in 0..8 {
        super::turn_toward(&mut transform, Vec2::X, 0.1);
    }
    assert!(super::turn_toward(&mut transform, Vec2::X, 0.1));
    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    assert!((yaw + std::f32::consts::FRAC_PI_2).abs() < 0.001);
}

#[test]
fn patrol_turns_across_the_yaw_wrap_by_the_shortest_path() {
    let mut transform = Transform::from_rotation(Quat::from_rotation_y(2.6));
    let target = Vec2::new(0.5155, 0.8569).normalize();
    assert!(!super::turn_toward(&mut transform, target, 0.01));
    let expected = Quat::from_rotation_y(2.6 + TURN_SPEED * 0.01);
    assert!(transform.rotation.angle_between(expected) < 0.001);
    for _ in 0..60 {
        super::turn_toward(&mut transform, target, 0.01);
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
    let heard = &app.world().resource::<Heard>().0;
    assert_eq!(
        heard
            .iter()
            .filter(|&&cue| cue == Sound::MonsterPresence)
            .count(),
        3
    );
    assert!(!heard.iter().any(|cue| matches!(cue, Sound::MonsterStep(_))));
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
fn patrol_opens_traverses_and_closes_an_unlocked_door() {
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
    let mut closed_after_crossing = false;
    for _ in 0..1100 {
        app.update();
        let world = app.world_mut();
        let door = world.query::<&Door>().single(world).unwrap();
        let position = world.get::<Transform>(monster).unwrap().translation.x;
        opened |= door.state == DoorState::Open;
        closed_after_crossing |= opened && position > 1.8 && door.state == DoorState::Closed;
        if closed_after_crossing {
            break;
        }
    }
    assert!(opened, "patrol must open the door");
    assert!(
        closed_after_crossing,
        "patrol must cross and close the door"
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
fn route_grid_is_half_meter() {
    assert_eq!(
        point(super::node(Vec2::new(-1.1, 2.1))),
        Vec2::new(-1.0, 2.0)
    );
}
