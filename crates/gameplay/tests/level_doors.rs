use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;
use gameplay::levels::{
    build_first_floor, Ceiling, Door, DoorOf, DoorRef, DoorState, Doors, Passage, Room,
};

#[test]
fn every_first_floor_room_has_a_ceiling() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut rooms = world.query::<(&Room, &Ceiling)>();
    let rooms: Vec<_> = rooms.iter(world).collect();
    assert_eq!(rooms.len(), 18);
    assert!(rooms.iter().all(|(_, ceiling)| ceiling.0 == "ceiling_tile"));
}

#[test]
fn first_floor_openings_link_rooms_and_align_with_walls() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut room_query = world.query::<(Entity, &Name, &Room, &Doors)>();
    let rooms: Vec<_> = room_query
        .iter(world)
        .map(|(entity, name, room, doors)| {
            (
                entity,
                name.as_str().to_owned(),
                room.0,
                doors.iter().collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(rooms.len(), 18);

    let mut door_query = world.query::<(Entity, &Door)>();
    let doors: Vec<_> = door_query
        .iter(world)
        .map(|(entity, door)| {
            (
                entity,
                door.position,
                door.rotation,
                door.state,
                door.frame.clone(),
                door.panel.clone(),
            )
        })
        .collect();
    assert_eq!(doors.len(), 18);
    assert!(doors.iter().all(|(_, _, _, state, frame, panel)| {
        *state == DoorState::Closed && frame == "wall_doorway" && panel == "door_panel"
    }));

    let mut passage_query = world.query::<(Entity, &Passage)>();
    let passages: Vec<_> = passage_query
        .iter(world)
        .map(|(entity, passage)| (entity, passage.0))
        .collect();
    assert_eq!(passages.len(), 10);

    let mut names = world.query_filtered::<&Name, Or<(With<Door>, With<Passage>)>>();
    let names: Vec<_> = names
        .iter(world)
        .map(|name| name.as_str().to_owned())
        .collect();
    assert_eq!(names.len(), 28);
    assert!(names.iter().all(|name| !name.is_empty()));
    assert_eq!(names.iter().collect::<HashSet<_>>().len(), 28);

    assert!(doors.iter().any(|(_, at, rotation, _, _, _)| {
        *at == Vec2::new(0.0, -31.25) && *rotation == Quat::from_rotation_y(std::f32::consts::PI)
    }));
    assert!(passages.iter().any(|(_, at)| *at == Vec2::new(0.0, -16.25)));
    assert!(!passages.iter().any(|(_, at)| *at == Vec2::new(6.25, 2.5)));
    assert!(doors.iter().any(|(_, at, rotation, ..)| {
        *at == Vec2::new(6.25, -5.0)
            && *rotation == Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
    }));
    assert!(passages.iter().any(|(_, at)| *at == Vec2::new(10.0, -3.75)));

    let room_openings = |room_name: &str| {
        let (_, _, _, links) = rooms
            .iter()
            .find(|(_, name, ..)| name == room_name)
            .unwrap();
        links
            .iter()
            .map(|&link| world.get::<DoorRef>(link).unwrap().0)
            .collect::<HashSet<_>>()
    };
    let prep = room_openings("prep");
    let hiding = room_openings("hiding");
    let east_hall = room_openings("east_hall");
    assert_eq!(prep.len(), 2);
    assert_eq!(hiding.len(), 2);
    assert_eq!(prep.intersection(&hiding).count(), 1);
    assert_eq!(prep.intersection(&east_hall).count(), 1);
    assert_eq!(hiding.intersection(&east_hall).count(), 1);

    for (entity, name, bounds, references) in &rooms {
        for &link in references {
            let belongs_to = world.get::<DoorOf>(link).unwrap();
            assert_eq!(belongs_to.0, *entity);
            let opening = world.get::<DoorRef>(link).unwrap().0;
            let position = world
                .get::<Door>(opening)
                .map(|door| door.position)
                .or_else(|| world.get::<Passage>(opening).map(|passage| passage.0))
                .unwrap();
            let tile_center = |coordinate: f32, min: f32| {
                let tile = (coordinate - min - 1.25) / 2.5;
                (tile - tile.round()).abs() < 0.001
            };
            let on_horizontal = (position.y == bounds.min.y || position.y == bounds.max.y)
                && position.x >= bounds.min.x + 1.25
                && position.x <= bounds.max.x - 1.25
                && tile_center(position.x, bounds.min.x);
            let on_vertical = (position.x == bounds.min.x || position.x == bounds.max.x)
                && position.y >= bounds.min.y + 1.25
                && position.y <= bounds.max.y - 1.25
                && tile_center(position.y, bounds.min.y);
            assert!(
                on_horizontal || on_vertical,
                "{name} has an opening off its wall"
            );
        }
        if name == "boiler" || name == "storage" {
            assert_eq!(references.len(), 1, "{name} must remain single-entry");
        } else {
            assert!(references.len() >= 2, "{name} needs a second entrance");
        }
    }

    for (left, (_, left_name, left_bounds, _)) in rooms.iter().enumerate() {
        for (_, right_name, right_bounds, _) in rooms.iter().skip(left + 1) {
            assert!(
                left_bounds.max.x <= right_bounds.min.x
                    || right_bounds.max.x <= left_bounds.min.x
                    || left_bounds.max.y <= right_bounds.min.y
                    || right_bounds.max.y <= left_bounds.min.y,
                "{left_name} overlaps {right_name}"
            );
        }
    }

    let mut neighbors: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for opening in doors
        .iter()
        .map(|(entity, ..)| *entity)
        .chain(passages.iter().map(|(entity, _)| *entity))
    {
        let linked_rooms: Vec<_> = rooms
            .iter()
            .filter(|(_, _, _, links)| {
                links
                    .iter()
                    .any(|&link| world.get::<DoorRef>(link).unwrap().0 == opening)
            })
            .map(|(entity, ..)| *entity)
            .collect();
        assert!((1..=2).contains(&linked_rooms.len()));
        if let [a, b] = linked_rooms.as_slice() {
            neighbors.entry(*a).or_default().push(*b);
            neighbors.entry(*b).or_default().push(*a);
        }
    }
    let start = rooms
        .iter()
        .find(|(_, name, ..)| name == "reception")
        .unwrap()
        .0;
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([start]);
    while let Some(current) = queue.pop_front() {
        if seen.insert(current) {
            queue.extend(neighbors.get(&current).into_iter().flatten().copied());
        }
    }
    assert_eq!(
        seen.len(),
        rooms.len(),
        "all rooms must connect to reception"
    );
}
