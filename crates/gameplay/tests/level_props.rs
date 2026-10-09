use std::{
    collections::HashMap,
    f32::consts::{FRAC_PI_2, PI},
};

use bevy::prelude::*;
use gameplay::levels::{
    build_first_floor, Door, LightEffect, LightIntensity, Passage, Prop, PropLightsPlugin, Room,
};

#[test]
fn first_floor_props_and_lights_are_linked() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PropLightsPlugin))
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut actual = HashMap::<String, usize>::new();
    let mut props = world.query::<&Prop>();
    for prop in props.iter(world) {
        *actual.entry(prop.0.clone()).or_default() += 1;
    }
    assert_eq!(actual.get("sign_hanger"), Some(&9));
    assert_eq!(actual.get("boiler_unit"), Some(&1));
    assert_eq!(actual.get("concept_containment_tank"), Some(&1));

    let mut light_query = world.query::<(Entity, &ChildOf, &PointLight, &LightIntensity)>();
    let lights: Vec<_> = light_query
        .iter(world)
        .map(|(entity, parent, light, base)| {
            assert!(world.get::<Prop>(parent.parent()).is_some());
            assert!(!light.shadow_maps_enabled);
            assert_eq!(light.intensity, base.0);
            entity
        })
        .collect();
    assert_eq!(lights.len(), 27);
    assert_eq!(
        lights
            .iter()
            .filter(|&&e| world.get::<LightEffect>(e).is_some())
            .count(),
        12
    );
    assert!(world.query::<&PointLight>().iter(world).count() == lights.len());
}

#[test]
fn wall_plate_and_tall_storage_face_into_their_rooms() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut props = world.query::<(&Prop, &Transform)>();
    let placements: Vec<_> = props
        .iter(world)
        .map(|(prop, transform)| (prop.0.clone(), *transform))
        .collect();
    for (module, position, facing) in [
        (
            "sign_label_boiler_room",
            Vec3::new(-6.10, 2.6, 0.0),
            Vec3::X,
        ),
        ("sign_label_utility", Vec3::new(-6.15, 2.6, -10.0), Vec3::X),
        ("sign_label_lab", Vec3::new(3.85, 2.6, 0.0), Vec3::X),
        (
            "sign_label_security",
            Vec3::new(8.65, 2.6, -27.5),
            Vec3::NEG_X,
        ),
        ("sign_label_prep", Vec3::new(6.15, 2.6, -5.0), Vec3::NEG_X),
    ] {
        let sign = placements
            .iter()
            .find(|(name, transform)| name == module && transform.translation == position)
            .unwrap_or_else(|| panic!("missing {module}"))
            .1;
        assert!((sign.rotation * Vec3::NEG_Z).distance(facing) < 0.001);
    }

    let pegboard = placements
        .iter()
        .find(|(module, transform)| {
            module == "tool_pegboard" && transform.translation == Vec3::new(-13.64, 1.6, 2.3)
        })
        .unwrap()
        .1;
    assert!((pegboard.rotation * Vec3::NEG_Z).distance(Vec3::X) < 0.001);
    assert!((pegboard.translation.x - -13.65).abs() < 0.02);

    for (module, x, z, inward) in [
        ("concept_locker", 10.5, -24.15, Vec3::NEG_Z),
        ("concept_locker", 9.5, -9.15, Vec3::NEG_Z),
        ("concept_locker", 13.35, -2.2, Vec3::NEG_X),
        ("concept_locker", 13.35, 2.2, Vec3::NEG_X),
        ("concept_locker", 13.35, -12.5, Vec3::NEG_X),
        ("concept_locker", -13.35, -18.75, Vec3::X),
        ("shelf_unit", 13.35, -18.75, Vec3::NEG_X),
        ("shelf_unit_bins", 8.4, -17.5, Vec3::NEG_X),
        ("shelf_unit_bins", -13.35, -10.0, Vec3::X),
    ] {
        let transform = placements
            .iter()
            .find(|(name, transform)| {
                name == module && transform.translation.x == x && transform.translation.z == z
            })
            .unwrap_or_else(|| panic!("missing {module} at ({x}, {z})"))
            .1;
        let front = if module == "shelf_unit_bins" {
            Vec3::Z
        } else {
            Vec3::NEG_Z
        };
        assert!((transform.rotation * front).distance(inward) < 0.001);
    }

    for (module, z) in [("shelf_unit", -12.5), ("shelf_unit_bins", -15.0)] {
        assert!(placements.iter().any(|(name, transform)| {
            name == module
                && transform.translation == Vec3::new(10.75, 0.0, z)
                && transform.rotation == Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
        }));
    }

    let mut rooms = world.query::<(&Name, &Room)>();
    for name in ["exit", "security", "office", "utility", "hiding", "boiler"] {
        let (_, room) = rooms
            .iter(world)
            .find(|(room_name, _)| room_name.as_str() == name)
            .unwrap();
        let added = placements.iter().filter(|(module, transform)| {
            matches!(
                module.as_str(),
                "storage_crate" | "steel_drum" | "concept_table" | "shelf_unit_low"
            ) && room
                .0
                .contains(Vec2::new(transform.translation.x, transform.translation.z))
        });
        assert!(added.count() > 0, "{name} has no detail props");
    }
}

#[test]
fn prep_and_new_crawl_vents_preserve_room_routes() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut props = world.query::<(&Prop, &Transform)>();
    let placements: Vec<_> = props
        .iter(world)
        .map(|(prop, transform)| (prop.0.clone(), *transform))
        .collect();
    for (module, position) in [
        ("shelf_unit_low", Vec3::new(13.15, 0.0, -6.25)),
        ("workbench", Vec3::new(7.0, 0.0, -7.3)),
        ("storage_crate", Vec3::new(12.0, 0.0, -8.0)),
        ("concept_locker", Vec3::new(13.35, 0.0, -12.5)),
        ("concept_crawl_vent", Vec3::new(-13.65, 0.55, -8.0)),
        ("concept_crawl_vent", Vec3::new(13.65, 0.55, -4.5)),
    ] {
        assert!(
            placements
                .iter()
                .any(|(name, transform)| { name == module && transform.translation == position }),
            "missing {module} at {position}"
        );
    }
    assert_eq!(
        placements
            .iter()
            .filter(|(name, _)| name == "concept_crawl_vent")
            .count(),
        3
    );

    let mut rooms = world.query::<(&Name, &Room)>();
    let prep = rooms
        .iter(world)
        .find(|(name, _)| name.as_str() == "prep")
        .unwrap()
        .1
         .0;
    for (x, z, half_x, half_z) in [
        (13.15, -6.25, 0.225, 0.6),
        (7.0, -7.3, 0.35, 0.8),
        (12.0, -8.0, 0.5, 0.4),
    ] {
        assert!(prep.contains(Vec2::new(x - half_x, z - half_z)));
        assert!(prep.contains(Vec2::new(x + half_x, z + half_z)));
        assert!(Vec2::new(x, z).distance(Vec2::new(6.25, -5.0)) > 1.5);
        assert!(Vec2::new(x, z).distance(Vec2::new(10.0, -3.75)) > 1.5);
    }
}

#[test]
fn first_floor_workbenches_face_open_space_from_walls() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let rooms: HashMap<_, _> = world
        .query::<(&Name, &Room)>()
        .iter(world)
        .map(|(name, room)| (name.as_str().to_owned(), room.0))
        .collect();
    let props: Vec<_> = world
        .query::<(&Prop, &Transform)>()
        .iter(world)
        .map(|(prop, transform)| (prop.0.clone(), *transform))
        .collect();
    let mut openings: Vec<_> = world
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

    for (room, position, yaw, wall, neighbors) in [
        (
            "maintenance",
            Vec2::new(-13.0, -21.5),
            -FRAC_PI_2,
            Vec2::new(-13.75, -21.5),
            &[][..],
        ),
        (
            "security",
            Vec2::new(10.0, -30.65),
            PI,
            Vec2::new(10.0, -31.25),
            &[][..],
        ),
        (
            "hiding",
            Vec2::new(13.0, 0.0),
            FRAC_PI_2,
            Vec2::new(13.75, 0.0),
            &[][..],
        ),
        (
            "utility",
            Vec2::new(-10.0, -15.5),
            PI,
            Vec2::new(-10.0, -16.25),
            &[][..],
        ),
        (
            "office",
            Vec2::new(3.0, -20.7),
            PI,
            Vec2::new(3.0, -21.25),
            &[
                (
                    "concept_table",
                    Vec2::new(3.75, -18.75),
                    Vec2::new(0.9, 0.46),
                ),
                (
                    "shelf_unit_bins",
                    Vec2::new(8.4, -17.5),
                    Vec2::new(0.25, 0.9),
                ),
            ][..],
        ),
        (
            "prep",
            Vec2::new(7.0, -7.3),
            FRAC_PI_2,
            Vec2::new(6.25, -7.3),
            &[
                (
                    "shelf_unit_low",
                    Vec2::new(13.15, -6.25),
                    Vec2::new(0.225, 0.6),
                ),
                ("storage_crate", Vec2::new(12.0, -8.0), Vec2::new(0.5, 0.4)),
            ][..],
        ),
        (
            "workshop",
            Vec2::new(-13.0, 6.0),
            -FRAC_PI_2,
            Vec2::new(-13.75, 6.0),
            &[][..],
        ),
        (
            "archive",
            Vec2::new(-13.0, 13.75),
            -FRAC_PI_2,
            Vec2::new(-13.75, 13.75),
            &[][..],
        ),
        (
            "assembly",
            Vec2::new(0.0, 15.7),
            0.0,
            Vec2::new(0.0, 16.25),
            &[][..],
        ),
        (
            "fabrication",
            Vec2::new(13.0, 6.0),
            FRAC_PI_2,
            Vec2::new(13.75, 6.0),
            &[][..],
        ),
        (
            "sample_store",
            Vec2::new(13.0, 13.75),
            FRAC_PI_2,
            Vec2::new(13.75, 13.75),
            &[][..],
        ),
    ] {
        let transform = props
            .iter()
            .find(|(module, transform)| {
                module == "workbench" && transform.translation.xz() == position
            })
            .unwrap_or_else(|| panic!("missing {room} workbench"))
            .1;
        assert!(transform.rotation.dot(Quat::from_rotation_y(yaw)).abs() > 0.9999);
        let horizontal = yaw == 0.0 || yaw.abs() == PI;
        let half = if horizontal {
            Vec2::new(0.8, 0.35)
        } else {
            Vec2::new(0.35, 0.8)
        };
        let bounds = rooms[room];
        assert!(bounds.contains(position - half));
        assert!(bounds.contains(position + half));
        let wall_gap = if horizontal {
            (position.y - wall.y).abs() - half.y
        } else {
            (position.x - wall.x).abs() - half.x
        };
        assert!(wall_gap <= 0.5, "{room} bench is not wall-mounted");
        assert!(
            openings.iter().all(|door| {
                let delta = (*door - position).abs() - half;
                delta.max(Vec2::ZERO).length() >= 1.0
            }),
            "{room} bench blocks an opening"
        );
        for &(module, other, other_half) in neighbors {
            assert!(props.iter().any(|(prop, transform)| {
                prop == module && transform.translation.xz() == other
            }));
            let separation = (other - position).abs() - half - other_half;
            assert!(
                separation.max_element() >= 0.5,
                "{room} bench crowds {module}"
            );
        }
    }
}

#[test]
fn reception_and_lab_links_have_furniture_away_from_openings() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut rooms = world.query::<(&Name, &Room)>();
    let bounds: HashMap<_, _> = rooms
        .iter(world)
        .map(|(name, room)| (name.as_str().to_owned(), room.0))
        .collect();
    let mut props = world.query::<(&Prop, &Transform)>();
    for (room, module, x, z, half_width, half_depth) in [
        ("reception", "shelf_unit_low", -2.5, -15.85, 0.6, 0.225),
        ("reception", "shelf_unit_low", 2.5, -15.85, 0.6, 0.225),
        ("reception", "chair_tipped", -2.3, -13.5, 0.32, 0.42),
        ("reception", "chair_tipped", 2.3, -13.5, 0.32, 0.42),
        ("west_link", "shelf_unit_bins", -2.5, -8.35, 0.9, 0.25),
        ("east_link", "shelf_unit", 2.5, -8.35, 0.9, 0.25),
    ] {
        let rect = bounds[room];
        let placed = props.iter(world).any(|(prop, transform)| {
            prop.0 == module && transform.translation == Vec3::new(x, 0.0, z)
        });
        assert!(placed, "missing {module} in {room}");
        assert!(rect.contains(Vec2::new(x - half_width, z - half_depth)));
        assert!(rect.contains(Vec2::new(x + half_width, z + half_depth)));
        if room == "reception" && module == "shelf_unit_low" {
            assert!(x.abs() - half_width >= 0.9);
        }
        if room.ends_with("_link") {
            assert!(-5.0 - (z + half_depth) >= 1.2);
        }
    }
}

#[test]
fn rendered_props_have_visible_parents() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut children = world.query::<(&ChildOf, &Prop)>();
    let mut checked = 0;
    let mut sign_children = 0;
    for (parent, _) in children.iter(world) {
        let parent = parent.parent();
        assert!(world.get::<Visibility>(parent).is_some());
        assert!(world.get::<InheritedVisibility>(parent).is_some());
        if world.get::<Prop>(parent).is_none() {
            sign_children += 1;
        }
        checked += 1;
    }
    assert!(checked > 0);
    assert_eq!(sign_children, 18);
}

#[test]
fn signs_group_labels_and_arrows_under_their_mounts() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut hangers = world.query::<(&Prop, &Children)>();
    let mut count = 0;
    for (prop, children) in hangers.iter(world) {
        if prop.0 != "sign_hanger" {
            continue;
        }
        count += 1;
        let labels = children
            .iter()
            .filter(|&child| {
                world
                    .get::<Prop>(child)
                    .is_some_and(|p| p.0.starts_with("sign_label_"))
            })
            .count();
        let arrows = children
            .iter()
            .filter(|&child| {
                world
                    .get::<Prop>(child)
                    .is_some_and(|p| p.0 == "sign_arrow")
            })
            .count();
        assert!(labels == 1 || labels == 2);
        assert_eq!(labels, arrows);
        assert_eq!(children.len(), labels + arrows);
    }
    assert_eq!(count, 9);
}
