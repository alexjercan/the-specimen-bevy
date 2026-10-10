use std::f32::consts::{FRAC_PI_2, PI};

use crate::levels::{
    builder::prop,
    module_names::{
        BOILER_UNIT, CONCEPT_CONTAINMENT_TANK, CONCEPT_LOCKER, CONCEPT_TABLE, PIPE_MANIFOLD,
        WALL_VENT,
    },
    pickups::PickupKind,
};
use bevy::prelude::*;

pub(super) fn spawn_fixtures(commands: &mut Commands) {
    commands.spawn(prop(
        WALL_VENT,
        WALL_VENT,
        Transform::from_xyz(-13.65, 0.45, -1.1).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        WALL_VENT,
        WALL_VENT,
        Transform::from_xyz(-13.65, 1.9, -20.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        WALL_VENT,
        WALL_VENT,
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
        "sign_label_utility",
        "sign_label_utility",
        Transform::from_xyz(-6.15, 2.6, -10.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
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
        PIPE_MANIFOLD,
        PIPE_MANIFOLD,
        Transform::from_xyz(-13.65, 1.6, 0.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "tool_pegboard",
        "tool_pegboard",
        Transform::from_xyz(-13.64, 1.6, 2.3).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    spawn_crawl_vent(
        commands,
        Transform::from_xyz(-13.65, 0.55, 3.1).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
        true,
    );
    spawn_crawl_vent(
        commands,
        Transform::from_xyz(-13.65, 0.55, -8.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
        false,
    );
    spawn_crawl_vent(
        commands,
        Transform::from_xyz(13.65, 0.55, -4.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        true,
    );
    commands.spawn(prop(
        "concept_crawl_vent",
        "concept_crawl_vent",
        Transform::from_xyz(-13.65, 0.55, 17.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
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
    commands.spawn(prop(
        "trace_claw_marks",
        "trace_claw_marks",
        Transform::from_xyz(-3.85, 1.6, 13.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "trace_claw_marks",
        "trace_claw_marks",
        Transform::from_xyz(12.05, 1.5, 11.35).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "tool_pegboard",
        "tool_pegboard",
        Transform::from_xyz(-13.64, 1.6, 8.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "tool_pegboard",
        "tool_pegboard",
        Transform::from_xyz(-13.64, 1.6, 17.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "trace_claw_marks",
        "trace_claw_marks",
        Transform::from_xyz(13.65, 1.5, 17.2).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "trace_claw_marks",
        "trace_claw_marks",
        Transform::from_xyz(-3.0, 1.6, 18.65).with_rotation(Quat::from_rotation_y(PI)),
    ));
}

pub(super) fn spawn_furniture(commands: &mut Commands) {
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(-13.0, 0.0, -21.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
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
    spawn_composition(
        commands,
        "shelf_unit_bins",
        Transform::from_xyz(8.1, 0.0, -17.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
        &[("shelf_bin_contents", Transform::IDENTITY)],
    );
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(2.5, 0.0, -26.25).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit_alt",
        "shelf_unit_alt",
        Transform::from_xyz(10.75, 0.0, -12.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    spawn_composition(
        commands,
        "shelf_unit_bins",
        Transform::from_xyz(10.75, 0.0, -14.8).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
        &[("shelf_bin_contents", Transform::IDENTITY)],
    );
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
        "storage_crate_open",
        "storage_crate_open",
        Transform::from_xyz(10.0, 0.0, -22.5).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(10.0, 0.0, -30.65).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "file_cabinet_open",
        "file_cabinet_open",
        Transform::from_xyz(13.4, 0.0, -26.25).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CONCEPT_LOCKER,
        CONCEPT_LOCKER,
        Transform::from_xyz(10.5, 0.0, -24.15).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(12.5, 0.0, -30.0).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        CONCEPT_LOCKER,
        CONCEPT_LOCKER,
        Transform::from_xyz(9.5, 0.0, -9.15).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        CONCEPT_LOCKER,
        CONCEPT_LOCKER,
        Transform::from_xyz(13.35, 0.0, -12.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(13.15, 0.0, -6.25).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(7.0, 0.0, -7.3).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(12.0, 0.0, -8.0),
    ));
    commands.spawn(prop(
        CONCEPT_LOCKER,
        CONCEPT_LOCKER,
        Transform::from_xyz(13.35, 0.0, -2.2).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CONCEPT_LOCKER,
        CONCEPT_LOCKER,
        Transform::from_xyz(13.35, 0.0, 2.2).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CONCEPT_TABLE,
        CONCEPT_TABLE,
        Transform::from_xyz(9.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(13.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CONCEPT_TABLE,
        CONCEPT_TABLE,
        Transform::from_xyz(-2.5, 0.0, -10.0).with_rotation(Quat::from_rotation_y(0.0)),
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
    spawn_composition(
        commands,
        "shelf_unit_bins",
        Transform::from_xyz(-2.5, 0.0, -8.35),
        &[("shelf_bin_contents", Transform::IDENTITY)],
    );
    commands.spawn(prop(
        "shelf_unit_alt",
        "shelf_unit_alt",
        Transform::from_xyz(2.5, 0.0, -8.35).with_rotation(Quat::from_rotation_y(PI)),
    ));
    spawn_composition(
        commands,
        "shelf_unit_bins",
        Transform::from_xyz(-9.5, 0.0, -7.0).with_rotation(Quat::from_rotation_y(PI)),
        &[("shelf_bin_contents", Transform::IDENTITY)],
    );
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
        CONCEPT_LOCKER,
        CONCEPT_LOCKER,
        Transform::from_xyz(-13.35, 0.0, -18.75).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CONCEPT_TABLE,
        CONCEPT_TABLE,
        Transform::from_xyz(-7.5, 0.0, -22.9).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(-2.5, 0.0, -17.2).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    spawn_composition(
        commands,
        CONCEPT_TABLE,
        Transform::from_xyz(3.75, 0.0, -18.75).with_rotation(Quat::from_rotation_y(PI)),
        &[("toolbox", Transform::from_xyz(0.46, 0.8, -0.14))],
    );
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(3.0, 0.0, -20.7).with_rotation(Quat::from_rotation_y(PI)),
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
        CONCEPT_TABLE,
        CONCEPT_TABLE,
        Transform::from_xyz(6.8, 0.0, -25.7).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit_alt",
        "shelf_unit_alt",
        Transform::from_xyz(-10.8, 0.0, -17.55).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "lab_console",
        "lab_console",
        Transform::from_xyz(-3.1, 0.0, -2.5).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(2.8, 0.0, 2.6).with_rotation(Quat::from_rotation_y(PI)),
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
    commands.spawn(prop(
        BOILER_UNIT,
        BOILER_UNIT,
        Transform::from_xyz(-10.0, 0.0, 0.0),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(-12.8, 0.0, -3.75).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "shelf_unit_alt",
        "shelf_unit_alt",
        Transform::from_xyz(-10.0, 0.0, -5.7).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(-13.1, 0.0, -9.4).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CONCEPT_CONTAINMENT_TANK,
        CONCEPT_CONTAINMENT_TANK,
        Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(-2.0, 0.0, 3.5).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(-13.0, 0.0, 6.0).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CONCEPT_LOCKER,
        CONCEPT_LOCKER,
        Transform::from_xyz(-13.35, 0.0, 9.75).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "work_island",
        "work_island",
        Transform::from_xyz(-9.5, 0.0, 6.0).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(-7.0, 0.0, 4.4).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "clutter_tools",
        "clutter_tools",
        Transform::from_xyz(-9.0, 0.0, 9.0).with_rotation(Quat::from_rotation_y(0.6)),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(-13.0, 0.0, 13.75).with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CONCEPT_TABLE,
        CONCEPT_TABLE,
        Transform::from_xyz(-7.5, 0.0, 13.75).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "shelf_unit",
        "shelf_unit",
        Transform::from_xyz(-12.35, 0.0, 15.65).with_rotation(Quat::from_rotation_y(PI)),
    ));
    spawn_composition(
        commands,
        "shelf_unit_bins",
        Transform::from_xyz(-12.0, 0.0, 9.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        &[("shelf_bin_contents", Transform::IDENTITY)],
    );
    commands.spawn(prop(
        "clutter_papers",
        "clutter_papers",
        Transform::from_xyz(-9.0, 0.0, 14.5).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(0.0, 0.0, 15.7).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "work_island",
        "work_island",
        Transform::from_xyz(0.0, 0.0, 7.0).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        CONCEPT_TABLE,
        CONCEPT_TABLE,
        Transform::from_xyz(-1.5, 0.0, 12.5).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        CONCEPT_LOCKER,
        CONCEPT_LOCKER,
        Transform::from_xyz(3.35, 0.0, 13.75).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "chair_tipped",
        "chair_tipped",
        Transform::from_xyz(1.8, 0.0, 12.6).with_rotation(Quat::from_rotation_y(0.4)),
    ));
    commands.spawn(prop(
        "clutter_papers",
        "clutter_papers",
        Transform::from_xyz(-1.2, 0.0, 9.0).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(13.0, 0.0, 6.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        CONCEPT_LOCKER,
        CONCEPT_LOCKER,
        Transform::from_xyz(13.35, 0.0, 9.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "shelf_unit",
        "shelf_unit",
        Transform::from_xyz(9.0, 0.0, 4.25).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "steel_drum",
        "steel_drum",
        Transform::from_xyz(7.0, 0.0, 10.5).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(11.0, 0.0, 6.2).with_rotation(Quat::from_rotation_y(0.3)),
    ));
    commands.spawn(prop(
        "workbench",
        "workbench",
        Transform::from_xyz(13.0, 0.0, 13.75).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    spawn_composition(
        commands,
        "shelf_unit_bins",
        Transform::from_xyz(7.1, 0.0, 13.75).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        &[("shelf_bin_contents", Transform::IDENTITY)],
    );
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(7.8, 0.0, 15.6).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "steel_drum",
        "steel_drum",
        Transform::from_xyz(12.6, 0.0, 11.8).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "steel_drum",
        "steel_drum",
        Transform::from_xyz(-12.8, 0.0, 18.2).with_rotation(Quat::from_rotation_y(0.0)),
    ));
    commands.spawn(prop(
        "storage_crate",
        "storage_crate",
        Transform::from_xyz(12.8, 0.0, 18.2).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "drum_spilled",
        "drum_spilled",
        Transform::from_xyz(1.5, 0.0, 18.2).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "trace_drag_marks",
        "trace_drag_marks",
        Transform::from_xyz(-2.5, 0.0, 17.5).with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    commands.spawn(prop(
        "vent_grille",
        "vent_grille",
        Transform::from_xyz(-12.6, 0.0, 17.0),
    ));
    commands.spawn(prop(
        "clutter_tools",
        "clutter_tools",
        Transform::from_xyz(12.3, 0.0, 17.0).with_rotation(Quat::from_rotation_y(0.8)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(-7.5, 0.0, 18.38).with_rotation(Quat::from_rotation_y(PI)),
    ));
    commands.spawn(prop(
        "shelf_unit_low",
        "shelf_unit_low",
        Transform::from_xyz(7.5, 0.0, 18.38).with_rotation(Quat::from_rotation_y(PI)),
    ));
}

fn spawn_crawl_vent(commands: &mut Commands, transform: Transform, covered: bool) {
    let grille = if covered {
        Transform::from_xyz(0.0, 0.0, -0.085).with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
    } else {
        Transform::from_xyz(0.0, -0.55, -0.72)
    };
    spawn_composition(
        commands,
        "concept_crawl_vent",
        transform,
        &[("vent_grille", grille)],
    );
}

fn spawn_composition(
    commands: &mut Commands,
    module: &'static str,
    transform: Transform,
    decorations: &[(&'static str, Transform)],
) {
    commands
        .spawn(prop(module, module, transform))
        .with_children(|children| {
            for &(name, local) in decorations {
                children.spawn(prop(name, name, local));
            }
        });
}

pub(super) fn spawn_dressing(commands: &mut Commands) {
    for (x, z, yaw) in [
        (-5.75, -14.7, 0.4),
        (5.75, -14.8, -0.2),
        (-5.7, -6.9, 1.0),
        (5.75, -6.8, 0.7),
        (-5.7, 2.8, -0.5),
        (5.7, 2.7, 0.3),
        (-5.7, 18.25, -0.4),
        (5.7, 18.25, 0.2),
    ] {
        commands.spawn(prop(
            "potted_plant",
            "potted_plant",
            Transform::from_xyz(x, 0.0, z).with_rotation(Quat::from_rotation_y(yaw)),
        ));
    }
    for (x, z, yaw) in [
        (-5.0, -19.0, 0.2),
        (5.1, -11.4, -0.5),
        (-5.1, -3.7, 0.7),
        (5.0, -1.7, 1.3),
        (-5.0, 10.8, -0.4),
        (5.0, 17.1, 0.6),
    ] {
        commands.spawn(prop(
            "clutter_papers",
            "clutter_papers",
            Transform::from_xyz(x, 0.0, z).with_rotation(Quat::from_rotation_y(yaw)),
        ));
    }
    for (x, y, z, yaw) in [
        (8.9, 0.802, -0.1, 0.4),
        (-7.5, 0.802, -22.9, 0.1),
        (6.8, 0.802, -25.7, -0.2),
        (-2.1, 0.802, -10.18, -0.3),
        (4.05, 0.802, -18.75, 0.25),
        (-7.5, 0.802, 13.6, -0.2),
        (-1.7, 0.802, 12.5, 0.7),
        (7.0, 0.902, -7.65, 0.1),
    ] {
        commands.spawn(prop(
            "clutter_papers",
            "clutter_papers",
            Transform::from_xyz(x, y, z)
                .with_rotation(Quat::from_rotation_y(yaw))
                .with_scale(Vec3::splat(0.5)),
        ));
    }
    for (x, y, z, yaw, scale) in [(7.0, 0.9, -7.12, 0.0, 0.65), (13.0, 0.9, 0.1, -0.2, 1.0)] {
        commands.spawn(prop(
            "clutter_tools",
            "clutter_tools",
            Transform::from_xyz(x, y, z)
                .with_rotation(Quat::from_rotation_y(yaw))
                .with_scale(Vec3::splat(scale)),
        ));
    }
}

pub(super) fn spawn_flashbang(commands: &mut Commands) {
    commands.spawn((
        Name::new("flashbang"),
        PickupKind::Flashbang,
        Transform::from_xyz(7.2, 0.78, -15.5).with_rotation(Quat::from_rotation_y(0.5)),
    ));
}

pub(super) fn spawn_detector(commands: &mut Commands) {
    commands.spawn((
        Name::new("detector"),
        PickupKind::Detector,
        Transform::from_xyz(-2.0, 1.0, 3.5).with_rotation(Quat::from_rotation_y(PI - 0.3)),
    ));
}
