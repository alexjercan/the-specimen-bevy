use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::math::{IVec2, Quat, Vec2, Vec3};

pub const TILE: f32 = 2.5;
pub const HALF: f32 = TILE / 2.0;
pub const WALL_T: f32 = 0.2;
pub const POST: f32 = 0.36;
pub const DOOR_W: f32 = 1.2;
pub const DOOR_H: f32 = 2.2;
pub const FRAME_DEPTH: f32 = WALL_T / 2.0 + 0.035;
pub const LAMP_HEIGHT: f32 = 2.7;
pub const WALL_CLEARANCE: f32 = 0.2;
pub const DOOR_ZONE_DEPTH: f32 = 1.3;
pub const TRIM_BANDS: [(f32, f32); 3] = [(0.0, 0.16), (1.1, 1.16), (2.78, 3.0)];
pub const CONDUIT_BAND: (f32, f32) = (2.36, 2.64);
pub const HANGER_HALF: Vec2 = Vec2::new(0.85, 0.015);
pub const HANGER_ROW: f32 = 2.33;
pub const LABEL_HALF: Vec2 = Vec2::new(0.65, 0.1);
pub const ROW_ARROW_X: f32 = 0.67;
pub const ROW_LABEL_X: f32 = -0.12;
pub const ROW_HALF_W: f32 = ROW_ARROW_X + 0.1;
pub const ROW_PITCH: f32 = 0.24;
pub const LAMP_HALF: Vec2 = Vec2::new(0.7, 0.16);
pub const LINE_OFFSET: f32 = 0.55;
pub const LINE_HALF_W: f32 = 0.05;
pub const LINE_END_INSET: f32 = 0.25;
pub const STRIPE_X: f32 = 0.95;
pub const STRIPE_HALF_W: f32 = 0.04;
pub const STRIPE_END: f32 = 0.02;

pub type Key = (i32, i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    North,
    East,
    South,
    West,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::North, Side::East, Side::South, Side::West];

    pub fn offset(self) -> IVec2 {
        match self {
            Side::North => IVec2::new(0, -1),
            Side::East => IVec2::new(1, 0),
            Side::South => IVec2::new(0, 1),
            Side::West => IVec2::new(-1, 0),
        }
    }

    pub fn opposite(self) -> Side {
        match self {
            Side::North => Side::South,
            Side::East => Side::West,
            Side::South => Side::North,
            Side::West => Side::East,
        }
    }

    pub fn yaw(self) -> f32 {
        match self {
            Side::North => 0.0,
            Side::East => -FRAC_PI_2,
            Side::South => PI,
            Side::West => FRAC_PI_2,
        }
    }

    pub fn dir(self) -> Vec3 {
        let o = self.offset();
        Vec3::new(o.x as f32, 0.0, o.y as f32)
    }

    pub fn left(self) -> Side {
        match self {
            Side::North => Side::West,
            Side::West => Side::South,
            Side::South => Side::East,
            Side::East => Side::North,
        }
    }

    pub fn across(self) -> Vec3 {
        let d = self.dir();
        Vec3::new(d.z.abs(), 0.0, d.x.abs())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Floor {
    Concrete,
    Striped,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WallStyle {
    Plain,
    Conduit,
}

#[derive(Clone, Debug)]
pub struct Area {
    pub name: &'static str,
    pub min: IVec2,
    pub max: IVec2,
    pub floor: Floor,
    pub walls: WallStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Door {
    Closed,
    Open(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpeningKind {
    Passage,
    Door(Door),
}

#[derive(Clone, Copy, Debug)]
pub struct Opening {
    pub cell: IVec2,
    pub side: Side,
    pub kind: OpeningKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Lamp {
    Cool,
    Amber,
    Dead,
    Flicker,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dest {
    Boiler,
    Storage,
    Maintenance,
    Office,
    Exit,
}

impl Dest {
    pub fn label(self) -> &'static str {
        match self {
            Dest::Boiler => "sign_label_boiler_room",
            Dest::Storage => "sign_label_storage",
            Dest::Maintenance => "sign_label_maintenance",
            Dest::Office => "sign_label_office",
            Dest::Exit => "sign_label_exit",
        }
    }

    pub fn area(self) -> Option<&'static str> {
        match self {
            Dest::Boiler => Some("boiler"),
            Dest::Storage => Some("storage"),
            Dest::Maintenance => Some("maintenance"),
            Dest::Office => Some("office"),
            Dest::Exit => None,
        }
    }

    pub fn line(self) -> Option<&'static str> {
        match self {
            Dest::Boiler => Some("route_line_orange"),
            Dest::Storage => Some("route_line_blue"),
            Dest::Exit => Some("route_line_green"),
            Dest::Maintenance | Dest::Office => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrow {
    Left,
    Right,
    Ahead,
    Back,
}

impl Arrow {
    pub fn roll(self) -> f32 {
        match self {
            Arrow::Left => 0.0,
            Arrow::Right => PI,
            Arrow::Ahead => FRAC_PI_2,
            Arrow::Back => -FRAC_PI_2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    pub dest: Dest,
    pub arrow: Arrow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteLine {
    pub dest: Dest,
    pub start: IVec2,
    pub heading: Side,
    pub cells: i32,
    pub side: Side,
}

impl RouteLine {
    pub fn cells(&self) -> impl Iterator<Item = IVec2> + '_ {
        (0..self.cells).map(|i| self.start + self.heading.offset() * i)
    }

    pub fn last(&self) -> IVec2 {
        self.start + self.heading.offset() * (self.cells - 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mount {
    FaultLamp,
    ExitSign,
    Vent,
    Label(Dest),
    PipeManifold,
    Pegboard,
    CrawlVent,
    ClawMarks,
    FusePanel,
}

impl Mount {
    pub fn module(self) -> &'static str {
        match self {
            Mount::FaultLamp => "wall_lamp_red",
            Mount::ExitSign => "exit_sign",
            Mount::Vent => "wall_vent",
            Mount::Label(dest) => dest.label(),
            Mount::PipeManifold => "pipe_manifold",
            Mount::Pegboard => "tool_pegboard",
            Mount::CrawlVent => "concept_crawl_vent",
            Mount::ClawMarks => "trace_claw_marks",
            Mount::FusePanel => "fuse_panel",
        }
    }

    pub fn half_size(self) -> Vec2 {
        match self {
            Mount::FaultLamp => Vec2::new(0.12, 0.16),
            Mount::ExitSign => Vec2::new(0.25, 0.1),
            Mount::Vent => Vec2::new(0.3, 0.2),
            Mount::Label(_) => LABEL_HALF,
            Mount::PipeManifold => Vec2::new(0.95, 0.25),
            Mount::Pegboard => Vec2::new(0.6, 0.4),
            Mount::CrawlVent => Vec2::new(0.45, 0.35),
            Mount::ClawMarks => Vec2::new(0.35, 0.3),
            Mount::FusePanel => Vec2::new(0.55, 0.65),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Fixture {
    Ceiling {
        cell: IVec2,
        lamp: Lamp,
        along_z: bool,
    },
    Wall {
        cell: IVec2,
        side: Side,
        offset: f32,
        height: f32,
        mount: Mount,
    },
    WallSign {
        cell: IVec2,
        side: Side,
        offset: f32,
        height: f32,
        rows: [Option<Row>; 3],
        reader: IVec2,
    },
    ExitHanger {
        cell: IVec2,
        facing: Side,
        shift: f32,
    },
}

impl Fixture {
    pub fn cell(&self) -> IVec2 {
        match *self {
            Fixture::Ceiling { cell, .. }
            | Fixture::Wall { cell, .. }
            | Fixture::WallSign { cell, .. }
            | Fixture::ExitHanger { cell, .. } => cell,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PropKind {
    Crate,
    Drum,
    Shelf,
    Workbench,
    Boiler,
    ShelfLow,
    ShelfBins,
    Locker,
    Table,
    VentGrille,
    Tank,
    Console,
    Island,
    Papers,
    Tools,
    ChairTipped,
    DrumSpilled,
    DragMarks,
}

impl PropKind {
    pub fn module(self) -> &'static str {
        match self {
            PropKind::Crate => "storage_crate",
            PropKind::Drum => "steel_drum",
            PropKind::Shelf => "shelf_unit",
            PropKind::Workbench => "workbench",
            PropKind::Boiler => "boiler_unit",
            PropKind::ShelfLow => "shelf_unit_low",
            PropKind::ShelfBins => "shelf_unit_bins",
            PropKind::Locker => "concept_locker",
            PropKind::Table => "concept_table",
            PropKind::VentGrille => "vent_grille",
            PropKind::Tank => "concept_containment_tank",
            PropKind::Console => "lab_console",
            PropKind::Island => "work_island",
            PropKind::Papers => "clutter_papers",
            PropKind::Tools => "clutter_tools",
            PropKind::ChairTipped => "chair_tipped",
            PropKind::DrumSpilled => "drum_spilled",
            PropKind::DragMarks => "trace_drag_marks",
        }
    }

    pub fn half_size(self) -> Vec2 {
        match self {
            PropKind::Crate => Vec2::new(0.51, 0.415),
            PropKind::Drum => Vec2::new(0.3, 0.3),
            PropKind::Shelf | PropKind::ShelfBins => Vec2::new(0.9, 0.26),
            PropKind::Workbench => Vec2::new(0.8, 0.35),
            PropKind::Boiler => Vec2::new(0.8, 0.8),
            PropKind::ShelfLow => Vec2::new(0.6, 0.23),
            PropKind::Locker => Vec2::new(0.5, 0.5),
            PropKind::Table => Vec2::new(0.9, 0.47),
            PropKind::VentGrille => Vec2::new(0.43, 0.33),
            PropKind::Tank => Vec2::new(0.95, 0.95),
            PropKind::Console => Vec2::new(0.8, 0.35),
            PropKind::Island => Vec2::new(1.0, 0.5),
            PropKind::Papers => Vec2::new(0.6, 0.45),
            PropKind::Tools => Vec2::new(0.5, 0.35),
            PropKind::ChairTipped => Vec2::new(0.5, 0.45),
            PropKind::DrumSpilled => Vec2::new(0.75, 0.45),
            PropKind::DragMarks => Vec2::new(0.3, 1.0),
        }
    }

    pub fn walkable(self) -> bool {
        matches!(self, PropKind::Papers | PropKind::DragMarks)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub kind: PropKind,
    pub cell: IVec2,
    pub offset: Vec2,
    pub facing: Side,
}

#[derive(Clone, Debug, Default)]
pub struct Layout {
    pub areas: Vec<Area>,
    pub openings: Vec<Opening>,
    pub fixtures: Vec<Fixture>,
    pub props: Vec<Prop>,
    pub lines: Vec<RouteLine>,
    pub exit: Option<(IVec2, Side)>,
    pub start: Option<(IVec2, Side)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    Floor,
    Ceiling,
    Wall,
    Doorway,
    Door,
    Post,
    Light,
    Mount,
    Prop,
    Sign,
    Route,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Glow {
    Steady,
    Flicker(f32),
    Pulse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Owner {
    Cell(Key),
    Edge(Key),
    Post(Key),
    Fixture(usize),
    Prop(usize),
    Line(usize),
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub owner: Owner,
    pub module: &'static str,
    pub part: Part,
    pub translation: Vec3,
    pub yaw: f32,
    pub roll: f32,
    pub scale: Vec3,
    pub glow: Option<Glow>,
}

impl Piece {
    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw) * Quat::from_rotation_z(self.roll)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightSpec {
    pub owner: Owner,
    pub position: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub shadows: bool,
    pub glow: Glow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EdgeKind {
    Wall(WallStyle),
    Doorway(Door),
    Passage,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub kind: EdgeKind,
    pub facing: Side,
}

impl Edge {
    pub fn solid(&self) -> bool {
        self.kind != EdgeKind::Passage
    }

    fn joint(&self) -> (u8, Side) {
        match self.kind {
            EdgeKind::Wall(WallStyle::Plain) => (0, self.facing),
            EdgeKind::Wall(WallStyle::Conduit) => (1, self.facing),
            EdgeKind::Doorway(_) => (2, self.facing),
            EdgeKind::Passage => (3, self.facing),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub cells: BTreeMap<(i32, i32), usize>,
    pub edges: BTreeMap<(i32, i32), Edge>,
    pub posts: Vec<(i32, i32)>,
    pub pieces: Vec<Piece>,
    pub lights: Vec<LightSpec>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub cell: IVec2,
    pub side: Side,
}

impl Plan {
    pub fn anchor(&self, layout: &Layout, owner: Owner) -> (Vec3, f32) {
        match owner {
            Owner::Cell(key) => (cell_center(IVec2::new(key.0, key.1)), 0.0),
            Owner::Edge(key) => (half_point(key), self.edges[&key].facing.yaw()),
            Owner::Post(key) => (half_point(key), 0.0),
            Owner::Prop(index) => {
                let prop = &layout.props[index];
                (
                    cell_center(prop.cell) + Vec3::new(prop.offset.x, 0.0, prop.offset.y),
                    prop.facing.yaw(),
                )
            }
            Owner::Fixture(index) => match layout.fixtures[index] {
                Fixture::Wall { .. } => self
                    .pieces
                    .iter()
                    .find(|piece| piece.owner == owner)
                    .map(|piece| (piece.translation, piece.yaw))
                    .expect("wall fixture has a piece"),
                fixture => (cell_center(fixture.cell()), 0.0),
            },
            Owner::Line(index) => (cell_center(layout.lines[index].start), 0.0),
        }
    }
}

pub fn half_point(key: (i32, i32)) -> Vec3 {
    Vec3::new(key.0 as f32 * HALF, 0.0, key.1 as f32 * HALF)
}

pub fn cell_center(cell: IVec2) -> Vec3 {
    Vec3::new(cell.x as f32 * TILE, 0.0, cell.y as f32 * TILE)
}

pub fn edge_key(cell: IVec2, side: Side) -> (i32, i32) {
    let k = cell * 2 + side.offset();
    (k.x, k.y)
}

pub fn edge_along(key: (i32, i32)) -> Vec3 {
    if key.0.rem_euclid(2) == 1 {
        Vec3::Z
    } else {
        Vec3::X
    }
}

pub fn edge_vertices(key: (i32, i32)) -> [(i32, i32); 2] {
    if key.0.rem_euclid(2) == 1 {
        [(key.0, key.1 - 1), (key.0, key.1 + 1)]
    } else {
        [(key.0 - 1, key.1), (key.0 + 1, key.1)]
    }
}

pub fn wall_right(facing: Side) -> Vec3 {
    Quat::from_rotation_y(facing.yaw()) * Vec3::X
}

pub fn cell_of(point: Vec3) -> (i32, i32) {
    (
        (point.x / TILE).round() as i32,
        (point.z / TILE).round() as i32,
    )
}

pub fn door_zones(plan: &Plan) -> Vec<(Vec2, Vec2)> {
    plan.edges
        .iter()
        .filter(|(_, edge)| matches!(edge.kind, EdgeKind::Doorway(_)))
        .map(|(key, _)| {
            let mid = half_point(*key);
            let along = edge_along(*key);
            let half =
                along * (DOOR_W / 2.0 + 0.1) + Vec3::new(along.z, 0.0, along.x) * DOOR_ZONE_DEPTH;
            (
                Vec2::new(mid.x - half.x, mid.z - half.z),
                Vec2::new(mid.x + half.x, mid.z + half.z),
            )
        })
        .collect()
}

pub fn prop_bounds(prop: &Prop) -> (Vec2, Vec2) {
    let center = cell_center(prop.cell) + Vec3::new(prop.offset.x, 0.0, prop.offset.y);
    let half = match prop.facing {
        Side::North | Side::South => prop.kind.half_size(),
        Side::East | Side::West => {
            let h = prop.kind.half_size();
            Vec2::new(h.y, h.x)
        }
    };
    let c = Vec2::new(center.x, center.z);
    (c - half, c + half)
}

fn overlaps(a: (Vec2, Vec2), b: (Vec2, Vec2)) -> bool {
    a.0.x < b.1.x && b.0.x < a.1.x && a.0.y < b.1.y && b.0.y < a.1.y
}

pub fn derive(layout: &Layout) -> Result<Plan, String> {
    let mut plan = Plan::default();
    for (index, area) in layout.areas.iter().enumerate() {
        if area.min.x > area.max.x || area.min.y > area.max.y {
            return Err(format!("area {} has an empty rectangle", area.name));
        }
        for x in area.min.x..=area.max.x {
            for z in area.min.y..=area.max.y {
                if let Some(other) = plan.cells.insert((x, z), index) {
                    return Err(format!(
                        "cell ({x}, {z}) is in both {} and {}",
                        layout.areas[other].name, area.name
                    ));
                }
            }
        }
    }

    let mut openings = BTreeMap::new();
    for opening in &layout.openings {
        let cell = (opening.cell.x, opening.cell.y);
        let Some(&area) = plan.cells.get(&cell) else {
            return Err(format!("opening at {cell:?} is outside every area"));
        };
        let n = opening.cell + opening.side.offset();
        let neighbor = plan.cells.get(&(n.x, n.y)).copied();
        if neighbor == Some(area) {
            return Err(format!(
                "opening at {cell:?} {:?} is inside {}",
                opening.side, layout.areas[area].name
            ));
        }
        if neighbor.is_none() && opening.kind == OpeningKind::Passage {
            return Err(format!(
                "passage at {cell:?} {:?} leads to void",
                opening.side
            ));
        }
        if openings
            .insert(edge_key(opening.cell, opening.side), *opening)
            .is_some()
        {
            return Err(format!(
                "two openings share the edge at {cell:?} {:?}",
                opening.side
            ));
        }
    }

    for (&(x, z), &area) in &plan.cells {
        let cell = IVec2::new(x, z);
        for side in Side::ALL {
            let n = cell + side.offset();
            let neighbor = plan.cells.get(&(n.x, n.y)).copied();
            if neighbor == Some(area) {
                continue;
            }
            let key = edge_key(cell, side);
            if plan.edges.contains_key(&key) {
                continue;
            }
            let edge = match openings.get(&key) {
                Some(opening) => Edge {
                    kind: match opening.kind {
                        OpeningKind::Passage => EdgeKind::Passage,
                        OpeningKind::Door(door) => EdgeKind::Doorway(door),
                    },
                    facing: opening.side.opposite(),
                },
                None => {
                    let (owner, facing) = match neighbor {
                        Some(other) if other < area => (other, side),
                        _ => (area, side.opposite()),
                    };
                    Edge {
                        kind: EdgeKind::Wall(layout.areas[owner].walls),
                        facing,
                    }
                }
            };
            plan.edges.insert(key, edge);
        }
    }

    for (&(x, z), &area) in &plan.cells {
        let center = cell_center(IVec2::new(x, z));
        let floor = match layout.areas[area].floor {
            Floor::Concrete => "floor_tile",
            Floor::Striped => "floor_tile_marked",
        };
        let owner = Owner::Cell((x, z));
        plan.pieces
            .push(piece(owner, floor, Part::Floor, center, 0.0));
        plan.pieces
            .push(piece(owner, "ceiling_tile", Part::Ceiling, center, 0.0));
    }

    for (&key, edge) in &plan.edges {
        let mid = half_point(key);
        let yaw = edge.facing.yaw();
        let owner = Owner::Edge(key);
        match edge.kind {
            EdgeKind::Passage => {}
            EdgeKind::Wall(style) => {
                let module = match style {
                    WallStyle::Plain => "wall",
                    WallStyle::Conduit => "wall_conduit",
                };
                plan.pieces.push(piece(owner, module, Part::Wall, mid, yaw));
            }
            EdgeKind::Doorway(door) => {
                plan.pieces
                    .push(piece(owner, "wall_doorway", Part::Doorway, mid, yaw));
                let (translation, yaw) = door_panel(key, edge.facing, door);
                plan.pieces
                    .push(piece(owner, "door_panel", Part::Door, translation, yaw));
            }
        }
    }

    plan.posts = derive_posts(&plan.edges);
    for &vertex in &plan.posts {
        plan.pieces.push(piece(
            Owner::Post(vertex),
            "wall_post",
            Part::Post,
            half_point(vertex),
            0.0,
        ));
    }

    if let Some((cell, side)) = layout.exit {
        let n = cell + side.offset();
        let edge = plan.edges.get(&edge_key(cell, side));
        if !matches!(edge.map(|e| e.kind), Some(EdgeKind::Doorway(_)))
            || plan.cells.contains_key(&(n.x, n.y))
        {
            return Err(format!(
                "exit at {cell} {side:?} is not a door to the outside"
            ));
        }
    }

    if let Some((cell, _)) = layout.start {
        if !plan.cells.contains_key(&(cell.x, cell.y)) {
            return Err(format!("player start at {cell} is outside every area"));
        }
    }

    for (index, fixture) in layout.fixtures.iter().enumerate() {
        place_fixture(&mut plan, layout, fixture, Owner::Fixture(index))?;
    }

    for (index, line) in layout.lines.iter().enumerate() {
        place_line(&mut plan, layout, line, Owner::Line(index))?;
    }

    let zones = door_zones(&plan);
    for (index, prop) in layout.props.iter().enumerate() {
        let cell = (prop.cell.x, prop.cell.y);
        let Some(&area) = plan.cells.get(&cell) else {
            return Err(format!("{:?} at {cell:?} is outside every area", prop.kind));
        };
        let (lo, hi) = prop_bounds(prop);
        for corner in [lo, hi, Vec2::new(lo.x, hi.y), Vec2::new(hi.x, lo.y)] {
            for d in [
                Vec2::new(WALL_CLEARANCE, WALL_CLEARANCE),
                Vec2::new(-WALL_CLEARANCE, -WALL_CLEARANCE),
                Vec2::new(WALL_CLEARANCE, -WALL_CLEARANCE),
                Vec2::new(-WALL_CLEARANCE, WALL_CLEARANCE),
            ] {
                let p = corner + d;
                if plan.cells.get(&cell_of(Vec3::new(p.x, 0.0, p.y))) != Some(&area) {
                    return Err(format!(
                        "{:?} at {cell:?} is closer than {WALL_CLEARANCE} m to the edge of {}",
                        prop.kind, layout.areas[area].name
                    ));
                }
            }
        }
        if zones.iter().any(|zone| overlaps(*zone, (lo, hi))) {
            return Err(format!("{:?} at {cell:?} blocks a door", prop.kind));
        }
        if !prop.kind.walkable() {
            if let Some(other) = layout.props[..index]
                .iter()
                .find(|o| !o.kind.walkable() && overlaps(prop_bounds(o), (lo, hi)))
            {
                return Err(format!(
                    "{:?} at {cell:?} overlaps {:?} at {}",
                    prop.kind, other.kind, other.cell
                ));
            }
        }
        let center = cell_center(prop.cell) + Vec3::new(prop.offset.x, 0.0, prop.offset.y);
        let light = match prop.kind {
            PropKind::Boiler => Some((FIREBOX, [1.0, 0.35, 0.08], 45_000.0, 7.0, center.x * 0.7)),
            PropKind::Tank => Some((TANK_LAMP, [0.3, 0.95, 0.85], 12_000.0, 4.5, center.z)),
            _ => None,
        };
        let glow = light.map(|l| Glow::Flicker(l.4));
        let owner = Owner::Prop(index);
        plan.pieces.push(Piece {
            glow,
            ..piece(
                owner,
                prop.kind.module(),
                Part::Prop,
                center,
                prop.facing.yaw(),
            )
        });
        if let (Some((anchor, color, intensity, range, _)), Some(glow)) = (light, glow) {
            plan.lights.push(LightSpec {
                owner,
                position: center + Quat::from_rotation_y(prop.facing.yaw()) * anchor,
                color,
                intensity,
                range,
                shadows: true,
                glow,
            });
        }
    }
    Ok(plan)
}

pub const FIREBOX: Vec3 = Vec3::new(0.0, 0.5, -0.85);
pub const TANK_LAMP: Vec3 = Vec3::new(0.0, 2.1, 0.0);

pub fn door_panel(key: Key, facing: Side, door: Door) -> (Vec3, f32) {
    let hinge = half_point(key) - wall_right(facing) * (DOOR_W / 2.0 - 0.01);
    match door {
        Door::Closed => (hinge, facing.yaw()),
        Door::Open(degrees) => (
            hinge + facing.dir() * (FRAME_DEPTH + 0.035),
            facing.yaw() + degrees.to_radians(),
        ),
    }
}

fn piece(owner: Owner, module: &'static str, part: Part, translation: Vec3, yaw: f32) -> Piece {
    Piece {
        owner,
        module,
        part,
        translation,
        yaw,
        roll: 0.0,
        scale: Vec3::ONE,
        glow: None,
    }
}

pub fn moves(plan: &Plan, cell: IVec2) -> Vec<(Side, Option<IVec2>)> {
    moves_through(plan, cell, |_| true)
}

pub fn moves_through(
    plan: &Plan,
    cell: IVec2,
    door_open: impl Fn(Key) -> bool,
) -> Vec<(Side, Option<IVec2>)> {
    let Some(&area) = plan.cells.get(&(cell.x, cell.y)) else {
        return Vec::new();
    };
    Side::ALL
        .into_iter()
        .filter_map(|side| {
            let n = cell + side.offset();
            let neighbor = plan.cells.get(&(n.x, n.y)).copied();
            if neighbor == Some(area) {
                return Some((side, Some(n)));
            }
            let key = edge_key(cell, side);
            match plan.edges.get(&key)?.kind {
                EdgeKind::Wall(_) => None,
                EdgeKind::Doorway(_) if !door_open(key) => None,
                EdgeKind::Passage | EdgeKind::Doorway(_) => Some((side, neighbor.map(|_| n))),
            }
        })
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reach {
    pub cells: BTreeSet<Key>,
    pub outside: bool,
}

pub fn reachable(plan: &Plan, from: IVec2, door_open: impl Fn(Key) -> bool) -> Reach {
    let mut reach = Reach::default();
    if !plan.cells.contains_key(&(from.x, from.y)) {
        return reach;
    }
    reach.cells.insert((from.x, from.y));
    let mut queue = VecDeque::from([from]);
    while let Some(cell) = queue.pop_front() {
        for (_, next) in moves_through(plan, cell, &door_open) {
            match next {
                None => reach.outside = true,
                Some(next) => {
                    if reach.cells.insert((next.x, next.y)) {
                        queue.push_back(next);
                    }
                }
            }
        }
    }
    reach
}

pub fn path(plan: &Plan, layout: &Layout, from: IVec2, dest: Dest) -> Result<Vec<Step>, String> {
    let target = dest
        .area()
        .map(|name| layout.areas.iter().position(|a| a.name == name));
    let exit = layout.exit;
    let reached = |cell: IVec2| match target {
        Some(Some(index)) => plan.cells.get(&(cell.x, cell.y)) == Some(&index),
        Some(None) => false,
        None => exit.is_some_and(|(c, _)| c == cell),
    };
    let finish = |cell: IVec2, mut steps: Vec<Step>| {
        if let (None, Some((_, side))) = (target, exit) {
            steps.push(Step { cell, side });
        }
        steps
    };
    if !plan.cells.contains_key(&(from.x, from.y)) {
        return Err(format!("route start {from} is outside every area"));
    }
    if reached(from) {
        return Ok(finish(from, Vec::new()));
    }
    let mut prev: BTreeMap<(i32, i32), Step> = BTreeMap::new();
    let mut queue = VecDeque::from([from]);
    while let Some(cell) = queue.pop_front() {
        for (side, next) in moves(plan, cell) {
            let Some(next) = next else { continue };
            if next == from || prev.contains_key(&(next.x, next.y)) {
                continue;
            }
            prev.insert((next.x, next.y), Step { cell, side });
            if reached(next) {
                let mut steps = Vec::new();
                let mut at = next;
                while at != from {
                    let step = prev[&(at.x, at.y)];
                    steps.push(step);
                    at = step.cell;
                }
                steps.reverse();
                return Ok(finish(next, steps));
            }
            queue.push_back(next);
        }
    }
    Err(format!("no route from {from} to {dest:?}"))
}

pub fn relative(forward: Side, side: Side) -> Option<Arrow> {
    if side == forward {
        Some(Arrow::Ahead)
    } else if side == forward.left() {
        Some(Arrow::Left)
    } else if side == forward.left().opposite() {
        Some(Arrow::Right)
    } else {
        None
    }
}

pub fn expected_arrow(
    plan: &Plan,
    layout: &Layout,
    cell: IVec2,
    forward: Side,
    dest: Dest,
) -> Result<Arrow, String> {
    let steps = path(plan, layout, cell, dest)?;
    if steps.is_empty() {
        return Err(format!("sign for {dest:?} at {cell} is inside {dest:?}"));
    }
    for step in &steps {
        if step.side != forward || moves(plan, step.cell).len() >= 3 {
            return relative(forward, step.side).ok_or_else(|| {
                format!("{dest:?} from {cell} is behind a reader facing {forward:?}")
            });
        }
    }
    Ok(Arrow::Ahead)
}

pub fn hanger_bounds(cell: IVec2, facing: Side, shift: f32) -> (Vec2, Vec2) {
    let center = cell_center(cell) + facing.dir() * shift;
    let half = facing.across() * HANGER_HALF.x + facing.dir().abs() * HANGER_HALF.y;
    (
        Vec2::new(center.x - half.x, center.z - half.z),
        Vec2::new(center.x + half.x, center.z + half.z),
    )
}

pub fn lamp_bounds(cell: IVec2, along_z: bool) -> (Vec2, Vec2) {
    let c = cell_center(cell);
    let half = if along_z {
        Vec2::new(LAMP_HALF.y, LAMP_HALF.x)
    } else {
        LAMP_HALF
    };
    let c = Vec2::new(c.x, c.z);
    (c - half, c + half)
}

pub fn sign_rows(rows: &[Option<Row>; 3]) -> impl Iterator<Item = (Row, f32)> + '_ {
    rows.iter()
        .flatten()
        .enumerate()
        .map(|(i, row)| (*row, i as f32 * ROW_PITCH))
}

pub fn row_span(rows: &[Option<Row>; 3], height: f32) -> (f32, f32) {
    let drop = sign_rows(rows).map(|(_, d)| d).fold(0.0, f32::max);
    (height - drop - LABEL_HALF.y, height + LABEL_HALF.y)
}

fn check_arrow(
    plan: &Plan,
    layout: &Layout,
    reader: IVec2,
    forward: Side,
    row: Row,
) -> Result<(), String> {
    let expected = expected_arrow(plan, layout, reader, forward, row.dest)?;
    if expected != row.arrow {
        return Err(format!(
            "wrong-way arrow: {:?} for a reader at {reader} facing {forward:?} shows {:?}, the route needs {expected:?}",
            row.dest, row.arrow
        ));
    }
    Ok(())
}

fn push_row(plan: &mut Plan, owner: Owner, row: Row, center: Vec3, yaw: f32, roll: f32) {
    let rot = Quat::from_rotation_y(yaw);
    let (arrow_x, label_x) = match row.arrow {
        Arrow::Right => (-ROW_ARROW_X, -ROW_LABEL_X),
        Arrow::Left | Arrow::Ahead | Arrow::Back => (ROW_ARROW_X, ROW_LABEL_X),
    };
    plan.pieces.push(piece(
        owner,
        row.dest.label(),
        Part::Sign,
        center + rot * Vec3::new(label_x, 0.0, 0.0),
        yaw,
    ));
    plan.pieces.push(Piece {
        roll,
        ..piece(
            owner,
            "sign_arrow",
            Part::Sign,
            center + rot * Vec3::new(arrow_x, 0.0, 0.0),
            yaw,
        )
    });
}

pub fn hanger_readers(facing: Side) -> [Side; 2] {
    [facing.opposite(), facing]
}

fn place_hanger(
    plan: &mut Plan,
    layout: &Layout,
    owner: Owner,
    cell: IVec2,
    facing: Side,
    shift: f32,
) -> Result<(), String> {
    if !plan.cells.contains_key(&(cell.x, cell.y)) {
        return Err(format!("exit hanger at {cell} is outside every area"));
    }
    if shift.abs() > HALF - 0.2 {
        return Err(format!("exit hanger at {cell} leaves its cell"));
    }
    let center = cell_center(cell) + facing.dir() * shift;
    let bounds = hanger_bounds(cell, facing, shift);
    for fixture in &layout.fixtures {
        if let Fixture::Ceiling {
            cell: lamp_cell,
            along_z,
            ..
        } = *fixture
        {
            if overlaps(lamp_bounds(lamp_cell, along_z), bounds) {
                return Err(format!(
                    "exit hanger at {cell} hits the ceiling light at {lamp_cell}"
                ));
            }
        }
    }
    if door_zones(plan).iter().any(|zone| overlaps(*zone, bounds)) {
        return Err(format!("exit hanger at {cell} hangs in a door swing"));
    }
    let exits: Vec<Side> = moves(plan, cell).into_iter().map(|(s, _)| s).collect();
    let exit_step = path(plan, layout, cell, Dest::Exit)?
        .first()
        .ok_or_else(|| format!("exit hanger at {cell} has no route to exit"))?
        .side;
    for reader in hanger_readers(facing) {
        if !exits.contains(&reader.opposite()) {
            return Err(format!("exit hanger at {cell} faces a wall"));
        }
        let arrow = relative(reader, exit_step).unwrap_or(Arrow::Back);
        let yaw = reader.opposite().yaw();
        let face = Quat::from_rotation_y(yaw) * Vec3::new(0.0, HANGER_ROW, -HANGER_HALF.y);
        push_row(
            plan,
            owner,
            Row {
                dest: Dest::Exit,
                arrow,
            },
            center + face,
            yaw,
            arrow.roll(),
        );
    }
    plan.pieces.push(piece(
        owner,
        "sign_hanger",
        Part::Sign,
        center,
        facing.yaw(),
    ));
    Ok(())
}

fn wall_slot(
    plan: &Plan,
    what: &str,
    cell: IVec2,
    side: Side,
    offset: f32,
    half_w: f32,
    (bottom, top): (f32, f32),
) -> Result<Vec3, String> {
    let key = edge_key(cell, side);
    let Some(edge) = plan.edges.get(&key).copied() else {
        return Err(format!("{what} at {cell} {side:?} has no wall"));
    };
    let face = side.opposite();
    if offset.abs() + half_w > HALF - POST / 2.0 {
        return Err(format!("{what} at {cell} {side:?} overlaps a post"));
    }
    if TRIM_BANDS.iter().any(|(lo, hi)| bottom < *hi && *lo < top) {
        return Err(format!("{what} at {cell} {side:?} crosses a wall trim"));
    }
    match edge.kind {
        EdgeKind::Passage => {
            return Err(format!("{what} at {cell} {side:?} is on a passage"));
        }
        EdgeKind::Doorway(_) if bottom < DOOR_H + 0.25 => {
            return Err(format!("{what} at {cell} {side:?} overlaps the door frame"));
        }
        EdgeKind::Wall(WallStyle::Conduit)
            if edge.facing == face && bottom < CONDUIT_BAND.1 && CONDUIT_BAND.0 < top =>
        {
            return Err(format!("{what} at {cell} {side:?} overlaps the conduit"));
        }
        _ => {}
    }
    Ok(half_point(key) + face.dir() * (WALL_T / 2.0) + wall_right(face) * offset)
}

fn place_wall_sign(
    plan: &mut Plan,
    layout: &Layout,
    fixture: &Fixture,
    owner: Owner,
) -> Result<(), String> {
    let Fixture::WallSign {
        cell,
        side,
        offset,
        height,
        rows,
        reader,
    } = *fixture
    else {
        return Ok(());
    };
    if sign_rows(&rows).next().is_none() {
        return Err(format!("wall sign at {cell} {side:?} has no rows"));
    }
    let base = wall_slot(
        plan,
        "wall sign",
        cell,
        side,
        offset,
        ROW_HALF_W,
        row_span(&rows, height),
    )?;
    if !plan.cells.contains_key(&(reader.x, reader.y)) {
        return Err(format!(
            "reader of the wall sign at {cell} is outside every area"
        ));
    }
    let yaw = side.opposite().yaw();
    for (row, drop) in sign_rows(&rows) {
        check_arrow(plan, layout, reader, side, row)?;
        push_row(
            plan,
            owner,
            row,
            base + Vec3::Y * (height - drop),
            yaw,
            row.arrow.roll(),
        );
    }
    Ok(())
}

pub fn stripe_bounds(cell: IVec2) -> [(Vec2, Vec2); 2] {
    let c = cell_center(cell);
    [-STRIPE_X, STRIPE_X].map(|x| {
        (
            Vec2::new(c.x + x - STRIPE_HALF_W, c.z - HALF + STRIPE_END),
            Vec2::new(c.x + x + STRIPE_HALF_W, c.z + HALF - STRIPE_END),
        )
    })
}

pub fn line_bounds(piece: &Piece) -> (Vec2, Vec2) {
    let along = Quat::from_rotation_y(piece.yaw) * Vec3::X;
    let half =
        along.abs() * piece.scale.x / 2.0 + Vec3::new(along.z, 0.0, along.x).abs() * LINE_HALF_W;
    let c = piece.translation;
    (
        Vec2::new(c.x - half.x, c.z - half.z),
        Vec2::new(c.x + half.x, c.z + half.z),
    )
}

fn solid(plan: &Plan, cell: IVec2, side: Side) -> bool {
    plan.edges
        .get(&edge_key(cell, side))
        .is_some_and(Edge::solid)
}

fn place_line(
    plan: &mut Plan,
    layout: &Layout,
    line: &RouteLine,
    owner: Owner,
) -> Result<(), String> {
    let dest = line.dest;
    let Some(module) = dest.line() else {
        return Err(format!("{dest:?} has no route color"));
    };
    if line.cells < 1 || relative(line.heading, line.side).is_none_or(|a| a == Arrow::Ahead) {
        return Err(format!(
            "line to {dest:?} at {} does not keep to one side",
            line.start
        ));
    }
    let last = line.last();
    let first_step = |cell: IVec2| -> Result<Option<Side>, String> {
        Ok(path(plan, layout, cell, dest)?.first().map(|s| s.side))
    };
    let turn_in = layout
        .lines
        .iter()
        .any(|o| o.dest == dest && o.last() == line.start && o.side == line.heading);
    let mut turn_out = false;
    for cell in line.cells() {
        let step = first_step(cell)?;
        if cell == last && step == Some(line.side) {
            turn_out = true;
        } else if cell == last && step == Some(line.side.opposite()) {
            return Err(format!(
                "line to {dest:?} at {cell} runs on the outside of the turn"
            ));
        } else if step != Some(line.heading) {
            return Err(format!(
                "line to {dest:?} at {cell} does not run along the route"
            ));
        }
    }
    if turn_out {
        let resumes = layout.lines.iter().any(|o| {
            o.dest == dest
                && o.start == last
                && o.heading == line.side
                && o.side == line.heading.opposite()
        });
        if !resumes {
            return Err(format!(
                "line to {dest:?} turns at {last} but does not resume on the inside of the turn"
            ));
        }
    } else {
        let turn = path(plan, layout, last, dest)?
            .into_iter()
            .find(|s| s.side != line.heading);
        if turn.is_some_and(|s| s.side != line.side) {
            return Err(format!(
                "line to {dest:?} at {last} runs on the outside of the turn"
            ));
        }
    }
    let along = line.heading.dir();
    for cell in line.cells() {
        let s0 = if cell == line.start && turn_in {
            LINE_OFFSET + LINE_HALF_W
        } else if solid(plan, cell, line.heading.opposite()) {
            -HALF + LINE_END_INSET
        } else {
            -HALF
        };
        let s1 = if cell == last && turn_out {
            -(LINE_OFFSET - LINE_HALF_W)
        } else if solid(plan, cell, line.heading) {
            HALF - LINE_END_INSET
        } else {
            HALF
        };
        let center = cell_center(cell) + line.side.dir() * LINE_OFFSET + along * (s0 + s1) / 2.0;
        let piece = Piece {
            scale: Vec3::new(s1 - s0, 1.0, 1.0),
            ..piece(
                owner,
                module,
                Part::Route,
                center,
                line.heading.yaw() + FRAC_PI_2,
            )
        };
        let bounds = line_bounds(&piece);
        if let Some(prop) = layout
            .props
            .iter()
            .find(|p| overlaps(prop_bounds(p), bounds))
        {
            return Err(format!(
                "line to {dest:?} at {cell} crosses {:?} at {}",
                prop.kind, prop.cell
            ));
        }
        for (&(x, z), &area) in &plan.cells {
            if layout.areas[area].floor == Floor::Striped
                && stripe_bounds(IVec2::new(x, z))
                    .into_iter()
                    .any(|stripe| overlaps(stripe, bounds))
            {
                return Err(format!("line to {dest:?} at {cell} crosses a floor stripe"));
            }
        }
        if plan
            .pieces
            .iter()
            .any(|p| p.part == Part::Route && overlaps(line_bounds(p), bounds))
        {
            return Err(format!("line to {dest:?} at {cell} overlaps another line"));
        }
        plan.pieces.push(piece);
    }
    Ok(())
}

fn derive_posts(edges: &BTreeMap<(i32, i32), Edge>) -> Vec<(i32, i32)> {
    let mut joints: BTreeMap<Key, Vec<(Key, &Edge)>> = BTreeMap::new();
    for (key, edge) in edges.iter().filter(|(_, edge)| edge.solid()) {
        for vertex in edge_vertices(*key) {
            joints.entry(vertex).or_default().push((*key, edge));
        }
    }
    joints
        .into_iter()
        .filter(|(vertex, list)| match list.as_slice() {
            [(a, ea), (b, eb)] => {
                let straight = edge_along(*a) == edge_along(*b);
                let along = if edge_along(*a) == Vec3::X {
                    vertex.0
                } else {
                    vertex.1
                };
                !straight || ea.joint() != eb.joint() || (along + 1).rem_euclid(4) == 0
            }
            _ => true,
        })
        .map(|(vertex, _)| vertex)
        .collect()
}

fn place_fixture(
    plan: &mut Plan,
    layout: &Layout,
    fixture: &Fixture,
    owner: Owner,
) -> Result<(), String> {
    match *fixture {
        Fixture::WallSign { .. } => place_wall_sign(plan, layout, fixture, owner)?,
        Fixture::ExitHanger {
            cell,
            facing,
            shift,
        } => place_hanger(plan, layout, owner, cell, facing, shift)?,
        Fixture::Ceiling {
            cell,
            lamp,
            along_z,
        } => {
            if !plan.cells.contains_key(&(cell.x, cell.y)) {
                return Err(format!("ceiling light at {cell} is outside every area"));
            }
            let center = cell_center(cell);
            let module = match lamp {
                Lamp::Cool | Lamp::Flicker => "ceiling_light_cool",
                Lamp::Amber => "ceiling_light_amber",
                Lamp::Dead => "ceiling_light_dead",
            };
            let glow = match lamp {
                Lamp::Dead => None,
                Lamp::Flicker => Some(Glow::Flicker(cell.x as f32 * 1.7 + cell.y as f32 * 0.9)),
                Lamp::Cool | Lamp::Amber => Some(Glow::Steady),
            };
            let yaw = if along_z { FRAC_PI_2 } else { 0.0 };
            plan.pieces.push(Piece {
                glow,
                ..piece(owner, module, Part::Light, center, yaw)
            });
            if let Some(glow) = glow {
                let (color, intensity) = match lamp {
                    Lamp::Amber => ([1.0, 0.58, 0.2], 140_000.0),
                    _ => ([0.78, 0.88, 1.0], 90_000.0),
                };
                plan.lights.push(LightSpec {
                    owner,
                    position: center + Vec3::Y * LAMP_HEIGHT,
                    color,
                    intensity,
                    range: 9.0,
                    shadows: true,
                    glow,
                });
            }
        }
        Fixture::Wall {
            cell,
            side,
            offset,
            height,
            mount,
        } => {
            let half = mount.half_size();
            let base = wall_slot(
                plan,
                &format!("{mount:?}"),
                cell,
                side,
                offset,
                half.x,
                (height - half.y, height + half.y),
            )?;
            let face = side.opposite();
            let position = base + Vec3::Y * height;
            let light = match mount {
                Mount::FaultLamp => Some(([1.0, 0.06, 0.03], 60_000.0, 0.25, Glow::Pulse)),
                Mount::ExitSign => Some(([0.12, 1.0, 0.35], 6_000.0, 0.2, Glow::Steady)),
                Mount::Vent
                | Mount::Label(_)
                | Mount::PipeManifold
                | Mount::Pegboard
                | Mount::CrawlVent
                | Mount::ClawMarks
                | Mount::FusePanel => None,
            };
            plan.pieces.push(Piece {
                glow: light.map(|l| l.3),
                ..piece(owner, mount.module(), Part::Mount, position, face.yaw())
            });
            if let Some((color, intensity, depth, glow)) = light {
                plan.lights.push(LightSpec {
                    owner,
                    position: position + face.dir() * depth,
                    color,
                    intensity,
                    range: 5.0,
                    shadows: false,
                    glow,
                });
            }
        }
    }
    Ok(())
}

fn area(
    name: &'static str,
    min: (i32, i32),
    max: (i32, i32),
    floor: Floor,
    walls: WallStyle,
) -> Area {
    Area {
        name,
        min: IVec2::new(min.0, min.1),
        max: IVec2::new(max.0, max.1),
        floor,
        walls,
    }
}

fn opening(cell: (i32, i32), side: Side, kind: OpeningKind) -> Opening {
    Opening {
        cell: IVec2::new(cell.0, cell.1),
        side,
        kind,
    }
}

fn lamp(cell: (i32, i32), lamp: Lamp, along_z: bool) -> Fixture {
    Fixture::Ceiling {
        cell: IVec2::new(cell.0, cell.1),
        lamp,
        along_z,
    }
}

fn mount(cell: (i32, i32), side: Side, offset: f32, height: f32, mount: Mount) -> Fixture {
    Fixture::Wall {
        cell: IVec2::new(cell.0, cell.1),
        side,
        offset,
        height,
        mount,
    }
}

fn prop(kind: PropKind, cell: (i32, i32), offset: (f32, f32), facing: Side) -> Prop {
    Prop {
        kind,
        cell: IVec2::new(cell.0, cell.1),
        offset: Vec2::new(offset.0, offset.1),
        facing,
    }
}

fn wall_sign(
    cell: (i32, i32),
    side: Side,
    offset: f32,
    height: f32,
    list: &[Row],
    reader: (i32, i32),
) -> Fixture {
    let mut rows = [None; 3];
    for (slot, row) in rows.iter_mut().zip(list) {
        *slot = Some(*row);
    }
    Fixture::WallSign {
        cell: IVec2::new(cell.0, cell.1),
        side,
        offset,
        height,
        rows,
        reader: IVec2::new(reader.0, reader.1),
    }
}

fn hanger(cell: (i32, i32), facing: Side, shift: f32) -> Fixture {
    Fixture::ExitHanger {
        cell: IVec2::new(cell.0, cell.1),
        facing,
        shift,
    }
}

fn row(dest: Dest, arrow: Arrow) -> Row {
    Row { dest, arrow }
}

pub fn authored() -> Layout {
    use Arrow::Right;
    use Dest::{Boiler, Exit, Maintenance, Office, Storage};
    use Lamp::*;
    use Side::*;
    use WallStyle::*;
    Layout {
        areas: vec![
            area("maintenance", (-5, -9), (-1, -7), Floor::Concrete, Conduit),
            area("exit", (0, -12), (3, -9), Floor::Concrete, Plain),
            area("security", (4, -12), (5, -10), Floor::Concrete, Plain),
            area("service", (0, -8), (0, -7), Floor::Striped, Plain),
            area("office", (1, -8), (3, -7), Floor::Concrete, Plain),
            area("utility", (-5, -6), (-3, -3), Floor::Concrete, Conduit),
            area("hiding", (3, -1), (5, 1), Floor::Concrete, Plain),
            area("west_hall", (-2, -6), (-2, 1), Floor::Concrete, Conduit),
            area("reception", (-1, -6), (1, -4), Floor::Concrete, Plain),
            area("east_hall", (2, -6), (2, 1), Floor::Concrete, Plain),
            area("storage", (3, -6), (5, -4), Floor::Concrete, Plain),
            area("west_link", (-1, -3), (-1, -2), Floor::Concrete, Conduit),
            area("intake", (0, -3), (0, -2), Floor::Striped, Conduit),
            area("east_link", (1, -3), (1, -2), Floor::Concrete, Plain),
            area("lab", (-1, -1), (1, 1), Floor::Concrete, Plain),
            area("boiler", (-5, -1), (-3, 1), Floor::Concrete, Conduit),
        ],
        openings: vec![
            opening((0, -12), North, OpeningKind::Door(Door::Closed)),
            opening((0, -9), South, OpeningKind::Door(Door::Open(75.0))),
            opening((1, -9), South, OpeningKind::Door(Door::Open(75.0))),
            opening((3, -11), East, OpeningKind::Door(Door::Open(75.0))),
            opening((0, -8), West, OpeningKind::Door(Door::Open(75.0))),
            opening((0, -8), East, OpeningKind::Door(Door::Open(75.0))),
            opening((-2, -7), South, OpeningKind::Door(Door::Open(70.0))),
            opening((0, -7), South, OpeningKind::Passage),
            opening((2, -7), South, OpeningKind::Door(Door::Open(70.0))),
            opening((-1, -5), West, OpeningKind::Passage),
            opening((1, -5), East, OpeningKind::Passage),
            opening((2, -5), East, OpeningKind::Door(Door::Open(75.0))),
            opening((0, -4), South, OpeningKind::Passage),
            opening((-2, -4), West, OpeningKind::Door(Door::Open(70.0))),
            opening((-2, -2), East, OpeningKind::Passage),
            opening((-1, -2), East, OpeningKind::Passage),
            opening((0, -2), East, OpeningKind::Passage),
            opening((1, -2), East, OpeningKind::Passage),
            opening((0, -2), South, OpeningKind::Door(Door::Open(75.0))),
            opening((-2, 0), West, OpeningKind::Door(Door::Open(70.0))),
            opening((-1, 0), West, OpeningKind::Door(Door::Open(75.0))),
            opening((1, 0), East, OpeningKind::Door(Door::Open(75.0))),
            opening((2, 0), East, OpeningKind::Door(Door::Open(75.0))),
        ],
        fixtures: vec![
            lamp((0, -11), Cool, true),
            lamp((2, -12), Cool, false),
            lamp((2, -10), Dead, false),
            lamp((4, -11), Cool, false),
            lamp((0, -8), Dead, true),
            lamp((-3, -8), Amber, false),
            lamp((-4, -9), Dead, false),
            lamp((2, -8), Cool, false),
            lamp((-4, -4), Amber, false),
            lamp((4, 0), Flicker, false),
            lamp((-2, -6), Flicker, false),
            lamp((0, -6), Cool, true),
            lamp((0, -4), Cool, true),
            lamp((3, -5), Dead, false),
            lamp((-2, -4), Dead, true),
            lamp((0, -2), Flicker, false),
            lamp((2, -4), Dead, true),
            lamp((-4, 1), Amber, false),
            lamp((0, 1), Flicker, false),
            lamp((2, 1), Dead, false),
            mount((0, -12), North, 0.0, 2.62, Mount::ExitSign),
            mount((2, -12), North, 0.0, 1.85, Mount::FusePanel),
            mount((5, -5), East, 0.0, 2.2, Mount::FaultLamp),
            mount((-5, -1), West, 0.4, 0.45, Mount::Vent),
            mount((-5, -8), West, 0.0, 1.9, Mount::Vent),
            mount((3, -8), East, -0.3, 1.9, Mount::Vent),
            mount((-2, 0), West, 0.0, 2.6, Mount::Label(Boiler)),
            mount((2, -5), East, 0.0, 2.6, Mount::Label(Storage)),
            mount((0, -8), West, 0.0, 2.6, Mount::Label(Maintenance)),
            mount((0, -8), East, 0.0, 2.6, Mount::Label(Office)),
            mount((-5, 0), West, 0.0, 1.6, Mount::PipeManifold),
            mount((-5, 1), South, -0.4, 1.6, Mount::Pegboard),
            mount((-5, 1), West, 0.6, 0.55, Mount::CrawlVent),
            mount((-1, -3), North, 0.3, 1.6, Mount::ClawMarks),
            mount((1, -1), East, 0.2, 1.55, Mount::ClawMarks),
            mount((-5, -9), West, 0.5, 1.6, Mount::ClawMarks),
            wall_sign(
                (-1, -6),
                North,
                0.0,
                2.04,
                &[row(Exit, Right), row(Maintenance, Right)],
                (-1, -6),
            ),
            wall_sign(
                (1, -6),
                East,
                0.0,
                2.04,
                &[row(Storage, Right), row(Office, Right)],
                (1, -6),
            ),
            hanger((0, -7), North, 0.0),
            hanger((0, -5), East, 0.0),
            hanger((0, -3), North, 0.0),
            hanger((-2, -2), North, 0.0),
            hanger((2, -2), North, 0.0),
        ],
        props: vec![
            prop(PropKind::Workbench, (-4, -8), (-0.65, 0.0), East),
            prop(PropKind::Crate, (-4, -9), (-0.3, -0.4), South),
            prop(PropKind::Drum, (-2, -9), (0.6, -0.6), South),
            prop(PropKind::Shelf, (3, -8), (0.75, 0.0), West),
            prop(PropKind::ShelfBins, (3, -7), (0.75, 0.0), West),
            prop(PropKind::Crate, (2, -9), (0.5, -0.5), South),
            prop(PropKind::Shelf, (4, -5), (0.75, 0.0), West),
            prop(PropKind::ShelfBins, (4, -6), (0.75, 0.0), West),
            prop(PropKind::Crate, (3, -6), (-0.3, -0.5), South),
            prop(PropKind::Workbench, (4, -12), (0.0, 0.0), South),
            prop(PropKind::Locker, (4, -10), (0.5, 0.4), West),
            prop(PropKind::Locker, (4, -4), (-0.5, 0.5), North),
            prop(PropKind::Locker, (4, -1), (0.55, 0.3), West),
            prop(PropKind::Locker, (4, 1), (0.55, -0.3), West),
            prop(PropKind::Table, (-1, -4), (0.0, 0.0), South),
            prop(PropKind::Papers, (-1, -4), (0.0, 0.0), North),
            prop(PropKind::ChairTipped, (1, -4), (0.0, 0.0), East),
            prop(PropKind::ShelfBins, (-5, -4), (-0.3, 0.0), East),
            prop(PropKind::Workbench, (-4, -6), (0.0, -0.5), South),
            prop(PropKind::Tools, (-4, -3), (0.0, 0.0), North),
            prop(PropKind::Locker, (-4, -7), (0.0, 0.5), North),
            prop(PropKind::Table, (-3, -9), (0.0, -0.4), South),
            prop(PropKind::ShelfLow, (-1, -7), (0.0, 0.3), West),
            prop(PropKind::Boiler, (-4, 0), (0.0, 0.0), North),
            prop(PropKind::VentGrille, (-5, 1), (-0.55, 0.5), East),
            prop(PropKind::Island, (-3, -8), (0.0, 0.0), North),
            prop(PropKind::ChairTipped, (-4, -7), (0.6, -0.6), North),
            prop(PropKind::DrumSpilled, (-2, -9), (-0.6, 0.5), North),
            prop(PropKind::Tools, (-4, -9), (0.1, 0.7), North),
            prop(PropKind::Papers, (-3, -7), (0.6, -1.0), North),
            prop(PropKind::Drum, (5, -4), (0.7, 0.3), West),
            prop(PropKind::Tank, (0, 0), (0.0, 0.0), East),
            prop(PropKind::Console, (-1, -1), (-0.6, 0.0), East),
            prop(PropKind::ChairTipped, (0, 1), (0.2, 0.0), North),
            prop(PropKind::Papers, (-1, -1), (0.6, 0.6), North),
            prop(PropKind::DragMarks, (1, -1), (0.0, 0.0), East),
        ],
        lines: vec![],
        exit: Some((IVec2::new(0, -12), North)),
        start: Some((IVec2::new(0, -2), North)),
    }
}
