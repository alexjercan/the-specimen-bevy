use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::levels::builder::{light, prop, LightEffect};

use super::{FIRE, SPECIMEN};

pub(super) fn spawn_fixtures(commands: &mut Commands) {
    commands.spawn(prop(
        "wall_vent",
        "wall_vent",
        Transform::from_xyz(-13.65, 0.45, -1.1).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "wall_vent",
        "wall_vent",
        Transform::from_xyz(-13.65, 1.9, -20.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "wall_vent",
        "wall_vent",
        Transform::from_xyz(13.65, 1.9, -21.25).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "sign_label_boiler_room",
        "sign_label_boiler_room",
        Transform::from_xyz(-6.10, 2.6, 0.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "sign_label_storage",
        "sign_label_storage",
        Transform::from_xyz(6.15, 2.6, -12.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "sign_label_maintenance",
        "sign_label_maintenance",
        Transform::from_xyz(-1.15, 2.6, -20.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "sign_label_office",
        "sign_label_office",
        Transform::from_xyz(1.15, 2.6, -17.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "sign_label_lab",
        "sign_label_lab",
        Transform::from_xyz(3.85, 2.6, 0.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "sign_label_security",
        "sign_label_security",
        Transform::from_xyz(8.65, 2.6, -27.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "sign_label_prep",
        "sign_label_prep",
        Transform::from_xyz(6.15, 2.6, -5.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "pipe_manifold",
        "pipe_manifold",
        Transform::from_xyz(-13.65, 1.6, 0.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "tool_pegboard",
        "tool_pegboard",
        Transform::from_xyz(-13.64, 1.6, 2.3).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "concept_crawl_vent",
        "concept_crawl_vent",
        Transform::from_xyz(-13.65, 0.55, 3.1).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "trace_claw_marks",
        "trace_claw_marks",
        Transform::from_xyz(-2.2, 1.6, -8.65).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "trace_claw_marks",
        "trace_claw_marks",
        Transform::from_xyz(3.65, 1.55, -2.3).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "trace_claw_marks",
        "trace_claw_marks",
        Transform::from_xyz(-13.65, 1.6, -22.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
}

pub(super) fn spawn_furniture(commands: &mut Commands) {
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(-10.65, 0.0, -20.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(-10.3, 0.0, -22.9).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "steel_drum",
        "steel_drum",
        Transform::from_xyz(-4.4, 0.0, -23.1).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit",
        "shelf_unit",
        Transform::from_xyz(13.35, 0.0, -18.75).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "shelf_unit_bins",
        "shelf_unit_bins",
        Transform::from_xyz(8.4, 0.0, -17.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(2.5, 0.0, -26.25).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit",
        "shelf_unit",
        Transform::from_xyz(10.75, 0.0, -12.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "shelf_unit_bins",
        "shelf_unit_bins",
        Transform::from_xyz(10.75, 0.0, -15.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(7.2, 0.0, -15.5).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(8.3, 0.0, -23.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(10.0, 0.0, -22.5),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(10.0, 0.0, -30.0).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "concept_locker",
        "concept_locker",
        Transform::from_xyz(10.5, 0.0, -24.15).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(12.5, 0.0, -30.0).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "concept_locker",
        "concept_locker",
        Transform::from_xyz(9.5, 0.0, -9.15).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "concept_locker",
        "concept_locker",
        Transform::from_xyz(13.35, 0.0, -2.2).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "concept_locker",
        "concept_locker",
        Transform::from_xyz(13.35, 0.0, 2.2).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "concept_table",
        "concept_table",
        Transform::from_xyz(9.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "concept_table",
        "concept_table",
        Transform::from_xyz(-2.5, 0.0, -10.0).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "clutter_papers",
        "clutter_papers",
        Transform::from_xyz(-2.5, 0.0, -10.0).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "chair_tipped",
        "chair_tipped",
        Transform::from_xyz(2.5, 0.0, -10.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(-2.5, 0.0, -15.85),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(2.5, 0.0, -15.85),
    ));
    commands.spawn(prop(
        "chair_tipped",
        "chair_tipped",
        Transform::from_xyz(-2.3, 0.0, -13.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "chair_tipped",
        "chair_tipped",
        Transform::from_xyz(2.3, 0.0, -13.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "clutter_papers",
        "clutter_papers",
        Transform::from_xyz(2.3, 0.0, -14.5).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit_bins",
        "shelf_unit_bins",
        Transform::from_xyz(-2.5, 0.0, -8.35),
    ));
    commands.spawn(prop(
        "shelf_unit",
        "shelf_unit",
        Transform::from_xyz(2.5, 0.0, -8.35).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit_bins",
        "shelf_unit_bins",
        Transform::from_xyz(-13.35, 0.0, -10.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(-13.3, 0.0, -13.2).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(-10.0, 0.0, -15.5).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "clutter_tools",
        "clutter_tools",
        Transform::from_xyz(-10.0, 0.0, -7.5).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "concept_locker",
        "concept_locker",
        Transform::from_xyz(-10.0, 0.0, -16.65).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "concept_table",
        "concept_table",
        Transform::from_xyz(-7.5, 0.0, -22.9).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(-2.5, 0.0, -17.2).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "concept_table",
        "concept_table",
        Transform::from_xyz(3.75, 0.0, -18.75).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "vent_grille",
        "vent_grille",
        Transform::from_xyz(-13.05, 0.0, 3.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "work_island",
        "work_island",
        Transform::from_xyz(-7.5, 0.0, -20.0).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "chair_tipped",
        "chair_tipped",
        Transform::from_xyz(-9.4, 0.0, -18.1).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "drum_spilled",
        "drum_spilled",
        Transform::from_xyz(-5.6, 0.0, -22.0).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "clutter_tools",
        "clutter_tools",
        Transform::from_xyz(-9.9, 0.0, -21.8).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "clutter_papers",
        "clutter_papers",
        Transform::from_xyz(-6.9, 0.0, -18.5).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "steel_drum",
        "steel_drum",
        Transform::from_xyz(13.2, 0.0, -9.7).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(7.4, 0.0, -29.8).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "lab_console",
        "lab_console",
        Transform::from_xyz(-3.1, 0.0, -2.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "chair_tipped",
        "chair_tipped",
        Transform::from_xyz(0.2, 0.0, 2.5).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "clutter_papers",
        "clutter_papers",
        Transform::from_xyz(-1.9, 0.0, -1.9).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "trace_drag_marks",
        "trace_drag_marks",
        Transform::from_xyz(2.5, 0.0, -2.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "steel_drum",
        "steel_drum",
        Transform::from_xyz(-12.5, 0.0, -2.5).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(-7.4, 0.0, 2.6).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands
        .spawn((
            prop(
                "boiler_unit",
                "boiler_unit",
                Transform::from_xyz(-10.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(0.0)),
            ),
            LightEffect::Flicker(-7.0),
        ))
        .with_children(|children| {
            children.spawn((
                light(
                    "light",
                    Transform::from_translation(Vec3::new(0.0, 0.5, -0.85)),
                    FIRE,
                    45_000.0,
                    7.0,
                ),
                LightEffect::Flicker(-7.0),
            ));
        });
    commands
        .spawn((
            prop(
                "concept_containment_tank",
                "concept_containment_tank",
                Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
            ),
            LightEffect::Flicker(0.0),
        ))
        .with_children(|children| {
            children.spawn((
                light(
                    "light",
                    Transform::from_translation(Vec3::new(0.0, 2.1, 0.0)),
                    SPECIMEN,
                    12_000.0,
                    4.5,
                ),
                LightEffect::Flicker(0.0),
            ));
        });
}
