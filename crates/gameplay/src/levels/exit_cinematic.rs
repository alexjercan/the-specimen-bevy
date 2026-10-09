use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{math::Rect, prelude::*};

use super::{
    builder::{prop, room, Door, DoorOf, DoorRef, DoorState, LevelRoot, LightEffect},
    lights::LightConfig,
    module_names::{DOOR_FRAME, DOOR_PANEL, EXIT_SIGN, WALL_LAMP_RED},
    DoorSwing,
};

pub struct ExitCinematic {
    pub root: Entity,
    pub door: Entity,
    pub view: Transform,
}

pub fn build_exit_cinematic(commands: &mut Commands) -> ExitCinematic {
    const FLOOR_TILE: &str = "floor_tile";
    const WALL: &str = "wall";
    const CEILING_TILE: &str = "ceiling_tile";

    let root = commands
        .spawn((
            Name::new("Exit cinematic"),
            LevelRoot,
            Transform::IDENTITY,
            Visibility::default(),
        ))
        .id();

    let door = commands
        .spawn((
            Name::new("Exit cinematic door"),
            Door {
                position: Vec2::new(0.0, -31.25),
                rotation: Quat::from_rotation_y(PI),
                frame: DOOR_FRAME.to_owned(),
                panel: DOOR_PANEL.to_owned(),
                state: DoorState::Open,
            },
            DoorSwing(FRAC_PI_2),
            ChildOf(root),
        ))
        .id();

    commands
        .spawn((
            room(
                "yard",
                Rect::new(-3.75, -38.75, 3.75, -31.25),
                FLOOR_TILE,
                WALL,
                CEILING_TILE,
            ),
            ChildOf(root),
        ))
        .with_related::<DoorOf>((DoorRef(door), ChildOf(root)));
    commands
        .spawn((
            room(
                "exit cinematic corridor",
                Rect::new(-1.25, -31.25, 1.25, -26.25),
                FLOOR_TILE,
                WALL,
                CEILING_TILE,
            ),
            ChildOf(root),
        ))
        .with_related::<DoorOf>((DoorRef(door), ChildOf(root)));

    commands.spawn((
        prop(
            "Exit cinematic lamp",
            WALL_LAMP_RED,
            Transform::from_xyz(1.0, 2.2, -31.35),
        ),
        LightConfig {
            intensity: Some(18_000.0),
            effect: Some(LightEffect::Pulse),
            ..default()
        },
        ChildOf(root),
    ));
    commands.spawn((
        prop(
            "Exit cinematic sign",
            EXIT_SIGN,
            Transform::from_xyz(0.0, 2.62, -31.35),
        ),
        ChildOf(root),
    ));

    let view =
        Transform::from_xyz(1.2, 1.55, -37.2).looking_at(Vec3::new(0.0, 1.5, -31.25), Vec3::Y);

    ExitCinematic { root, door, view }
}
