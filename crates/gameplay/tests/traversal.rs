use bevy::prelude::*;
use gameplay::facility::{
    layout::{edge_key, path, Dest, Side},
    DoorState, Facility, FacilityGrid, FacilityLayout, FacilityPlugin, Objective, PlayerStart,
};

fn headless() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(FacilityPlugin);
    app.update();
    app
}

fn start(app: &mut App) -> IVec2 {
    let world = app.world_mut();
    let start = world
        .query_filtered::<&Transform, With<PlayerStart>>()
        .single(world)
        .unwrap();
    IVec2::new(
        (start.translation.x / 2.5).round() as i32,
        (start.translation.z / 2.5).round() as i32,
    )
}

fn set_door(app: &mut App, cell: IVec2, side: Side, state: DoorState) {
    let world = app.world_mut();
    let grid = world
        .query_filtered::<&FacilityGrid, With<Facility>>()
        .single(world)
        .unwrap();
    let door = grid.boundaries[&edge_key(cell, side)];
    *world.get_mut::<DoorState>(door).unwrap() = state;
}

fn reach(app: &mut App, from: IVec2) -> gameplay::facility::layout::Reach {
    let world = app.world();
    let mut query = app
        .world()
        .try_query::<(&FacilityLayout, &FacilityGrid)>()
        .unwrap();
    let (layout, grid) = query.single(world).unwrap();
    grid.reachable(&layout.plan, from, |entity| {
        world.get::<DoorState>(entity).copied()
    })
}

fn rooms_reached(app: &mut App, from: IVec2) -> Vec<&'static str> {
    let reach = reach(app, from);
    let world = app.world();
    let mut query = world.try_query::<&FacilityLayout>().unwrap();
    let facility = query.single(world).unwrap();
    let mut names: Vec<_> = facility
        .layout
        .areas
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            facility
                .plan
                .cells
                .iter()
                .any(|(key, area)| area == index && reach.cells.contains(key))
        })
        .map(|(_, area)| area.name)
        .collect();
    names.sort();
    names
}

#[test]
fn locked_exit_blocks_walking_while_signs_still_route_to_it() {
    let mut app = headless();
    let from = start(&mut app);
    let reach = reach(&mut app, from);
    let world = app.world();
    let mut query = world.try_query::<&FacilityLayout>().unwrap();
    let facility = query.single(world).unwrap();
    assert!(!reach.outside, "player can leave through the locked EXIT");
    assert_eq!(reach.cells.len(), facility.plan.cells.len());
    let route = path(&facility.plan, &facility.layout, from, Dest::Exit).unwrap();
    let last = route.last().unwrap();
    assert_eq!(Some((last.cell, last.side)), facility.layout.exit);
}

#[test]
fn closed_and_locked_doors_block_but_open_doors_pass() {
    let mut app = headless();
    let from = start(&mut app);
    let (exit_cell, exit_side) = (IVec2::new(0, -12), Side::North);
    for (state, outside) in [
        (DoorState::Closed, false),
        (DoorState::Locked, false),
        (DoorState::Open, true),
    ] {
        set_door(&mut app, exit_cell, exit_side, state);
        assert_eq!(reach(&mut app, from).outside, outside, "{state:?}");
    }
    let world = app.world_mut();
    let objective = world
        .query::<(&Objective, &DoorState)>()
        .single(world)
        .unwrap();
    assert_eq!(
        (*objective.0, *objective.1),
        (Objective::Exit, DoorState::Open)
    );
}

#[test]
fn closing_the_only_boiler_door_seals_the_boiler_room() {
    let mut app = headless();
    let from = start(&mut app);
    assert!(rooms_reached(&mut app, from).contains(&"boiler"));
    set_door(&mut app, IVec2::new(-2, 0), Side::West, DoorState::Closed);
    let rooms = rooms_reached(&mut app, from);
    assert!(!rooms.contains(&"boiler"));
    assert_eq!(rooms.len(), 15);
}

#[test]
fn maintenance_stays_reachable_through_the_service_bypass() {
    let mut app = headless();
    let from = start(&mut app);
    set_door(&mut app, IVec2::new(-2, -7), Side::South, DoorState::Closed);
    assert!(rooms_reached(&mut app, from).contains(&"maintenance"));
    set_door(&mut app, IVec2::new(0, -8), Side::West, DoorState::Closed);
    assert!(!rooms_reached(&mut app, from).contains(&"maintenance"));
}

#[test]
fn walls_never_appear_as_walkable_moves() {
    let app = headless();
    let world = app.world();
    let mut query = world
        .try_query::<(&FacilityLayout, &FacilityGrid)>()
        .unwrap();
    let (facility, grid) = query.single(world).unwrap();
    let corner = IVec2::new(-5, 1);
    let moves: Vec<_> = grid
        .walkable_moves(&facility.plan, corner, |entity| {
            world.get::<DoorState>(entity).copied()
        })
        .into_iter()
        .map(|(side, _)| side)
        .collect();
    assert_eq!(moves, [Side::North, Side::East]);
}
