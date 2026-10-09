use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::levels::{
    builder::prop,
    fuses::FusePanel,
    lights::LightConfig,
    module_names::{CEILING_LIGHT_COOL, EXIT_SIGN, WALL_LAMP_RED},
};

pub(super) fn spawn(commands: &mut Commands) {
    commands.spawn(prop(
        CEILING_LIGHT_COOL,
        CEILING_LIGHT_COOL,
        Transform::from_xyz(0.0, 0.0, -27.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CEILING_LIGHT_COOL,
        CEILING_LIGHT_COOL,
        Transform::from_xyz(5.0, 0.0, -30.0),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(5.0, 0.0, -25.0),
    ));
    commands.spawn(prop(
        CEILING_LIGHT_COOL,
        CEILING_LIGHT_COOL,
        Transform::from_xyz(10.0, 0.0, -27.5),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(0.0, 0.0, -20.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "ceiling_light_amber",
        "ceiling_light_amber",
        Transform::from_xyz(-7.5, 0.0, -20.0),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(-10.0, 0.0, -22.5),
    ));
    commands.spawn(prop(
        CEILING_LIGHT_COOL,
        CEILING_LIGHT_COOL,
        Transform::from_xyz(5.0, 0.0, -20.0),
    ));
    commands.spawn((
        prop(
            "ceiling_light_amber",
            "ceiling_light_amber",
            Transform::from_xyz(11.25, 0.0, -20.0),
        ),
        LightConfig::intensity(110_000.0),
    ));
    commands.spawn(prop(
        "ceiling_light_amber",
        "ceiling_light_amber",
        Transform::from_xyz(-10.0, 0.0, -10.0),
    ));
    commands.spawn((
        prop(
            CEILING_LIGHT_COOL,
            CEILING_LIGHT_COOL,
            Transform::from_xyz(10.0, 0.0, 0.0),
        ),
        LightConfig::flicker(6.8),
    ));
    commands.spawn((
        prop(
            CEILING_LIGHT_COOL,
            CEILING_LIGHT_COOL,
            Transform::from_xyz(-5.0, 0.0, -15.0),
        ),
        LightConfig::flicker(-8.8),
    ));
    commands.spawn(prop(
        CEILING_LIGHT_COOL,
        CEILING_LIGHT_COOL,
        Transform::from_xyz(0.0, 0.0, -15.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CEILING_LIGHT_COOL,
        CEILING_LIGHT_COOL,
        Transform::from_xyz(0.0, 0.0, -10.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(7.5, 0.0, -12.5),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(-5.0, 0.0, -10.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn((
        prop(
            CEILING_LIGHT_COOL,
            CEILING_LIGHT_COOL,
            Transform::from_xyz(0.0, 0.0, -5.0),
        ),
        LightConfig::flicker(-1.8),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(5.0, 0.0, -10.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "ceiling_light_amber",
        "ceiling_light_amber",
        Transform::from_xyz(-10.0, 0.0, 2.5),
    ));
    commands.spawn((
        prop(
            CEILING_LIGHT_COOL,
            CEILING_LIGHT_COOL,
            Transform::from_xyz(0.0, 0.0, 2.5),
        ),
        LightConfig::flicker(0.9),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(5.0, 0.0, 2.5),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(-5.0, 0.0, 7.5),
    ));
    commands.spawn((
        prop(
            CEILING_LIGHT_COOL,
            CEILING_LIGHT_COOL,
            Transform::from_xyz(-5.0, 0.0, 12.5),
        ),
        LightConfig::flicker(2.4),
    ));
    commands.spawn((
        prop(
            CEILING_LIGHT_COOL,
            CEILING_LIGHT_COOL,
            Transform::from_xyz(5.0, 0.0, 7.5),
        ),
        LightConfig::flicker(-3.7),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(5.0, 0.0, 12.5),
    ));
    commands.spawn(prop(
        "ceiling_light_amber",
        "ceiling_light_amber",
        Transform::from_xyz(-10.0, 0.0, 7.5),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(-10.0, 0.0, 13.75),
    ));
    commands.spawn((
        prop(
            CEILING_LIGHT_COOL,
            CEILING_LIGHT_COOL,
            Transform::from_xyz(0.0, 0.0, 7.5),
        ),
        LightConfig::flicker(4.1),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(0.0, 0.0, 12.5),
    ));
    commands.spawn(prop(
        CEILING_LIGHT_COOL,
        CEILING_LIGHT_COOL,
        Transform::from_xyz(10.0, 0.0, 7.5),
    ));
    commands.spawn(prop(
        "ceiling_light_amber",
        "ceiling_light_amber",
        Transform::from_xyz(10.0, 0.0, 13.75),
    ));
    commands.spawn(prop(
        "ceiling_light_dead",
        "ceiling_light_dead",
        Transform::from_xyz(-7.5, 0.0, 17.5),
    ));
    commands.spawn((
        prop(
            CEILING_LIGHT_COOL,
            CEILING_LIGHT_COOL,
            Transform::from_xyz(7.5, 0.0, 17.5),
        ),
        LightConfig::flicker(-5.3),
    ));
    commands.spawn(prop(
        EXIT_SIGN,
        EXIT_SIGN,
        Transform::from_xyz(0.0, 2.62, -31.15).with_rotation(Quat::from_rotation_y(PI)),
    ));
    let fuse_panel =
        Transform::from_xyz(5.0, 1.85, -31.15).with_rotation(Quat::from_rotation_y(PI));
    commands.spawn(prop("fuse_panel", "fuse_panel", fuse_panel));
    commands.spawn((Name::new("fuse panel"), FusePanel::default(), fuse_panel));
    commands.spawn(prop(
        WALL_LAMP_RED,
        WALL_LAMP_RED,
        Transform::from_xyz(13.65, 2.2, -12.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
}
