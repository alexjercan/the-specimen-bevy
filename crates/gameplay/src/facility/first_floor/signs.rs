use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::facility::{
    builder::{prop, LevelRoot},
    module::Module,
};

const EXIT: Module = Module::SignLabelExit;
const LAB: Module = Module::SignLabelLab;
const STORAGE: Module = Module::SignLabelStorage;
const OFFICE: Module = Module::SignLabelOffice;
const MAINTENANCE: Module = Module::SignLabelMaintenance;
const BOILER_ROOM: Module = Module::SignLabelBoilerRoom;
const HANGER_HEIGHT: f32 = 2.33;
const HANGER_FACE_OFFSET: f32 = 0.015;
const ROW_SPACING: f32 = 0.24;

#[derive(Clone, Copy)]
enum Arrow {
    Ahead,
    Left,
    Right,
}

impl Arrow {
    fn turn(self) -> f32 {
        match self {
            Self::Ahead => FRAC_PI_2,
            Self::Left => 0.0,
            Self::Right => PI,
        }
    }
}

type Row = (Module, Arrow);

pub(super) fn spawn(commands: &mut Commands) {
    wall_sign(
        commands,
        Transform::from_xyz(-2.5, 2.04, -16.15).with_rotation(Quat::from_rotation_y(PI)),
        &[(EXIT, Arrow::Right), (MAINTENANCE, Arrow::Right)],
    );
    wall_sign(
        commands,
        Transform::from_xyz(3.65, 2.04, -15.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        &[(STORAGE, Arrow::Right), (OFFICE, Arrow::Right)],
    );
    wall_sign(
        commands,
        Transform::from_xyz(2.5, 2.04, -21.35),
        &[(OFFICE, Arrow::Left), (MAINTENANCE, Arrow::Right)],
    );
    wall_sign(
        commands,
        Transform::from_xyz(0.0, 2.04, 18.65),
        &[(EXIT, Arrow::Right), (EXIT, Arrow::Left)],
    );
    hanger(
        commands,
        Transform::from_xyz(0.0, 0.0, -17.5),
        Some((STORAGE, Arrow::Ahead)),
        Some((EXIT, Arrow::Ahead)),
    );
    hanger(
        commands,
        Transform::from_xyz(0.0, 0.0, -12.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
        Some((EXIT, Arrow::Right)),
        Some((EXIT, Arrow::Left)),
    );
    hanger(
        commands,
        Transform::from_xyz(0.0, 0.0, -7.5),
        Some((LAB, Arrow::Ahead)),
        Some((EXIT, Arrow::Ahead)),
    );
    hanger(
        commands,
        Transform::from_xyz(-5.0, 0.0, -5.0),
        Some((BOILER_ROOM, Arrow::Ahead)),
        Some((EXIT, Arrow::Ahead)),
    );
    hanger(
        commands,
        Transform::from_xyz(5.0, 0.0, -5.0),
        Some((LAB, Arrow::Ahead)),
        Some((EXIT, Arrow::Ahead)),
    );
    hanger(
        commands,
        Transform::from_xyz(-5.0, 0.0, 1.0),
        None,
        Some((EXIT, Arrow::Ahead)),
    );
    hanger(
        commands,
        Transform::from_xyz(5.0, 0.0, 1.0),
        None,
        Some((EXIT, Arrow::Ahead)),
    );
    hanger(
        commands,
        Transform::from_xyz(-5.0, 0.0, 15.0),
        None,
        Some((EXIT, Arrow::Ahead)),
    );
    hanger(
        commands,
        Transform::from_xyz(5.0, 0.0, 15.0),
        None,
        Some((EXIT, Arrow::Ahead)),
    );
}

fn row(label: Module, arrow: Arrow, face: Transform, height: f32) -> [impl Bundle; 2] {
    [
        prop(
            format!("{label:?}"),
            label,
            face * Transform::from_xyz(0.12, height, 0.0),
        ),
        prop(
            "sign arrow".to_owned(),
            Module::SignArrow,
            face * Transform::from_xyz(-0.67, height, 0.0)
                .with_rotation(Quat::from_rotation_z(arrow.turn())),
        ),
    ]
}

fn wall_sign(commands: &mut Commands, transform: Transform, rows: &[Row]) {
    commands
        .spawn((transform, Visibility::default(), LevelRoot))
        .with_children(|children| {
            for (index, &(label, arrow)) in rows.iter().enumerate() {
                let height = -ROW_SPACING * index as f32;
                for bundle in row(label, arrow, Transform::IDENTITY, height) {
                    children.spawn(bundle);
                }
            }
        });
}

fn hanger(commands: &mut Commands, transform: Transform, front: Option<Row>, back: Option<Row>) {
    let faces = [
        (front, Transform::from_xyz(0.0, 0.0, -HANGER_FACE_OFFSET)),
        (
            back,
            Transform::from_xyz(0.0, 0.0, HANGER_FACE_OFFSET)
                .with_rotation(Quat::from_rotation_y(PI)),
        ),
    ];
    commands
        .spawn(prop("sign_hanger", Module::SignHanger, transform))
        .with_children(|children| {
            for (sign, face) in faces {
                if let Some((label, arrow)) = sign {
                    for bundle in row(label, arrow, face, HANGER_HEIGHT) {
                        children.spawn(bundle);
                    }
                }
            }
        });
}
