use std::f32::consts::{FRAC_PI_2, PI};

use bevy::math::{IVec2, Quat, Vec3};

pub const TILE: f32 = 2.5;
pub const HALF: f32 = TILE / 2.0;
pub const WALL_T: f32 = 0.2;
pub const POST: f32 = 0.36;
pub const DOOR_W: f32 = 1.2;
pub const DOOR_H: f32 = 2.2;
pub const FRAME_DEPTH: f32 = WALL_T / 2.0 + 0.035;
pub const LAMP_HEIGHT: f32 = 2.7;

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

    pub fn left(self) -> Side {
        match self {
            Side::North => Side::West,
            Side::West => Side::South,
            Side::South => Side::East,
            Side::East => Side::North,
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
}

pub fn cell_center(cell: IVec2) -> Vec3 {
    Vec3::new(cell.x as f32 * TILE, 0.0, cell.y as f32 * TILE)
}

pub fn cell_of(point: Vec3) -> IVec2 {
    IVec2::new(
        (point.x / TILE).round() as i32,
        (point.z / TILE).round() as i32,
    )
}

pub fn half_point(key: IVec2) -> Vec3 {
    Vec3::new(key.x as f32 * HALF, 0.0, key.y as f32 * HALF)
}

pub fn edge_key(cell: IVec2, side: Side) -> IVec2 {
    cell * 2 + side.offset()
}

pub fn edge_along(key: IVec2) -> Vec3 {
    if key.x.rem_euclid(2) == 1 {
        Vec3::Z
    } else {
        Vec3::X
    }
}

pub fn edge_vertices(key: IVec2) -> [IVec2; 2] {
    let along = if key.x.rem_euclid(2) == 1 {
        IVec2::Y
    } else {
        IVec2::X
    };
    [key - along, key + along]
}

pub fn wall_right(facing: Side) -> Vec3 {
    Quat::from_rotation_y(facing.yaw()) * Vec3::X
}

pub fn wall_face(cell: IVec2, side: Side, offset: f32) -> Vec3 {
    let face = side.opposite();
    half_point(edge_key(cell, side)) + face.dir() * (WALL_T / 2.0) + wall_right(face) * offset
}
