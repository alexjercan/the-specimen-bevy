pub mod layout;
pub mod render;

use std::collections::BTreeMap;

use bevy::prelude::*;

use layout::{
    authored, derive, Door, EdgeKind, Fixture, Key, Layout, Mount, Owner, Plan, PropKind, Reach,
};

pub const DEFAULT_DOOR_SWING: f32 = 75.0;

pub struct FacilityPlugin;

impl Plugin for FacilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_authored_facility);
    }
}

#[derive(Component, Debug)]
pub struct Facility;

#[derive(Component, Debug)]
pub struct FacilityLayout {
    pub layout: Layout,
    pub plan: Plan,
    pieces: BTreeMap<Owner, Vec<usize>>,
    lights: BTreeMap<Owner, Vec<usize>>,
}

impl FacilityLayout {
    pub fn new(layout: Layout, plan: Plan) -> Self {
        let mut pieces: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for (index, piece) in plan.pieces.iter().enumerate() {
            pieces.entry(piece.owner).or_default().push(index);
        }
        let mut lights: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for (index, light) in plan.lights.iter().enumerate() {
            lights.entry(light.owner).or_default().push(index);
        }
        Self {
            layout,
            plan,
            pieces,
            lights,
        }
    }

    pub fn pieces(&self, owner: Owner) -> impl Iterator<Item = &layout::Piece> {
        self.pieces
            .get(&owner)
            .into_iter()
            .flatten()
            .map(|&index| &self.plan.pieces[index])
    }

    pub fn lights(&self, owner: Owner) -> impl Iterator<Item = &layout::LightSpec> {
        self.lights
            .get(&owner)
            .into_iter()
            .flatten()
            .map(|&index| &self.plan.lights[index])
    }

    pub fn anchor(&self, owner: Owner) -> Transform {
        let (translation, yaw) = self.plan.anchor(&self.layout, owner);
        Transform::from_translation(translation).with_rotation(Quat::from_rotation_y(yaw))
    }
}

#[derive(Component, Debug, Default)]
pub struct FacilityGrid {
    pub rooms: Vec<Entity>,
    pub cells: BTreeMap<Key, Entity>,
    pub boundaries: BTreeMap<Key, Entity>,
}

impl FacilityGrid {
    pub fn door_open(&self, key: Key, state: impl Fn(Entity) -> Option<DoorState>) -> bool {
        self.boundaries
            .get(&key)
            .and_then(|&entity| state(entity))
            .is_some_and(DoorState::passable)
    }

    pub fn walkable_moves(
        &self,
        plan: &Plan,
        cell: IVec2,
        state: impl Fn(Entity) -> Option<DoorState>,
    ) -> Vec<(layout::Side, Option<IVec2>)> {
        layout::moves_through(plan, cell, |key| self.door_open(key, &state))
    }

    pub fn reachable(
        &self,
        plan: &Plan,
        from: IVec2,
        state: impl Fn(Entity) -> Option<DoorState>,
    ) -> Reach {
        layout::reachable(plan, from, |key| self.door_open(key, &state))
    }
}

#[derive(Component, Debug)]
pub struct FacilityPart(pub Owner);

#[derive(Component, Debug)]
pub struct Room {
    pub name: &'static str,
    pub min: IVec2,
    pub max: IVec2,
}

#[derive(Component, Debug)]
pub struct FloorCell {
    pub cell: IVec2,
}

#[derive(Component, Debug)]
pub struct Boundary {
    pub key: Key,
    pub facing: layout::Side,
}

#[derive(Component, Debug)]
pub struct Wall;

#[derive(Component, Debug)]
pub struct Passage;

#[derive(Component, Debug)]
pub struct Doorway {
    pub swing: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
}

impl DoorState {
    pub fn passable(self) -> bool {
        self == DoorState::Open
    }
}

#[derive(Component, Debug)]
pub struct WallPost;

#[derive(Component, Debug)]
pub struct FacilityProp(pub PropKind);

#[derive(Component, Debug)]
pub struct Decor;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Objective {
    Exit,
    FusePanel,
}

#[derive(Component, Debug)]
pub struct PlayerStart;

fn spawn_authored_facility(mut commands: Commands) {
    let layout = authored();
    let plan = derive(&layout).unwrap_or_else(|error| panic!("invalid facility layout: {error}"));
    spawn_facility(&mut commands, layout, plan);
}

pub fn spawn_facility(commands: &mut Commands, layout: Layout, plan: Plan) -> Entity {
    let facility = FacilityLayout::new(layout, plan);
    let (layout, plan) = (&facility.layout, &facility.plan);
    let mut grid = FacilityGrid::default();
    let root = commands
        .spawn((Facility, Name::new("facility"), Transform::default()))
        .id();

    for area in &layout.areas {
        grid.rooms.push(
            commands
                .spawn((
                    Room {
                        name: area.name,
                        min: area.min,
                        max: area.max,
                    },
                    Name::new(area.name),
                    Transform::default(),
                    ChildOf(root),
                ))
                .id(),
        );
    }
    let rooms = grid.rooms.clone();
    let room_of = |cell: IVec2| rooms[plan.cells[&(cell.x, cell.y)]];

    let mut parts = Vec::new();
    for &key in plan.cells.keys() {
        let cell = IVec2::new(key.0, key.1);
        let entity = commands
            .spawn((FloorCell { cell }, ChildOf(room_of(cell))))
            .id();
        grid.cells.insert(key, entity);
        parts.push((entity, Owner::Cell(key)));
    }

    for (&key, edge) in &plan.edges {
        let mut entity = commands.spawn((
            Boundary {
                key,
                facing: edge.facing,
            },
            ChildOf(root),
        ));
        match edge.kind {
            EdgeKind::Wall(_) => {
                entity.insert(Wall);
            }
            EdgeKind::Passage => {
                entity.insert(Passage);
            }
            EdgeKind::Doorway(door) => {
                let (swing, state) = match door {
                    Door::Open(degrees) => (degrees, DoorState::Open),
                    Door::Closed => (DEFAULT_DOOR_SWING, DoorState::Closed),
                };
                let exit = layout
                    .exit
                    .is_some_and(|(cell, side)| layout::edge_key(cell, side) == key);
                if exit {
                    entity.insert((Doorway { swing }, DoorState::Locked, Objective::Exit));
                } else {
                    entity.insert((Doorway { swing }, state));
                }
            }
        }
        grid.boundaries.insert(key, entity.id());
        parts.push((entity.id(), Owner::Edge(key)));
    }

    for &vertex in &plan.posts {
        let entity = commands.spawn((WallPost, ChildOf(root))).id();
        parts.push((entity, Owner::Post(vertex)));
    }

    for (index, prop) in layout.props.iter().enumerate() {
        let entity = commands
            .spawn((FacilityProp(prop.kind), ChildOf(room_of(prop.cell))))
            .id();
        parts.push((entity, Owner::Prop(index)));
    }

    for (index, fixture) in layout.fixtures.iter().enumerate() {
        let mut entity = commands.spawn((Decor, ChildOf(room_of(fixture.cell()))));
        if let Fixture::Wall {
            mount: Mount::FusePanel,
            ..
        } = fixture
        {
            entity.insert(Objective::FusePanel);
        }
        parts.push((entity.id(), Owner::Fixture(index)));
    }

    for (index, line) in layout.lines.iter().enumerate() {
        let entity = commands.spawn((Decor, ChildOf(room_of(line.start)))).id();
        parts.push((entity, Owner::Line(index)));
    }

    if let Some((cell, facing)) = layout.start {
        commands.spawn((
            PlayerStart,
            Name::new("player_start"),
            Transform::from_translation(layout::cell_center(cell))
                .with_rotation(Quat::from_rotation_y(facing.yaw())),
            ChildOf(room_of(cell)),
        ));
    }

    let anchors: Vec<_> = parts
        .into_iter()
        .map(|(entity, owner)| (entity, owner, facility.anchor(owner)))
        .collect();
    commands.entity(root).insert(facility);
    for (entity, owner, anchor) in anchors {
        commands
            .entity(entity)
            .insert((FacilityPart(owner), anchor));
    }
    commands.entity(root).insert(grid);
    root
}
