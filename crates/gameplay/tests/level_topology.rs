use std::collections::{HashMap, HashSet, VecDeque};

use bevy::{ecs::system::SystemState, prelude::*};
use gameplay::{
    controller::{door_frames, wall_obstacles},
    levels::{
        build_first_floor, DevicePlaceholder, Door, DoorOf, DoorRef, DoorSwing, Doors, FusePickup,
        FuseSeed, FuseZone, HidingSpot, Passage, Prop, PropCollider, Room, FUSE_TABLES,
    },
};

type Structure<'w, 's> = (
    Query<'w, 's, (&'static Room, Option<&'static Doors>)>,
    Query<'w, 's, (&'static DoorRef, &'static DoorOf)>,
    Query<'w, 's, (&'static Door, &'static DoorSwing)>,
    Query<'w, 's, &'static Passage>,
);

const PLAYER_RADIUS: f32 = 0.25;
const SPAWN: Vec2 = Vec2::new(2.5, -27.5);
const WALK_STEP: f32 = 0.25;

fn first_floor(seed: u64) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(FuseSeed(seed))
        .add_systems(Startup, build_first_floor);
    app.update();
    app
}

struct RoomGraph {
    by_name: HashMap<String, Entity>,
    names: HashMap<Entity, String>,
    bounds: HashMap<Entity, Rect>,
    opening_count: HashMap<Entity, usize>,
    neighbors: HashMap<Entity, Vec<Entity>>,
}

impl RoomGraph {
    fn entity(&self, name: &str) -> Entity {
        *self
            .by_name
            .get(name)
            .unwrap_or_else(|| panic!("missing room {name}"))
    }

    fn reachable_excluding(&self, start: Entity, excluded: Option<Entity>) -> HashSet<Entity> {
        let mut seen = HashSet::new();
        if Some(start) == excluded {
            return seen;
        }
        let mut queue = VecDeque::from([start]);
        while let Some(current) = queue.pop_front() {
            if seen.insert(current) {
                queue.extend(
                    self.neighbors
                        .get(&current)
                        .into_iter()
                        .flatten()
                        .filter(|&&n| Some(n) != excluded)
                        .copied(),
                );
            }
        }
        seen
    }
}

fn room_graph(app: &mut App) -> RoomGraph {
    let world = app.world_mut();
    let mut room_query = world.query::<(Entity, &Name, &Room, &Doors)>();
    let rooms: Vec<(Entity, String, Rect, Vec<Entity>)> = room_query
        .iter(world)
        .map(|(entity, name, room, doors)| {
            (
                entity,
                name.as_str().to_owned(),
                room.0,
                doors.iter().collect(),
            )
        })
        .collect();

    let mut opening_rooms: HashMap<Entity, Vec<Entity>> = HashMap::new();
    let mut opening_count = HashMap::new();
    for (entity, _, _, links) in &rooms {
        opening_count.insert(*entity, links.len());
        for &link in links {
            let opening = world.get::<DoorRef>(link).unwrap().0;
            opening_rooms.entry(opening).or_default().push(*entity);
        }
    }

    let mut neighbors: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for rooms_for_opening in opening_rooms.values() {
        if let [a, b] = rooms_for_opening.as_slice() {
            neighbors.entry(*a).or_default().push(*b);
            neighbors.entry(*b).or_default().push(*a);
        }
    }

    RoomGraph {
        by_name: rooms.iter().map(|(e, n, ..)| (n.clone(), *e)).collect(),
        names: rooms.iter().map(|(e, n, ..)| (*e, n.clone())).collect(),
        bounds: rooms.iter().map(|(e, _, r, _)| (*e, *r)).collect(),
        opening_count,
        neighbors,
    }
}

fn back_rooms(graph: &RoomGraph) -> HashSet<Entity> {
    graph
        .bounds
        .iter()
        .filter(|(_, rect)| rect.min.y >= 3.75)
        .map(|(&entity, _)| entity)
        .collect()
}

#[test]
fn every_room_is_reachable_from_exit() {
    let mut app = first_floor(0);
    let graph = room_graph(&mut app);
    let exit = graph.entity("exit");
    let seen = graph.reachable_excluding(exit, None);
    assert_eq!(seen.len(), graph.names.len(), "a room cannot reach exit");
}

#[test]
fn back_wing_is_entered_from_the_middle_only_via_west_and_east_hall() {
    let mut app = first_floor(0);
    let graph = room_graph(&mut app);
    let back = back_rooms(&graph);
    assert_eq!(back.len(), 8);

    let mut entries = HashSet::new();
    let mut crossings = 0;
    for (&room, list) in &graph.neighbors {
        if back.contains(&room) {
            continue;
        }
        for &neighbor in list {
            if back.contains(&neighbor) {
                entries.insert(graph.names[&room].clone());
                crossings += 1;
            }
        }
    }
    assert_eq!(
        crossings, 2,
        "expected exactly two crossings into the back wing"
    );
    assert_eq!(
        entries,
        HashSet::from(["west_hall".to_owned(), "east_hall".to_owned()])
    );
}

#[test]
fn storage_and_boiler_remain_single_entry_rooms() {
    let mut app = first_floor(0);
    let graph = room_graph(&mut app);
    for name in ["storage", "boiler"] {
        let entity = graph.entity(name);
        assert_eq!(
            graph.opening_count[&entity], 1,
            "{name} must stay single-entry"
        );
    }
}

#[test]
fn fuse_candidates_and_lab_have_no_unavoidable_choke() {
    let mut app = first_floor(0);
    let graph = room_graph(&mut app);
    let exit = graph.entity("exit");
    let mut targets: Vec<&str> = FUSE_TABLES.iter().map(|table| table.room).collect();
    targets.push("lab");

    for target_name in targets {
        let target = graph.entity(target_name);
        for (&other, other_name) in &graph.names {
            if other == exit || other == target {
                continue;
            }
            let seen = graph.reachable_excluding(exit, Some(other));
            assert!(
                seen.contains(&target),
                "removing {other_name} isolates {target_name} from exit"
            );
        }
    }
}

#[test]
fn front_to_back_connectivity_survives_losing_lab_or_one_hall() {
    let mut app = first_floor(0);
    let graph = room_graph(&mut app);
    let exit = graph.entity("exit");
    let back = back_rooms(&graph);

    for blocker in ["lab", "west_hall", "east_hall"] {
        let blocked = graph.entity(blocker);
        let seen = graph.reachable_excluding(exit, Some(blocked));
        for &room in &back {
            assert!(
                seen.contains(&room),
                "removing {blocker} isolates {} from exit",
                graph.names[&room]
            );
        }
    }
}

fn rotate(point: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(c * point.x + s * point.y, -s * point.x + c * point.y)
}

fn walk_obstacles(app: &mut App) -> Vec<(Vec2, Vec2, f32)> {
    let world = app.world_mut();
    let mut obstacles = {
        let mut structure = SystemState::<Structure>::new(world);
        let (rooms, links, doors, passages) =
            structure.get(world).expect("structure queries are valid");
        let mut obstacles = wall_obstacles(&rooms, &links, &doors, &passages);
        for (door, _) in &doors {
            obstacles.extend(door_frames(door));
        }
        obstacles
    };
    let mut props = world.query::<(&PropCollider, &Transform)>();
    for (collider, transform) in props.iter(world) {
        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let scale = transform.scale.xz();
        let center = transform.translation.xz() + rotate(collider.center * scale, yaw);
        obstacles.push((center, collider.half * scale.abs(), yaw));
    }
    obstacles
}

fn free(point: Vec2, obstacles: &[(Vec2, Vec2, f32)]) -> bool {
    obstacles.iter().all(|&(center, half, angle)| {
        let local = rotate(point - center, -angle);
        let clamped = local.clamp(-half, half);
        (local - clamped).length_squared() >= PLAYER_RADIUS * PLAYER_RADIUS
    })
}

fn building_bounds(app: &mut App) -> Rect {
    let world = app.world_mut();
    let mut rooms = world.query::<&Room>();
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for room in rooms.iter(world) {
        min = min.min(room.0.min);
        max = max.max(room.0.max);
    }
    Rect {
        min: min - Vec2::splat(1.0),
        max: max + Vec2::splat(1.0),
    }
}

fn reachable_positions(
    start: Vec2,
    obstacles: &[(Vec2, Vec2, f32)],
    step: f32,
    bounds: Rect,
) -> HashSet<(i32, i32)> {
    let key = |point: Vec2| {
        (
            (point.x / step).round() as i32,
            (point.y / step).round() as i32,
        )
    };
    let in_bounds = |point: Vec2| point.cmpge(bounds.min).all() && point.cmple(bounds.max).all();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let start_key = key(start);
    if free(start, obstacles) {
        visited.insert(start_key);
        queue.push_back(start_key);
    }
    while let Some((x, z)) = queue.pop_front() {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next_key = (x + dx, z + dz);
            if visited.contains(&next_key) {
                continue;
            }
            let next = Vec2::new(next_key.0 as f32 * step, next_key.1 as f32 * step);
            if !in_bounds(next) {
                continue;
            }
            if free(next, obstacles) {
                visited.insert(next_key);
                queue.push_back(next_key);
            }
        }
    }
    visited
}

#[test]
fn player_can_walk_from_spawn_to_every_fuse_table_and_placeholder() {
    let mut app = first_floor(0);
    let bounds = building_bounds(&mut app);
    let obstacles = walk_obstacles(&mut app);
    let visited = reachable_positions(SPAWN, &obstacles, WALK_STEP, bounds);
    assert!(!visited.is_empty(), "spawn point is blocked");

    let covers = |target: Vec2| {
        visited.iter().any(|&(x, z)| {
            Vec2::new(x as f32 * WALK_STEP, z as f32 * WALK_STEP).distance(target) <= 1.0
        })
    };

    for table in &FUSE_TABLES {
        assert!(
            covers(table.position.xz()),
            "cannot reach {} table on foot",
            table.room
        );
    }

    let world = app.world_mut();
    let placeholders: Vec<Vec2> = world
        .query::<(&DevicePlaceholder, &Transform)>()
        .iter(world)
        .map(|(_, transform)| transform.translation.xz())
        .collect();
    assert_eq!(placeholders.len(), 2);
    for position in placeholders {
        assert!(
            covers(position),
            "cannot reach placeholder at {position} on foot"
        );
    }
}

fn fuse_placements(seed: u64) -> Vec<(usize, Vec3)> {
    let mut app = first_floor(seed);
    let world = app.world_mut();
    let mut placements: Vec<(usize, Vec3)> = world
        .query::<&FusePickup>()
        .iter(world)
        .map(|fuse| (fuse.slot, FUSE_TABLES[fuse.slot].position))
        .collect();
    placements.sort_by_key(|(slot, _)| *slot);
    placements
}

#[test]
fn three_fuses_spawn_in_one_front_and_two_back_tables_reproducibly() {
    for seed in 0..200u64 {
        let first = fuse_placements(seed);
        assert_eq!(first.len(), 3);
        let zones: Vec<FuseZone> = first
            .iter()
            .map(|(slot, _)| FUSE_TABLES[*slot].zone)
            .collect();
        assert_eq!(
            zones
                .iter()
                .filter(|&&zone| zone == FuseZone::Front)
                .count(),
            1
        );
        assert_eq!(
            zones.iter().filter(|&&zone| zone == FuseZone::Back).count(),
            2
        );
        assert_eq!(first, fuse_placements(seed));
    }
}

#[test]
fn device_placeholders_are_fixed_and_disjoint_from_fuses_and_hiding() {
    let positions = |seed: u64| -> HashMap<DevicePlaceholder, Vec3> {
        let mut app = first_floor(seed);
        let world = app.world_mut();
        world
            .query::<(&DevicePlaceholder, &Transform)>()
            .iter(world)
            .map(|(kind, transform)| (*kind, transform.translation))
            .collect()
    };
    let p0 = positions(0);
    let p1 = positions(12345);
    assert_eq!(p0.len(), 2);
    assert_eq!(p0, p1);

    let mut app = first_floor(0);
    let world = app.world_mut();
    let bounds: HashMap<String, Rect> = world
        .query::<(&Name, &Room)>()
        .iter(world)
        .map(|(name, room)| (name.as_str().to_owned(), room.0))
        .collect();
    let flashbang = p0[&DevicePlaceholder::Flashbang];
    let detector = p0[&DevicePlaceholder::Detector];
    assert!(bounds["storage"].contains(flashbang.xz()));
    assert!(bounds["lab"].contains(detector.xz()));

    let devices: HashSet<Entity> = world
        .query_filtered::<Entity, With<DevicePlaceholder>>()
        .iter(world)
        .collect();
    let fuses: HashSet<Entity> = world
        .query_filtered::<Entity, With<FusePickup>>()
        .iter(world)
        .collect();
    assert!(devices.is_disjoint(&fuses));
    let hiding_props: HashSet<Entity> = world
        .query_filtered::<Entity, With<HidingSpot>>()
        .iter(world)
        .collect();
    assert!(devices.is_disjoint(&hiding_props));

    for table in &FUSE_TABLES {
        assert!(table.position.xz().distance(flashbang.xz()) > 1.8);
        assert!(table.position.xz().distance(detector.xz()) > 1.8);
    }
}

#[test]
fn fuse_tables_and_placeholders_keep_clearance_from_hiding_props_and_openings() {
    let mut app = first_floor(0);
    let world = app.world_mut();
    let hiding_positions: Vec<Vec2> = world
        .query::<(&Prop, &Transform)>()
        .iter(world)
        .filter(|(prop, _)| prop.0 == "concept_locker" || prop.0 == "concept_table")
        .map(|(_, transform)| transform.translation.xz())
        .collect();
    let mut openings: Vec<Vec2> = world
        .query::<&Door>()
        .iter(world)
        .map(|door| door.position)
        .collect();
    openings.extend(
        world
            .query::<&Passage>()
            .iter(world)
            .map(|passage| passage.0),
    );

    for table in &FUSE_TABLES {
        let position = table.position.xz();
        assert!(
            hiding_positions.iter().all(|&p| position.distance(p) > 1.8),
            "{} fuse is too close to a hiding prop",
            table.room
        );
        assert!(
            openings.iter().all(|&p| position.distance(p) > 1.5),
            "{} fuse blocks an opening",
            table.room
        );
    }

    let placeholders: Vec<Vec2> = world
        .query::<(&DevicePlaceholder, &Transform)>()
        .iter(world)
        .map(|(_, transform)| transform.translation.xz())
        .collect();
    assert_eq!(placeholders.len(), 2);
    for position in placeholders {
        assert!(
            openings.iter().all(|&p| position.distance(p) > 1.0),
            "placeholder at {position} blocks an opening"
        );
    }
}

#[test]
fn rear_hall_is_decorated_and_keeps_clear_routes() {
    let mut app = first_floor(0);
    let graph = room_graph(&mut app);
    let rear_hall = graph.entity("rear_hall");
    assert_eq!(graph.opening_count[&rear_hall], 4);

    let rear_hall_bounds = Rect::new(-13.75, 16.25, 13.75, 18.75);
    let openings = [
        Vec2::new(-5.0, 16.25),
        Vec2::new(5.0, 16.25),
        Vec2::new(-10.0, 16.25),
        Vec2::new(10.0, 16.25),
    ];

    {
        let world = app.world_mut();
        let mut all_openings: Vec<Vec2> = world
            .query::<&Door>()
            .iter(world)
            .map(|door| door.position)
            .collect();
        all_openings.extend(
            world
                .query::<&Passage>()
                .iter(world)
                .map(|passage| passage.0),
        );
        for opening in &openings {
            assert!(
                all_openings.contains(opening),
                "missing rear_hall opening at {opening}"
            );
        }

        let props_inside: Vec<Vec2> = world
            .query::<(&Prop, &Transform)>()
            .iter(world)
            .filter(|(_, transform)| rear_hall_bounds.contains(transform.translation.xz()))
            .map(|(_, transform)| transform.translation.xz())
            .collect();
        assert!(
            props_inside.len() >= 6,
            "expected at least 6 props in rear_hall, found {}",
            props_inside.len()
        );
        assert!(
            props_inside.iter().any(|position| position.x < -11.25),
            "missing a prop in the west rear_hall alcove"
        );
        assert!(
            props_inside.iter().any(|position| position.x > 11.25),
            "missing a prop in the east rear_hall alcove"
        );

        let colliders: Vec<(Vec2, Vec2, f32)> = world
            .query::<(&Transform, &PropCollider)>()
            .iter(world)
            .filter(|(transform, _)| rear_hall_bounds.contains(transform.translation.xz()))
            .map(|(transform, collider)| {
                let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
                let scale = transform.scale.xz();
                let center = transform.translation.xz() + rotate(collider.center * scale, yaw);
                (center, collider.half * scale.abs(), yaw)
            })
            .collect();
        for opening in &openings {
            for &(center, half, angle) in &colliders {
                let local = rotate(*opening - center, -angle);
                let clamped = local.clamp(-half, half);
                assert!(
                    (local - clamped).length() >= 1.0,
                    "a prop footprint at {center} blocks rear_hall opening {opening}"
                );
            }
        }
    }

    let obstacles = walk_obstacles(&mut app);
    let mut sets = Vec::new();
    for opening in &openings {
        let start = Vec2::new(opening.x, rear_hall_bounds.min.y + 0.5);
        let visited = reachable_positions(start, &obstacles, WALK_STEP, rear_hall_bounds);
        assert!(
            !visited.is_empty(),
            "rear_hall opening at {opening} is blocked"
        );
        sets.push(visited);
    }
    let first = &sets[0];
    for (index, visited) in sets.iter().enumerate().skip(1) {
        assert!(
            !first.is_disjoint(visited),
            "rear_hall opening {index} is not connected to opening 0"
        );
    }
}
