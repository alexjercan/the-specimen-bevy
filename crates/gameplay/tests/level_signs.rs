use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use gameplay::levels::{build_first_floor, Door, DoorRef, Doors, Passage, Prop, Room};

const READER_DISTANCE: f32 = 1.0;
const LABEL_TO_ARROW: f32 = 0.79;
const ALTERNATE_ROUTE_SLACK: usize = 1;
const OUTSIDE: Vec2 = Vec2::new(0.0, -31.25);

struct Floor {
    rooms: Vec<(String, Rect)>,
    openings: HashMap<String, Vec<(Vec2, Option<String>)>>,
    props: Vec<(String, Transform, bool)>,
}

impl Floor {
    fn room_at(&self, point: Vec2) -> &str {
        self.rooms
            .iter()
            .find(|(_, rect)| rect.contains(point))
            .map(|(name, _)| name.as_str())
            .unwrap_or_else(|| panic!("no room at {point}"))
    }

    fn hops(&self, target: &str, excluded: &str) -> HashMap<String, usize> {
        let mut hops = HashMap::from([(target.to_owned(), 0)]);
        let mut queue = VecDeque::from([target.to_owned()]);
        while let Some(room) = queue.pop_front() {
            let next = hops[&room] + 1;
            for neighbor in self.openings[&room].iter().filter_map(|(_, n)| n.as_ref()) {
                if neighbor != excluded && !hops.contains_key(neighbor) {
                    hops.insert(neighbor.clone(), next);
                    queue.push_back(neighbor.clone());
                }
            }
        }
        hops
    }

    fn waypoints(&self, from: &str, target: &str, slack: usize) -> Vec<Vec2> {
        if from == target {
            assert_eq!(target, "exit");
            return vec![OUTSIDE];
        }
        let shortest = self.hops(target, "")[from];
        let onward = self.hops(target, from);
        self.openings[from]
            .iter()
            .filter(|(_, neighbor)| {
                neighbor
                    .as_ref()
                    .and_then(|neighbor| onward.get(neighbor))
                    .is_some_and(|&hops| hops < shortest + slack)
            })
            .map(|(position, _)| *position)
            .collect()
    }

    fn arrows(&self) -> impl Iterator<Item = &Transform> {
        self.props
            .iter()
            .filter(|(module, ..)| module == "sign_arrow")
            .map(|(_, transform, _)| transform)
    }

    fn label_for(&self, arrow: &Transform) -> &str {
        let expected = arrow.translation + face_rotation(arrow) * Vec3::X * LABEL_TO_ARROW;
        self.props
            .iter()
            .find(|(module, transform, _)| {
                module.starts_with("sign_label_") && transform.translation.distance(expected) < 0.01
            })
            .map(|(module, ..)| module.as_str())
            .unwrap_or_else(|| panic!("arrow at {} has no label", arrow.translation))
    }
}

fn face_rotation(arrow: &Transform) -> Quat {
    let forward = arrow.rotation * Vec3::NEG_Z;
    Quat::from_rotation_y((-forward.x).atan2(-forward.z))
}

fn floor() -> Floor {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();
    let world = app.world_mut();

    let rooms: Vec<(Entity, String, Rect, Vec<Entity>)> = world
        .query::<(Entity, &Name, &Room, &Doors)>()
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
    let mut linked: HashMap<Entity, Vec<String>> = HashMap::new();
    for (_, name, _, links) in &rooms {
        for &link in links {
            let opening = world.get::<DoorRef>(link).unwrap().0;
            linked.entry(opening).or_default().push(name.clone());
        }
    }
    let mut openings: HashMap<String, Vec<(Vec2, Option<String>)>> = HashMap::new();
    for (opening, names) in &linked {
        let position = world
            .get::<Door>(*opening)
            .map(|door| door.position)
            .or_else(|| world.get::<Passage>(*opening).map(|passage| passage.0))
            .unwrap();
        for name in names {
            let other = names.iter().find(|other| *other != name).cloned();
            openings
                .entry(name.clone())
                .or_default()
                .push((position, other));
        }
    }

    let parents: HashMap<Entity, Transform> = world
        .query::<(Entity, &Transform)>()
        .iter(world)
        .map(|(entity, transform)| (entity, *transform))
        .collect();
    let props = world
        .query::<(&Prop, &Transform, Option<&ChildOf>)>()
        .iter(world)
        .map(|(prop, transform, parent)| {
            let global = parent
                .map(|parent| parents[&parent.parent()] * *transform)
                .unwrap_or(*transform);
            (prop.0.clone(), global, parent.is_some())
        })
        .collect();

    Floor {
        rooms: rooms
            .into_iter()
            .map(|(_, name, rect, _)| (name, rect))
            .collect(),
        openings,
        props,
    }
}

fn target_room(label: &str) -> &'static str {
    match label {
        "sign_label_exit" => "exit",
        "sign_label_lab" => "lab",
        "sign_label_storage" => "storage",
        "sign_label_office" => "office",
        "sign_label_maintenance" => "maintenance",
        "sign_label_boiler_room" => "boiler",
        other => panic!("unexpected wayfinding label {other}"),
    }
}

fn reader(arrow: &Transform) -> (Vec2, Vec2) {
    let normal = (face_rotation(arrow) * Vec3::NEG_Z).xz();
    (arrow.translation.xz() + normal * READER_DISTANCE, -normal)
}

fn heading(arrow: &Transform) -> Vec3 {
    arrow.rotation * Vec3::X
}

#[test]
fn every_arrow_points_along_a_route_to_its_label() {
    let floor = floor();
    let mut checked = 0;
    for arrow in floor.arrows() {
        let label = floor.label_for(arrow);
        let target = target_room(label);
        let (position, forward) = reader(arrow);
        let room = floor.room_at(position);
        let pointing = heading(arrow);
        assert!(
            pointing.y > -0.01,
            "{label} arrow at {} points down",
            arrow.translation
        );
        let direction = if pointing.y > 0.99 {
            forward
        } else {
            assert!(pointing.y.abs() < 0.01);
            pointing.xz().normalize()
        };
        let alternates = floor
            .arrows()
            .filter(|other| {
                reader(other).0.distance(position) < 0.01 && floor.label_for(other) == label
            })
            .count();
        let slack = if alternates > 1 {
            ALTERNATE_ROUTE_SLACK
        } else {
            0
        };
        let waypoints = floor.waypoints(room, target, slack);
        assert!(!waypoints.is_empty(), "{label} read from {room}");
        assert!(
            waypoints
                .iter()
                .any(|waypoint| (*waypoint - position).normalize().dot(direction) > 0.7),
            "{label} arrow at {} read from {room} points {direction}, route {waypoints:?}",
            arrow.translation
        );
        checked += 1;
    }
    assert_eq!(checked, 22);
}

#[test]
fn exit_arrows_are_spread_across_front_middle_and_back() {
    let floor = floor();
    let exits: Vec<Vec3> = floor
        .arrows()
        .filter(|arrow| floor.label_for(arrow) == "sign_label_exit")
        .map(|arrow| arrow.translation)
        .collect();
    let front = exits.iter().filter(|position| position.z < -8.75).count();
    let back = exits.iter().filter(|position| position.z > 3.75).count();
    let middle = exits.len() - front - back;
    assert!(front >= 2, "front exits {front}");
    assert!(middle >= 2, "middle exits {middle}");
    assert!(back >= 2, "back exits {back}");
}

#[test]
fn rear_hall_points_to_both_back_wing_routes() {
    let floor = floor();
    let headings: Vec<Vec3> = floor
        .arrows()
        .filter(|arrow| {
            floor.label_for(arrow) == "sign_label_exit"
                && floor.room_at(reader(arrow).0) == "rear_hall"
        })
        .map(heading)
        .collect();
    assert!(headings.iter().any(|heading| heading.x > 0.99));
    assert!(headings.iter().any(|heading| heading.x < -0.99));
}

#[test]
fn hangers_keep_clear_of_ceiling_lights_and_openings() {
    let floor = floor();
    let lights: Vec<Vec2> = floor
        .props
        .iter()
        .filter(|(module, _, child)| module.starts_with("ceiling_light") && !child)
        .map(|(_, transform, _)| transform.translation.xz())
        .collect();
    let openings: Vec<Vec2> = floor
        .openings
        .values()
        .flatten()
        .map(|(position, _)| *position)
        .collect();
    let hangers: Vec<Vec2> = floor
        .props
        .iter()
        .filter(|(module, ..)| module == "sign_hanger")
        .map(|(_, transform, _)| transform.translation.xz())
        .collect();
    assert_eq!(hangers.len(), 9);
    for hanger in hangers {
        for light in &lights {
            assert!(
                hanger.distance(*light) >= 1.2,
                "hanger {hanger} is near light {light}"
            );
        }
        for opening in &openings {
            assert!(
                hanger.distance(*opening) >= 1.0,
                "hanger {hanger} is near opening {opening}"
            );
        }
    }
}

#[test]
fn wall_signs_sit_on_walls_and_face_into_their_rooms() {
    let floor = floor();
    let mut checked = 0;
    for arrow in floor.arrows() {
        let (position, _) = reader(arrow);
        let behind = arrow.translation.xz() - (position - arrow.translation.xz()) * 0.3;
        let room = floor.room_at(position);
        let rect = floor.rooms.iter().find(|(name, _)| name == room).unwrap().1;
        let on_hanger = floor.props.iter().any(|(module, transform, _)| {
            module == "sign_hanger"
                && transform.translation.xz().distance(arrow.translation.xz()) < 0.8
        });
        if on_hanger {
            continue;
        }
        let edge = (arrow.translation.x - rect.min.x)
            .abs()
            .min((arrow.translation.x - rect.max.x).abs())
            .min((arrow.translation.z - rect.min.y).abs())
            .min((arrow.translation.z - rect.max.y).abs());
        assert!(
            edge < 0.2,
            "wall sign at {} is off the wall",
            arrow.translation
        );
        assert!(
            !rect.contains(behind),
            "wall sign at {} faces outward",
            arrow.translation
        );
        checked += 1;
    }
    assert_eq!(checked, 8);
}
