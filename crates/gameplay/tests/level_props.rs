use std::{
    collections::HashMap,
    f32::consts::{FRAC_PI_2, PI},
};

use bevy::prelude::*;
use gameplay::levels::{
    build_first_floor, Door, LightEffect, LightIntensity, Passage, Prop, PropLightsPlugin, Room,
};

#[test]
fn room_props_face_inward_and_tabletop_dressing_rests_on_surfaces() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let placements: Vec<_> = world
        .query::<(&Prop, &Transform)>()
        .iter(world)
        .map(|(prop, transform)| (prop.0.clone(), *transform))
        .collect();
    let at = |name: &str, x: f32, z: f32| {
        placements
            .iter()
            .find(|(module, transform)| {
                module == name
                    && (transform.translation.x - x).abs() < 0.001
                    && (transform.translation.z - z).abs() < 0.001
            })
            .unwrap_or_else(|| panic!("missing {name} at ({x}, {z})"))
            .1
    };

    let cabinet = at("file_cabinet_open", 13.4, -26.25);
    assert!(cabinet.translation.x + 0.24 < 13.75);
    assert!((cabinet.rotation * Vec3::NEG_Z).distance(Vec3::NEG_X) < 0.001);
    let crate_open = at("storage_crate_open", 10.0, -22.5);
    assert!((crate_open.rotation * Vec3::NEG_Z).distance(Vec3::Z) < 0.001);
    for x in [-5.7, 5.7] {
        let plant = at("potted_plant", x, 18.25);
        assert!(plant.translation.z + 0.28 < 18.75);
    }

    for (x, z) in [
        (8.9, -0.1),
        (-7.5, -22.9),
        (6.8, -25.7),
        (-2.1, -10.18),
        (4.05, -18.75),
        (-7.5, 13.6),
        (-1.7, 12.5),
    ] {
        let paper = at("clutter_papers", x, z);
        assert!((paper.translation.y - 0.802).abs() < 0.001);
        assert_eq!(paper.scale, Vec3::splat(0.5));
    }
    let office_papers = at("clutter_papers", 4.05, -18.75);
    let office_table = at("concept_table", 3.75, -18.75);
    assert!((office_papers.translation.x - office_table.translation.x).abs() + 0.3 < 0.9);
    let office_bins = at("shelf_unit_bins", 8.1, -17.5);
    assert!(office_bins.translation.x + 0.275 < 8.625 - 0.2);
    let storage_bins = at("shelf_unit_bins", 10.75, -14.8);
    assert!(storage_bins.translation.z - 0.925 > -16.125 + 0.25);
    let prep_papers = at("clutter_papers", 7.0, -7.65);
    assert!((prep_papers.translation.y - 0.902).abs() < 0.001);
    let prep_tools = at("clutter_tools", 7.0, -7.12);
    assert!((prep_tools.translation.y - 0.9).abs() < 0.001);
    assert_eq!(prep_tools.scale, Vec3::splat(0.65));
    assert!(prep_tools.translation.z - 0.65 * 0.303 > prep_papers.translation.z + 0.25);
    assert!(prep_tools.translation.z + 0.65 * 0.303 < -6.78);
    assert!(prep_tools.translation.x - 0.65 * 0.443 > 6.65);
    assert!(prep_tools.translation.x + 0.65 * 0.443 < 7.35);
    let other_tools = at("clutter_tools", 13.0, 0.1);
    assert!((other_tools.translation.y - 0.9).abs() < 0.001);
}

#[test]
fn moved_shelves_clear_walls_furniture_and_the_workshop_door() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut props = world.query::<(&Prop, &Transform)>();
    let shelves: Vec<_> = props
        .iter(world)
        .filter(|(prop, _)| prop.0 == "shelf_unit" || prop.0 == "shelf_unit_bins")
        .map(|(prop, transform)| (prop.0.clone(), *transform))
        .collect();
    let find = |module: &str, x: f32| {
        shelves
            .iter()
            .find(|(name, transform)| name == module && transform.translation.x == x)
            .unwrap_or_else(|| panic!("missing marked {module} at x={x}"))
            .1
    };

    let east = find("shelf_unit", 9.0);
    assert!((east.translation.z - 4.25).abs() < 0.001);
    assert!(east.translation.z - 0.275 > 3.75 + 0.1);

    let workshop = find("shelf_unit_bins", -12.0);
    assert!((workshop.translation.z - 9.5).abs() < 0.001);
    assert!((workshop.rotation * Vec3::Z).distance(Vec3::X) < 0.001);
    assert!(workshop.translation.z + 0.925 < 11.25 - 0.1);
    assert!(workshop.translation.x - 0.275 > -13.35 + 0.4881 + 0.1);
    assert!(workshop.translation.x + 0.275 < -10.0 - 1.0);
    assert!(workshop.translation.z - 0.925 > 6.0 + 0.6);

    let archive_workbench = props
        .iter(world)
        .find(|(prop, transform)| {
            prop.0 == "workbench" && transform.translation.x == -13.0
                && transform.translation.z == 13.75
        })
        .expect("archive workbench")
        .1;
    assert!(workshop.translation.z + 0.925 < archive_workbench.translation.z - 1.0);
}

#[test]
fn rear_hall_shelves_clear_the_back_wall() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut props = world.query::<(&Prop, &Transform)>();
    let shelves: Vec<_> = props
        .iter(world)
        .filter(|(prop, transform)| prop.0 == "shelf_unit_low" && transform.translation.z > 16.25)
        .map(|(_, transform)| transform.translation)
        .collect();
    assert_eq!(shelves.len(), 2);
    for shelf in shelves {
        assert!((shelf.x.abs() - 7.5).abs() < 0.001);
        assert!((shelf.z - 18.38).abs() < 0.001);
        assert!(shelf.z + 0.225 <= 18.75 - 0.1);
    }
}

#[test]
fn utility_shelves_clear_the_floor_grille_and_doorway() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut props = world.query::<(&Prop, &Transform)>();
    let small = props
        .iter(world)
        .find(|(prop, transform)| prop.0 == "shelf_unit_low" && transform.translation.x == -13.1)
        .expect("utility small shelf")
        .1
        .to_owned();
    let large = props
        .iter(world)
        .find(|(prop, transform)| prop.0 == "shelf_unit_bins" && transform.translation.x == -9.5)
        .expect("utility large shelf")
        .1
        .to_owned();
    let mut grilles = world.query::<(&Prop, &Transform, &ChildOf)>();
    let (grille, parent) = grilles
        .iter(world)
        .find(|(prop, local, parent)| {
            prop.0 == "vent_grille"
                && local.translation.y == -0.55
                && world
                    .get::<Prop>(parent.parent())
                    .is_some_and(|base| base.0 == "concept_crawl_vent")
        })
        .map(|(_, local, parent)| (local, parent.parent()))
        .expect("utility floor grille");
    let grille_center = world
        .get::<Transform>(parent)
        .unwrap()
        .transform_point(grille.translation);

    assert!((small.translation.z + 9.4).abs() < 0.001);
    assert!(small.translation.z + 0.6 < grille_center.z - 0.425);
    assert!((large.translation - Vec3::new(-9.5, 0.0, -7.0)).length() < 0.001);
    assert!(large.translation.z + 0.275 < -6.375 - 0.25);
    assert!(large.translation.x + 0.9 < -6.25 - 1.0);
    assert!((large.rotation * Vec3::Z).distance(Vec3::NEG_Z) < 0.001);
}

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
    assert_eq!(actual.get("potted_plant"), Some(&8));
    assert_eq!(actual.get("file_cabinet_open"), Some(&1));
    assert_eq!(actual.get("shelf_unit_alt"), Some(&4));
    assert_eq!(actual.get("storage_crate_open"), Some(&1));
    assert_eq!(actual.get("shelf_bin_contents"), Some(&6));
    assert_eq!(actual.get("toolbox"), Some(&1));
    let mut attached = world.query::<(&Prop, &ChildOf)>();
    for (decoration, expected) in [
        ("toolbox", "concept_table"),
        ("shelf_bin_contents", "shelf_unit_bins"),
    ] {
        let count = attached
            .iter(world)
            .filter(|(prop, parent)| {
                prop.0 == decoration
                    && world
                        .get::<Prop>(parent.parent())
                        .is_some_and(|base| base.0 == expected)
            })
            .count();
        assert_eq!(count, if decoration == "toolbox" { 1 } else { 6 });
    }

    let mut vent_parts = world.query::<(&Prop, &Transform, &ChildOf)>();
    let grille_locations: Vec<_> = vent_parts
        .iter(world)
        .filter(|(part, _, parent)| {
            part.0 == "vent_grille"
                && world
                    .get::<Prop>(parent.parent())
                    .is_some_and(|base| base.0 == "concept_crawl_vent")
        })
        .map(|(_, transform, _)| (transform.translation, transform.rotation * Vec3::Y))
        .collect();
    assert_eq!(grille_locations.len(), 3);
    let rear_vent = vent_parts
        .iter(world)
        .filter(|(prop, local, parent)| {
            prop.0 == "vent_grille"
                && local.translation.y == 0.0
                && world
                    .get::<Transform>(parent.parent())
                    .is_some_and(|vent| vent.translation == Vec3::new(-13.65, 0.55, 17.0))
        })
        .count();
    assert_eq!(rear_vent, 0);
    let floor_grille = world
        .query::<(&Prop, &Transform, Option<&ChildOf>)>()
        .iter(world)
        .filter(|(prop, transform, parent)| {
            prop.0 == "vent_grille"
                && parent.is_none()
                && transform.translation == Vec3::new(-12.6, 0.0, 17.0)
        })
        .count();
    assert_eq!(floor_grille, 1);
    assert_eq!(
        grille_locations
            .iter()
            .filter(|&&(position, normal)| position.y == -0.55
                && position.x.abs() <= 0.15
                && (-0.83..=-0.67).contains(&position.z)
                && normal == Vec3::Y)
            .count(),
        1
    );
    assert_eq!(
        grille_locations
            .iter()
            .filter(|&&(position, normal)| {
                position.y == 0.0
                    && (position.z + 0.085).abs() < 0.001
                    && normal.distance(-Vec3::Z) < 0.001
            })
            .count(),
        2
    );

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
        ("shelf_unit_bins", 8.1, -17.5, Vec3::NEG_X),
        ("shelf_unit_bins", -9.5, -7.0, Vec3::NEG_Z),
        ("shelf_unit_bins", 10.75, -14.8, Vec3::NEG_X),
        ("shelf_unit_bins", -12.0, 9.5, Vec3::X),
        ("shelf_unit_bins", 7.1, 13.75, Vec3::X),
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

    for (module, z, yaw) in [
        ("shelf_unit_alt", -12.5, FRAC_PI_2),
        ("shelf_unit_bins", -14.8, -FRAC_PI_2),
    ] {
        assert!(placements.iter().any(|(name, transform)| {
            name == module
                && transform.translation == Vec3::new(10.75, 0.0, z)
                && transform.rotation == Quat::from_rotation_y(yaw)
        }));
    }

    let archive_bins = placements
        .iter()
        .find(|(module, transform)| {
            module == "shelf_unit_bins" && transform.translation == Vec3::new(7.1, 0.0, 13.75)
        })
        .expect("back-right bin shelf")
        .1;
    assert!(archive_bins.translation.x - 0.25 > 6.25 + 0.5);

    let west_wing_shelf = placements
        .iter()
        .find(|(module, transform)| {
            module == "shelf_unit" && transform.translation == Vec3::new(-12.35, 0.0, 15.65)
        })
        .expect("west wing shelf")
        .1;
    assert!(west_wing_shelf.translation.x - 0.9 > -13.75 + 0.4);
    assert!(west_wing_shelf.translation.z + 0.25 < 16.25 - 0.3);

    let maintenance_shelf = placements
        .iter()
        .find(|(module, transform)| {
            module == "shelf_unit_alt" && transform.translation == Vec3::new(-10.8, 0.0, -17.55)
        })
        .expect("maintenance shelf")
        .1;
    assert!(maintenance_shelf.translation.z + 0.875 < -16.25 - 0.25);
    assert!(maintenance_shelf.translation.x - 0.25 > -12.5 + 1.0);

    let prep_bench = placements
        .iter()
        .find(|(module, transform)| {
            module == "workbench" && transform.translation == Vec3::new(7.0, 0.0, -7.3)
        })
        .expect("prep workbench")
        .1;
    assert!((prep_bench.rotation * Vec3::NEG_Z).x > 0.99);

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
        ("concept_crawl_vent", Vec3::new(-13.65, 0.55, 17.0)),
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
        4
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
                    Vec2::new(8.1, -17.5),
                    Vec2::new(0.25, 0.9),
                ),
            ][..],
        ),
        (
            "prep",
            Vec2::new(7.0, -7.3),
            -FRAC_PI_2,
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
        ("east_link", "shelf_unit_alt", 2.5, -8.35, 0.9, 0.25),
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
