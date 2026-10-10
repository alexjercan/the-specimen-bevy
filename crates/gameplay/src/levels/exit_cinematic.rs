use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{math::Rect, prelude::*};

use super::{
    builder::{room, Door, DoorOf, DoorRef, DoorState, LevelRoot},
    exterior::{spawn_details, EXIT_CORRIDOR},
    module_names::{DOOR_FRAME, DOOR_PANEL},
    DoorSwing,
};

pub struct ExitCinematic {
    pub root: Entity,
    pub door: Entity,
    pub view: Transform,
}

pub fn build_exit_cinematic(commands: &mut Commands) -> ExitCinematic {
    build_exit_cinematic_with_door(commands, DoorState::Open)
}

pub fn build_closed_exit_cinematic(commands: &mut Commands) -> ExitCinematic {
    build_exit_cinematic_with_door(commands, DoorState::Closed)
}

fn build_exit_cinematic_with_door(commands: &mut Commands, state: DoorState) -> ExitCinematic {
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
                state,
            },
            DoorSwing(if state == DoorState::Open {
                FRAC_PI_2
            } else {
                0.0
            }),
            ChildOf(root),
        ))
        .id();

    commands
        .spawn((
            room(
                "exit cinematic exterior corridor",
                EXIT_CORRIDOR,
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

    spawn_details(commands, root);

    let view =
        Transform::from_xyz(0.6, 1.55, -37.2).looking_at(Vec3::new(0.0, 1.5, -31.25), Vec3::Y);

    ExitCinematic { root, door, view }
}
