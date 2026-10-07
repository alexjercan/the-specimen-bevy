use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::{
    grid::{cell_center, edge_along, edge_key, edge_vertices, wall_face, Side, LAMP_HEIGHT},
    DoorState, Objective,
};

pub const DOOR_SWING: f32 = 75.0;
const ROW_ARROW_X: f32 = 0.67;
const ROW_LABEL_X: f32 = -0.12;
const ROW_PITCH: f32 = 0.24;
const HANGER_ROW: f32 = 2.33;
const HANGER_FACE: f32 = 0.015;

#[derive(Clone, Debug, Default)]
pub struct Level {
    pub rooms: Vec<Area>,
    pub openings: Vec<Opening>,
    pub placements: Vec<Placement>,
    pub start: Option<(IVec2, Side)>,
}

#[derive(Clone, Debug)]
pub struct Area {
    pub name: &'static str,
    pub min: IVec2,
    pub max: IVec2,
    pub floor: &'static str,
    pub wall: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opening {
    pub cell: IVec2,
    pub side: Side,
    pub door: Option<Door>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Door {
    pub state: DoorState,
    pub swing: f32,
    pub objective: Option<Objective>,
}

#[derive(Clone, Debug)]
pub struct Placement {
    pub cell: IVec2,
    pub transform: Transform,
    pub glow: Glow,
    pub overhead: bool,
    pub objective: Option<Objective>,
    pub pieces: Vec<Piece>,
    pub lights: Vec<LightSpec>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub scene: &'static str,
    pub transform: Transform,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightSpec {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Glow {
    Steady,
    Flicker(f32),
    Pulse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lamp {
    Cool,
    Amber,
    Dead,
    Flicker,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrow {
    Left,
    Right,
    Ahead,
    Back,
}

impl Arrow {
    pub fn toward(forward: Side, side: Side) -> Arrow {
        if side == forward {
            Arrow::Ahead
        } else if side == forward.left() {
            Arrow::Left
        } else if side == forward.left().opposite() {
            Arrow::Right
        } else {
            Arrow::Back
        }
    }

    fn roll(self) -> f32 {
        match self {
            Arrow::Left => 0.0,
            Arrow::Right => PI,
            Arrow::Ahead => FRAC_PI_2,
            Arrow::Back => -FRAC_PI_2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EdgeKind {
    Wall(&'static str),
    Passage,
    Door(Door),
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

    fn joint(&self) -> (&'static str, Side) {
        let style = match self.kind {
            EdgeKind::Wall(scene) => scene,
            EdgeKind::Door(_) => "door",
            EdgeKind::Passage => "passage",
        };
        (style, self.facing)
    }
}

impl Area {
    pub fn floor(&mut self, scene: &'static str) -> &mut Self {
        self.floor = scene;
        self
    }

    pub fn walls(&mut self, scene: &'static str) -> &mut Self {
        self.wall = scene;
        self
    }
}

impl Placement {
    fn new(cell: IVec2, transform: Transform) -> Self {
        Self {
            cell,
            transform,
            glow: Glow::Steady,
            overhead: false,
            objective: None,
            pieces: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn scene(&mut self, scene: &'static str, transform: Transform) -> &mut Self {
        self.pieces.push(Piece { scene, transform });
        self
    }

    pub fn light(&mut self, position: Vec3, color: Color, intensity: f32, range: f32) -> &mut Self {
        self.lights.push(LightSpec {
            position,
            color,
            intensity,
            range,
        });
        self
    }

    pub fn glow(&mut self, glow: Glow) -> &mut Self {
        self.glow = glow;
        self
    }

    pub fn objective(&mut self, objective: Objective) -> &mut Self {
        self.objective = Some(objective);
        self
    }

    fn row(&mut self, label: &'static str, arrow: Arrow, center: Vec3, yaw: f32) {
        let rotation = Quat::from_rotation_y(yaw);
        let (arrow_x, label_x) = match arrow {
            Arrow::Right => (-ROW_ARROW_X, -ROW_LABEL_X),
            Arrow::Left | Arrow::Ahead | Arrow::Back => (ROW_ARROW_X, ROW_LABEL_X),
        };
        self.scene(
            label,
            Transform::from_translation(center + rotation * Vec3::X * label_x)
                .with_rotation(rotation),
        )
        .scene(
            "sign_arrow",
            Transform::from_translation(center + rotation * Vec3::X * arrow_x)
                .with_rotation(rotation * Quat::from_rotation_z(arrow.roll())),
        );
    }
}

impl Level {
    pub fn room(
        &mut self,
        name: &'static str,
        min: impl Into<IVec2>,
        max: impl Into<IVec2>,
    ) -> &mut Area {
        self.rooms.push(Area {
            name,
            min: min.into(),
            max: max.into(),
            floor: "floor_tile",
            wall: "wall",
        });
        self.rooms.last_mut().unwrap()
    }

    pub fn passage(&mut self, cell: impl Into<IVec2>, side: Side) {
        self.opening(cell.into(), side, None);
    }

    pub fn door(&mut self, cell: impl Into<IVec2>, side: Side, swing: f32) {
        let door = Door {
            state: DoorState::Open,
            swing,
            objective: None,
        };
        self.opening(cell.into(), side, Some(door));
    }

    pub fn exit(&mut self, cell: impl Into<IVec2>, side: Side) {
        let door = Door {
            state: DoorState::Locked,
            swing: DOOR_SWING,
            objective: Some(Objective::Exit),
        };
        self.opening(cell.into(), side, Some(door));
    }

    fn opening(&mut self, cell: IVec2, side: Side, door: Option<Door>) {
        self.openings.push(Opening { cell, side, door });
    }

    pub fn start(&mut self, cell: impl Into<IVec2>, facing: Side) {
        self.start = Some((cell.into(), facing));
    }

    fn place(&mut self, cell: IVec2, transform: Transform) -> &mut Placement {
        self.placements.push(Placement::new(cell, transform));
        self.placements.last_mut().unwrap()
    }

    pub fn prop(
        &mut self,
        scene: &'static str,
        cell: impl Into<IVec2>,
        offset: impl Into<Vec2>,
        facing: Side,
    ) -> &mut Placement {
        let (cell, offset) = (cell.into(), offset.into());
        let transform =
            Transform::from_translation(cell_center(cell) + Vec3::new(offset.x, 0.0, offset.y))
                .with_rotation(Quat::from_rotation_y(facing.yaw()));
        self.place(cell, transform)
            .scene(scene, Transform::IDENTITY)
    }

    pub fn ceiling_lamp(
        &mut self,
        cell: impl Into<IVec2>,
        lamp: Lamp,
        along_z: bool,
    ) -> &mut Placement {
        let cell = cell.into();
        let yaw = if along_z { FRAC_PI_2 } else { 0.0 };
        let transform = Transform::from_translation(cell_center(cell))
            .with_rotation(Quat::from_rotation_y(yaw));
        let scene = match lamp {
            Lamp::Cool | Lamp::Flicker => "ceiling_light_cool",
            Lamp::Amber => "ceiling_light_amber",
            Lamp::Dead => "ceiling_light_dead",
        };
        let placement = self
            .place(cell, transform)
            .scene(scene, Transform::IDENTITY);
        placement.overhead = true;
        let bulb = Vec3::Y * LAMP_HEIGHT;
        match lamp {
            Lamp::Dead => placement,
            Lamp::Amber => placement.light(bulb, Color::linear_rgb(1.0, 0.58, 0.2), 140_000.0, 9.0),
            Lamp::Cool => placement.light(bulb, Color::linear_rgb(0.78, 0.88, 1.0), 90_000.0, 9.0),
            Lamp::Flicker => placement
                .light(bulb, Color::linear_rgb(0.78, 0.88, 1.0), 90_000.0, 9.0)
                .glow(Glow::Flicker(cell.x as f32 * 1.7 + cell.y as f32 * 0.9)),
        }
    }

    pub fn wall_fixture(
        &mut self,
        scene: &'static str,
        cell: impl Into<IVec2>,
        side: Side,
        offset: f32,
        height: f32,
    ) -> &mut Placement {
        let cell = cell.into();
        let transform =
            Transform::from_translation(wall_face(cell, side, offset) + Vec3::Y * height)
                .with_rotation(Quat::from_rotation_y(side.opposite().yaw()));
        self.place(cell, transform)
            .scene(scene, Transform::IDENTITY)
    }

    pub fn wall_sign(
        &mut self,
        cell: impl Into<IVec2>,
        side: Side,
        offset: f32,
        height: f32,
        rows: &[(&'static str, Arrow)],
    ) -> &mut Placement {
        let cell = cell.into();
        let transform =
            Transform::from_translation(wall_face(cell, side, offset) + Vec3::Y * height)
                .with_rotation(Quat::from_rotation_y(side.opposite().yaw()));
        let placement = self.place(cell, transform);
        for (index, &(label, arrow)) in rows.iter().enumerate() {
            placement.row(label, arrow, Vec3::NEG_Y * (index as f32 * ROW_PITCH), 0.0);
        }
        placement
    }

    pub fn exit_hanger(
        &mut self,
        cell: impl Into<IVec2>,
        facing: Side,
        toward: Side,
    ) -> &mut Placement {
        let cell = cell.into();
        let transform = Transform::from_translation(cell_center(cell))
            .with_rotation(Quat::from_rotation_y(facing.yaw()));
        let placement = self
            .place(cell, transform)
            .scene("sign_hanger", Transform::IDENTITY);
        for (reader, yaw) in [(facing.opposite(), 0.0), (facing, PI)] {
            let center = Quat::from_rotation_y(yaw) * Vec3::new(0.0, HANGER_ROW, -HANGER_FACE);
            placement.row(
                "sign_label_exit",
                Arrow::toward(reader, toward),
                center,
                yaw,
            );
        }
        placement
    }

    pub fn cells(&self) -> HashMap<IVec2, usize> {
        let mut cells = HashMap::new();
        for (index, area) in self.rooms.iter().enumerate() {
            for x in area.min.x..=area.max.x {
                for z in area.min.y..=area.max.y {
                    cells.insert(IVec2::new(x, z), index);
                }
            }
        }
        cells
    }

    pub fn edges(&self, cells: &HashMap<IVec2, usize>) -> HashMap<IVec2, Edge> {
        let openings: HashMap<_, _> = self
            .openings
            .iter()
            .map(|opening| (edge_key(opening.cell, opening.side), opening))
            .collect();
        let mut edges = HashMap::new();
        for (&cell, &area) in cells {
            for side in Side::ALL {
                let neighbor = cells.get(&(cell + side.offset())).copied();
                let key = edge_key(cell, side);
                if neighbor == Some(area) || edges.contains_key(&key) {
                    continue;
                }
                let edge = match openings.get(&key) {
                    Some(opening) => Edge {
                        kind: opening.door.map_or(EdgeKind::Passage, EdgeKind::Door),
                        facing: opening.side.opposite(),
                    },
                    None => {
                        let (owner, facing) = match neighbor {
                            Some(other) if other < area => (other, side),
                            _ => (area, side.opposite()),
                        };
                        Edge {
                            kind: EdgeKind::Wall(self.rooms[owner].wall),
                            facing,
                        }
                    }
                };
                edges.insert(key, edge);
            }
        }
        edges
    }

    pub fn posts(edges: &HashMap<IVec2, Edge>) -> Vec<IVec2> {
        let mut joints: HashMap<IVec2, Vec<(IVec2, &Edge)>> = HashMap::new();
        for (&key, edge) in edges.iter().filter(|(_, edge)| edge.solid()) {
            for vertex in edge_vertices(key) {
                joints.entry(vertex).or_default().push((key, edge));
            }
        }
        joints
            .into_iter()
            .filter(|(vertex, list)| match list.as_slice() {
                [(a, ea), (b, eb)] => {
                    let straight = edge_along(*a) == edge_along(*b);
                    let along = if edge_along(*a) == Vec3::X {
                        vertex.x
                    } else {
                        vertex.y
                    };
                    !straight || ea.joint() != eb.joint() || (along + 1).rem_euclid(4) == 0
                }
                _ => true,
            })
            .map(|(vertex, _)| vertex)
            .collect()
    }
}
