use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::levels::builder::prop;

pub(super) fn spawn(commands: &mut Commands) {
    commands
        .spawn((
            Transform::from_xyz(-2.5, 2.04, -16.15).with_rotation(Quat::from_rotation_y(PI)),
            Visibility::default(),
        ))
        .with_children(|children| {
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(0.12, 0.0, 0.0),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(-0.67, 0.0, 0.0).with_rotation(Quat::from_rotation_z(PI)),
            ));
            children.spawn(prop(
                "sign_label_maintenance",
                "sign_label_maintenance",
                Transform::from_xyz(0.12, -0.24, 0.0),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(-0.67, -0.24, 0.0).with_rotation(Quat::from_rotation_z(PI)),
            ));
        });
    commands
        .spawn((
            Transform::from_xyz(3.65, 2.04, -15.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
            Visibility::default(),
        ))
        .with_children(|children| {
            children.spawn(prop(
                "sign_label_storage",
                "sign_label_storage",
                Transform::from_xyz(0.12, 0.0, 0.0),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(-0.67, 0.0, 0.0).with_rotation(Quat::from_rotation_z(PI)),
            ));
            children.spawn(prop(
                "sign_label_office",
                "sign_label_office",
                Transform::from_xyz(0.12, -0.24, 0.0),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(-0.67, -0.24, 0.0).with_rotation(Quat::from_rotation_z(PI)),
            ));
        });
    commands
        .spawn(prop(
            "sign_hanger",
            "sign_hanger",
            Transform::from_xyz(0.0, 0.0, -17.5),
        ))
        .with_children(|children| {
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(0.12, 2.33, -0.015),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(-0.67, 2.33, -0.015)
                    .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            ));
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(-0.12, 2.33, 0.015).with_rotation(Quat::from_rotation_y(PI)),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(0.67, 2.33, 0.015)
                    .with_rotation(Quat::from_rotation_y(PI) * Quat::from_rotation_z(FRAC_PI_2)),
            ));
        });
    commands
        .spawn(prop(
            "sign_hanger",
            "sign_hanger",
            Transform::from_xyz(0.0, 0.0, -12.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
        ))
        .with_children(|children| {
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(0.12, 2.33, -0.015),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(-0.67, 2.33, -0.015).with_rotation(Quat::from_rotation_z(PI)),
            ));
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(-0.12, 2.33, 0.015).with_rotation(Quat::from_rotation_y(PI)),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(0.67, 2.33, 0.015).with_rotation(Quat::from_rotation_y(PI)),
            ));
        });
    commands
        .spawn(prop(
            "sign_hanger",
            "sign_hanger",
            Transform::from_xyz(0.0, 0.0, -7.5),
        ))
        .with_children(|children| {
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(0.12, 2.33, -0.015),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(-0.67, 2.33, -0.015)
                    .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            ));
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(-0.12, 2.33, 0.015).with_rotation(Quat::from_rotation_y(PI)),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(0.67, 2.33, 0.015)
                    .with_rotation(Quat::from_rotation_y(PI) * Quat::from_rotation_z(FRAC_PI_2)),
            ));
        });
    commands
        .spawn(prop(
            "sign_hanger",
            "sign_hanger",
            Transform::from_xyz(-5.0, 0.0, -5.0),
        ))
        .with_children(|children| {
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(0.12, 2.33, -0.015),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(-0.67, 2.33, -0.015)
                    .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            ));
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(-0.12, 2.33, 0.015).with_rotation(Quat::from_rotation_y(PI)),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(0.67, 2.33, 0.015)
                    .with_rotation(Quat::from_rotation_y(PI) * Quat::from_rotation_z(FRAC_PI_2)),
            ));
        });
    commands
        .spawn(prop(
            "sign_hanger",
            "sign_hanger",
            Transform::from_xyz(5.0, 0.0, -5.0),
        ))
        .with_children(|children| {
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(0.12, 2.33, -0.015),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(-0.67, 2.33, -0.015)
                    .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            ));
            children.spawn(prop(
                "sign_label_exit",
                "sign_label_exit",
                Transform::from_xyz(-0.12, 2.33, 0.015).with_rotation(Quat::from_rotation_y(PI)),
            ));
            children.spawn(prop(
                "sign arrow",
                "sign_arrow",
                Transform::from_xyz(0.67, 2.33, 0.015)
                    .with_rotation(Quat::from_rotation_y(PI) * Quat::from_rotation_z(FRAC_PI_2)),
            ));
        });
}
