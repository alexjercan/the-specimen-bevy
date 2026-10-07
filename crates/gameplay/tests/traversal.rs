use std::collections::{BTreeSet, HashMap, VecDeque};

use bevy::prelude::*;
use gameplay::{
    facility::{
        grid::{cell_of, edge_key, Side},
        Boundary, DoorState, Doorway, FacilityPlugin, FloorCell, Passage, PlayerStart, Room,
    },
    levels::first_floor,
};

fn headless() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FacilityPlugin::new(first_floor()));
    app.finish();
    app.update();
    app
}

fn start(app: &mut App) -> IVec2 {
    let world = app.world_mut();
    let start = world
        .query_filtered::<&Transform, With<PlayerStart>>()
        .single(world)
        .unwrap();
    cell_of(start.translation)
}

fn set_door(app: &mut App, cell: IVec2, side: Side, state: DoorState) {
    let world = app.world_mut();
    let key = edge_key(cell, side);
    let door = world
        .query_filtered::<(Entity, &Boundary), With<Doorway>>()
        .iter(world)
        .find(|(_, boundary)| boundary.key == key)
        .map(|(entity, _)| entity)
        .unwrap();
    *world.get_mut::<DoorState>(door).unwrap() = state;
}

struct Walk {
    rooms: HashMap<IVec2, &'static str>,
    open: HashMap<IVec2, bool>,
}

impl Walk {
    fn new(app: &mut App) -> Self {
        let world = app.world_mut();
        let rooms = world
            .query::<(&FloorCell, &ChildOf)>()
            .iter(world)
            .map(|(cell, parent)| (cell.cell, world.get::<Room>(parent.parent()).unwrap().name))
            .collect();
        let open = world
            .query::<(&Boundary, Has<Passage>, Option<&DoorState>)>()
            .iter(world)
            .map(|(boundary, passage, door)| {
                (boundary.key, passage || door.is_some_and(|s| s.passable()))
            })
            .collect();
        Self { rooms, open }
    }

    fn moves(&self, cell: IVec2) -> Vec<(Side, Option<IVec2>)> {
        Side::ALL
            .into_iter()
            .filter_map(|side| {
                let next = cell + side.offset();
                let inside = self.rooms.contains_key(&next).then_some(next);
                match self.open.get(&edge_key(cell, side)) {
                    None => inside.map(|next| (side, Some(next))),
                    Some(true) => Some((side, inside)),
                    Some(false) => None,
                }
            })
            .collect()
    }

    fn from(&self, start: IVec2) -> (BTreeSet<&'static str>, usize, bool) {
        let mut seen = BTreeSet::from([(start.x, start.y)]);
        let mut queue = VecDeque::from([start]);
        let mut outside = false;
        while let Some(cell) = queue.pop_front() {
            for (_, next) in self.moves(cell) {
                match next {
                    None => outside = true,
                    Some(next) => {
                        if seen.insert((next.x, next.y)) {
                            queue.push_back(next);
                        }
                    }
                }
            }
        }
        let rooms = seen
            .iter()
            .map(|&(x, z)| self.rooms[&IVec2::new(x, z)])
            .collect();
        (rooms, seen.len(), outside)
    }
}

#[test]
fn locked_exit_keeps_the_player_inside_every_room() {
    let mut app = headless();
    let from = start(&mut app);
    let walk = Walk::new(&mut app);
    let (rooms, cells, outside) = walk.from(from);
    assert!(!outside, "player can leave through the locked EXIT");
    assert_eq!(cells, walk.rooms.len());
    assert_eq!(rooms.len(), 16);
}

#[test]
fn closed_and_locked_doors_block_but_open_doors_pass() {
    let mut app = headless();
    let from = start(&mut app);
    for (state, outside) in [
        (DoorState::Closed, false),
        (DoorState::Locked, false),
        (DoorState::Open, true),
    ] {
        set_door(&mut app, IVec2::new(0, -12), Side::North, state);
        assert_eq!(Walk::new(&mut app).from(from).2, outside, "{state:?}");
    }
}

#[test]
fn closing_the_only_boiler_door_seals_the_boiler_room() {
    let mut app = headless();
    let from = start(&mut app);
    assert!(Walk::new(&mut app).from(from).0.contains("boiler"));
    set_door(&mut app, IVec2::new(-2, 0), Side::West, DoorState::Closed);
    let (rooms, ..) = Walk::new(&mut app).from(from);
    assert!(!rooms.contains("boiler"));
    assert_eq!(rooms.len(), 15);
}

#[test]
fn maintenance_stays_reachable_through_the_service_bypass() {
    let mut app = headless();
    let from = start(&mut app);
    set_door(&mut app, IVec2::new(-2, -7), Side::South, DoorState::Closed);
    assert!(Walk::new(&mut app).from(from).0.contains("maintenance"));
    set_door(&mut app, IVec2::new(0, -8), Side::West, DoorState::Closed);
    assert!(!Walk::new(&mut app).from(from).0.contains("maintenance"));
}

#[test]
fn walls_never_appear_as_walkable_moves() {
    let mut app = headless();
    let moves: Vec<_> = Walk::new(&mut app)
        .moves(IVec2::new(-5, 1))
        .into_iter()
        .map(|(side, _)| side)
        .collect();
    assert_eq!(moves, [Side::North, Side::East]);
}
