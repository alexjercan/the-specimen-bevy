use bevy::{math::Rect, prelude::*};

use super::animation::DoorSwing;

#[derive(Component)]
pub struct Room(pub Rect);

#[derive(Component)]
pub struct Floor(pub String);

#[derive(Component)]
pub struct Walls(pub String);

#[derive(Component)]
pub struct Ceiling(pub String);

#[derive(Component)]
#[require(DoorSwing)]
pub struct Door {
    pub position: Vec2,
    pub rotation: Quat,
    pub frame: String,
    pub panel: String,
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
    floor: &str,
    walls: &str,
    ceiling: &str,
) -> impl Bundle {
    (
        Name::new(name),
        Room(bounds),
        Floor(floor.to_owned()),
        Walls(walls.to_owned()),
        Ceiling(ceiling.to_owned()),
    )
}

pub(crate) fn door(
    name: &'static str,
    position: Vec2,
    yaw: f32,
    frame: &str,
    panel: &str,
) -> impl Bundle {
    (
        Name::new(name),
        Door {
            position,
            rotation: Quat::from_rotation_y(yaw),
            frame: frame.to_owned(),
            panel: panel.to_owned(),
            state: DoorState::Closed,
        },
    )
}

pub(crate) fn passage(name: &'static str, position: Vec2) -> impl Bundle {
    (Name::new(name), Passage(position))
}

#[derive(Component)]
pub struct Prop(pub String);

#[derive(Component, Clone, Copy)]
pub struct PropCollider {
    pub center: Vec2,
    pub half: Vec2,
}

#[derive(Component, Clone, Copy)]
pub enum LightEffect {
    Flicker(f32),
    Pulse,
}

#[derive(Component)]
pub struct LightIntensity(pub f32);

pub(crate) fn prop(name: &'static str, module: &str, transform: Transform) -> impl Bundle {
    (
        Name::new(name),
        Prop(module.to_owned()),
        transform,
        Visibility::default(),
    )
}

pub(crate) fn light(
    name: &'static str,
    transform: Transform,
    color: Color,
    intensity: f32,
    range: f32,
) -> impl Bundle {
    (
        Name::new(name),
        PointLight {
            color,
            intensity,
            range,
            radius: 0.08,
            shadow_maps_enabled: false,
            ..default()
        },
        LightIntensity(intensity),
        transform,
    )
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
            Self::Pulse => 0.55 + 0.45 * (t * 2.0).sin(),
        }
    }
}
