use std::collections::BTreeSet;

use bevy::prelude::*;
use gameplay::facility::{
    layout::{authored, derive, Owner},
    Boundary, Decor, DoorState, Doorway, Facility, FacilityGrid, FacilityLayout, FacilityPart,
    FacilityPlugin, FacilityProp, FloorCell, Objective, Passage, PlayerStart, Room, Wall, WallPost,
};

fn headless() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(FacilityPlugin);
    app.update();
    app
}

fn count<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, F>()
        .iter(app.world())
        .count()
}

#[test]
fn logical_facility_spawns_without_assets_or_rendering() {
    let mut app = headless();
    let layout = authored();
    let plan = derive(&layout).unwrap();
    assert!(!app.world().contains_resource::<AssetServer>());
    assert_eq!(count::<With<Facility>>(&mut app), 1);
    assert_eq!(count::<With<Room>>(&mut app), 16);
    assert_eq!(count::<With<FloorCell>>(&mut app), plan.cells.len());
    assert_eq!(count::<With<Boundary>>(&mut app), plan.edges.len());
    assert_eq!(count::<With<Doorway>>(&mut app), 15);
    assert_eq!(count::<With<Passage>>(&mut app), 8);
    assert_eq!(count::<With<Wall>>(&mut app), plan.edges.len() - 15 - 8);
    assert_eq!(count::<With<WallPost>>(&mut app), plan.posts.len());
    assert_eq!(count::<With<FacilityProp>>(&mut app), layout.props.len());
    assert_eq!(count::<With<Decor>>(&mut app), layout.fixtures.len());
    assert_eq!(count::<With<Visibility>>(&mut app), 0);
}

#[test]
fn logical_parts_have_unique_owners_and_authored_anchors() {
    let app = headless();
    let world = app.world();
    let mut layouts = world
        .try_query_filtered::<&FacilityLayout, With<Facility>>()
        .unwrap();
    let facility = layouts.single(world).unwrap();
    let mut parts = world.try_query::<(&FacilityPart, &Transform)>().unwrap();
    let anchors: Vec<_> = parts
        .iter(world)
        .map(|(part, transform)| (part.0, *transform, facility.anchor(part.0)))
        .collect();
    let owners: BTreeSet<Owner> = anchors.iter().map(|(owner, ..)| *owner).collect();
    assert_eq!(owners.len(), anchors.len(), "duplicate logical owner");
    for owner in facility.plan.pieces.iter().map(|p| p.owner) {
        assert!(owners.contains(&owner), "{owner:?} has no logical entity");
    }
    for (owner, transform, anchor) in anchors {
        assert_eq!(transform, anchor, "{owner:?}");
    }
}

#[test]
fn objectives_and_player_start_are_marked() {
    let mut app = headless();
    let world = app.world_mut();
    let exits: Vec<_> = world
        .query::<(&Objective, &DoorState, &Boundary)>()
        .iter(world)
        .map(|(objective, state, boundary)| (*objective, *state, boundary.key))
        .collect();
    assert_eq!(exits, [(Objective::Exit, DoorState::Locked, (0, -25))]);
    let doors: Vec<_> = world.query::<&DoorState>().iter(world).copied().collect();
    assert_eq!(doors.iter().filter(|s| **s == DoorState::Open).count(), 14);
    let panels: Vec<_> = world
        .query_filtered::<(&Objective, &ChildOf), With<Decor>>()
        .iter(world)
        .map(|(objective, parent)| (*objective, parent.parent()))
        .collect();
    assert_eq!(panels.len(), 1);
    assert_eq!(panels[0].0, Objective::FusePanel);
    assert_eq!(world.get::<Room>(panels[0].1).unwrap().name, "exit");

    let (start, parent) = world
        .query_filtered::<(&Transform, &ChildOf), With<PlayerStart>>()
        .single(world)
        .unwrap();
    assert_eq!(start.translation, Vec3::new(0.0, 0.0, -5.0));
    assert_eq!(world.get::<Room>(parent.parent()).unwrap().name, "intake");
}

#[test]
fn grid_maps_cells_and_boundaries_to_logical_entities() {
    let mut app = headless();
    let world = app.world_mut();
    let (layout, grid) = world
        .query::<(&FacilityLayout, &FacilityGrid)>()
        .single(world)
        .unwrap();
    assert_eq!(grid.rooms.len(), layout.layout.areas.len());
    assert_eq!(grid.cells.len(), layout.plan.cells.len());
    assert_eq!(grid.boundaries.len(), layout.plan.edges.len());
    for (&key, &entity) in &grid.cells {
        assert_eq!(
            world.get::<FloorCell>(entity).unwrap().cell,
            IVec2::new(key.0, key.1)
        );
    }
    for (&key, &entity) in &grid.boundaries {
        assert_eq!(world.get::<Boundary>(entity).unwrap().key, key);
    }
}

#[test]
fn despawning_the_facility_removes_every_logical_entity() {
    let mut app = headless();
    let root = app
        .world_mut()
        .query_filtered::<Entity, With<Facility>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(root).despawn();
    assert_eq!(count::<With<FacilityPart>>(&mut app), 0);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(count::<With<PlayerStart>>(&mut app), 0);
}
