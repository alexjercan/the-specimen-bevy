use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::levels::{
    builder::{light, prop, LightEffect},
    fuses::FusePanel,
    module_names::{EXIT_SIGN, WALL_LAMP_RED},
};

use super::{AMBER, COOL, EXIT_GREEN, FAULT_RED};

pub(super) fn spawn(commands: &mut Commands) {
    commands
        .spawn(prop(
            "ceiling_light_cool",
            "ceiling_light_cool",
            Transform::from_xyz(0.0, 0.0, -27.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                COOL,
                90_000.0,
                9.0,
            ));
        });
    commands
        .spawn(prop(
            "ceiling_light_cool",
            "ceiling_light_cool",
            Transform::from_xyz(5.0, 0.0, -30.0).with_rotation(Quat::from_rotation_y(0.0)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                COOL,
                90_000.0,
                9.0,
            ));
        });
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(5.0, 0.0, -25.0).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands
        .spawn(prop(
            "ceiling_light_cool",
            "ceiling_light_cool",
            Transform::from_xyz(10.0, 0.0, -27.5).with_rotation(Quat::from_rotation_y(0.0)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                COOL,
                90_000.0,
                9.0,
            ));
        });
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(0.0, 0.0, -20.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands
        .spawn(prop(
            "ceiling_light_amber",
            "ceiling_light_amber",
            Transform::from_xyz(-7.5, 0.0, -20.0).with_rotation(Quat::from_rotation_y(0.0)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                AMBER,
                140_000.0,
                9.0,
            ));
        });
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(-10.0, 0.0, -22.5).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands
        .spawn(prop(
            "ceiling_light_cool",
            "ceiling_light_cool",
            Transform::from_xyz(5.0, 0.0, -20.0).with_rotation(Quat::from_rotation_y(0.0)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                COOL,
                90_000.0,
                9.0,
            ));
        });
    commands
        .spawn(prop(
            "ceiling_light_amber",
            "ceiling_light_amber",
            Transform::from_xyz(11.25, 0.0, -20.0),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                AMBER,
                110_000.0,
                7.0,
            ));
        });
    commands
        .spawn(prop(
            "ceiling_light_amber",
            "ceiling_light_amber",
            Transform::from_xyz(-10.0, 0.0, -10.0).with_rotation(Quat::from_rotation_y(0.0)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                AMBER,
                140_000.0,
                9.0,
            ));
        });
    commands
        .spawn((
            prop(
                "ceiling_light_cool",
                "ceiling_light_cool",
                Transform::from_xyz(10.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(0.0)),
            ),
            LightEffect::Flicker(6.8),
        ))
        .with_children(|children| {
            children.spawn((
                light(
                    "light",
                    Transform::from_translation(Vec3::Y * 2.7),
                    COOL,
                    90_000.0,
                    9.0,
                ),
                LightEffect::Flicker(6.8),
            ));
        });
    commands
        .spawn((
            prop(
                "ceiling_light_cool",
                "ceiling_light_cool",
                Transform::from_xyz(-5.0, 0.0, -15.0).with_rotation(Quat::from_rotation_y(0.0)),
            ),
            LightEffect::Flicker(-8.8),
        ))
        .with_children(|children| {
            children.spawn((
                light(
                    "light",
                    Transform::from_translation(Vec3::Y * 2.7),
                    COOL,
                    90_000.0,
                    9.0,
                ),
                LightEffect::Flicker(-8.8),
            ));
        });
    commands
        .spawn(prop(
            "ceiling_light_cool",
            "ceiling_light_cool",
            Transform::from_xyz(0.0, 0.0, -15.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                COOL,
                90_000.0,
                9.0,
            ));
        });
    commands
        .spawn(prop(
            "ceiling_light_cool",
            "ceiling_light_cool",
            Transform::from_xyz(0.0, 0.0, -10.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                COOL,
                90_000.0,
                9.0,
            ));
        });
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(7.5, 0.0, -12.5).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(-5.0, 0.0, -10.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands
        .spawn((
            prop(
                "ceiling_light_cool",
                "ceiling_light_cool",
                Transform::from_xyz(0.0, 0.0, -5.0).with_rotation(Quat::from_rotation_y(0.0)),
            ),
            LightEffect::Flicker(-1.8),
        ))
        .with_children(|children| {
            children.spawn((
                light(
                    "light",
                    Transform::from_translation(Vec3::Y * 2.7),
                    COOL,
                    90_000.0,
                    9.0,
                ),
                LightEffect::Flicker(-1.8),
            ));
        });
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(5.0, 0.0, -10.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands
        .spawn(prop(
            "ceiling_light_amber",
            "ceiling_light_amber",
            Transform::from_xyz(-10.0, 0.0, 2.5).with_rotation(Quat::from_rotation_y(0.0)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::Y * 2.7),
                AMBER,
                140_000.0,
                9.0,
            ));
        });
    commands
        .spawn((
            prop(
                "ceiling_light_cool",
                "ceiling_light_cool",
                Transform::from_xyz(0.0, 0.0, 2.5).with_rotation(Quat::from_rotation_y(0.0)),
            ),
            LightEffect::Flicker(0.9),
        ))
        .with_children(|children| {
            children.spawn((
                light(
                    "light",
                    Transform::from_translation(Vec3::Y * 2.7),
                    COOL,
                    90_000.0,
                    9.0,
                ),
                LightEffect::Flicker(0.9),
            ));
        });
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(5.0, 0.0, 2.5).with_rotation(Quat::from_rotation_y(0.0)),
    ));

    commands
        .spawn(prop(
            EXIT_SIGN,
            EXIT_SIGN,
            Transform::from_xyz(0.0, 2.62, -31.15).with_rotation(Quat::from_rotation_y(PI)),
        ))
        .with_children(|children| {
            children.spawn(light(
                "light",
                Transform::from_translation(Vec3::NEG_Z * 0.2),
                EXIT_GREEN,
                6_000.0,
                5.0,
            ));
        });
    let fuse_panel =
        Transform::from_xyz(5.0, 1.85, -31.15).with_rotation(Quat::from_rotation_y(PI));
    commands.spawn(prop("fuse_panel", "fuse_panel", fuse_panel));
    commands.spawn((Name::new("fuse panel"), FusePanel::default(), fuse_panel));
    commands
        .spawn((
            prop(
                WALL_LAMP_RED,
                WALL_LAMP_RED,
                Transform::from_xyz(13.65, 2.2, -12.5)
                    .with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
            ),
            LightEffect::Pulse,
        ))
        .with_children(|children| {
            children.spawn((
                light(
                    "light",
                    Transform::from_translation(Vec3::NEG_Z * 0.25),
                    FAULT_RED,
                    60_000.0,
                    5.0,
                ),
                LightEffect::Pulse,
            ));
        });
}
