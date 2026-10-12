use bevy::{math::Rect, prelude::*};

use super::{animation::DoorSwing, module::Module};

pub const TILE: f32 = 2.5;

#[derive(Component)]
pub struct LevelRoot;

#[derive(Component)]
pub struct Room(pub Rect);

#[derive(Component)]
pub struct Floor(pub Module);

#[derive(Component)]
pub struct Walls(pub Module);

#[derive(Component)]
pub struct Ceiling(pub Module);

#[derive(Component)]
#[require(DoorSwing)]
pub struct Door {
    pub position: Vec2,
    pub rotation: Quat,
    pub frame: Module,
    pub panel: Module,
    pub state: DoorState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorState {
    Closed,
    Open,
}

#[derive(Component)]
pub struct Passage(pub Vec2);

#[derive(Component)]
#[relationship(relationship_target = Doors)]
pub struct DoorOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = DoorOf)]
pub struct Doors(Vec<Entity>);

#[derive(Component)]
pub struct DoorRef(pub Entity);

pub(crate) fn room(
    name: &'static str,
    bounds: Rect,
    floor: Module,
    walls: Module,
    ceiling: Module,
) -> impl Bundle {
    (
        Name::new(name),
        Room(bounds),
        Floor(floor),
        Walls(walls),
        Ceiling(ceiling),
    )
}

pub(crate) fn door(
    name: &'static str,
    position: Vec2,
    yaw: f32,
    frame: Module,
    panel: Module,
) -> impl Bundle {
    (
        Name::new(name),
        Door {
            position,
            rotation: Quat::from_rotation_y(yaw),
            frame,
            panel,
            state: DoorState::Closed,
        },
    )
}

pub(crate) fn passage(name: &'static str, position: Vec2) -> impl Bundle {
    (Name::new(name), Passage(position))
}

#[derive(Component)]
pub struct Prop(pub Module);

#[derive(Component, Clone, Copy)]
pub struct PropCollider {
    pub center: Vec2,
    pub half: Vec2,
}

#[derive(Component, Clone, Copy)]
pub enum LightEffect {
    Flicker(f32),
    FlickerStrong(f32),
    Pulse,
}

#[derive(Component)]
pub struct LightIntensity(pub f32);

pub(crate) fn prop(name: impl Into<Name>, module: Module, transform: Transform) -> impl Bundle {
    (name.into(), Prop(module), transform, Visibility::default())
}

impl LightEffect {
    pub(crate) fn factor(self, t: f32) -> f32 {
        match self {
            Self::Flicker(phase) => {
                let s = (t * 1.3 + phase).sin() + 0.6 * (t * 4.1 + phase * 2.0).sin();
                if s > 1.1 {
                    0.06
                } else if s > 0.9 {
                    0.5
                } else {
                    1.0
                }
            }
            Self::FlickerStrong(phase) => {
                let s = (t * 2.3 + phase).sin() + 0.65 * (t * 7.7 + phase * 2.0).sin();
                if s > 0.5 {
                    0.03
                } else if s > 0.1 {
                    0.35
                } else {
                    1.0
                }
            }
            Self::Pulse => 0.55 + 0.45 * (t * 2.0).sin(),
        }
    }
}
