use bevy::prelude::*;

use crate::facility::{
    grid::Side::{East, North, South, West},
    level::{Arrow::Right, Glow, Lamp::*, Level},
    Objective,
};

const CONDUIT: &str = "wall_conduit";
const STRIPED: &str = "floor_tile_marked";
const EXIT_GREEN: Color = Color::linear_rgb(0.12, 1.0, 0.35);
const FAULT_RED: Color = Color::linear_rgb(1.0, 0.06, 0.03);
const FIRE: Color = Color::linear_rgb(1.0, 0.35, 0.08);
const SPECIMEN: Color = Color::linear_rgb(0.3, 0.95, 0.85);
const FIREBOX: Vec3 = Vec3::new(0.0, 0.5, -0.85);
const TANK_LAMP: Vec3 = Vec3::new(0.0, 2.1, 0.0);

pub fn first_floor() -> Level {
    let mut level = Level::default();

    level.room("maintenance", (-5, -9), (-1, -7)).walls(CONDUIT);
    level.room("exit", (0, -12), (3, -9));
    level.room("security", (4, -12), (5, -10));
    level.room("service", (0, -8), (0, -7)).floor(STRIPED);
    level.room("office", (1, -8), (3, -7));
    level.room("utility", (-5, -6), (-3, -3)).walls(CONDUIT);
    level.room("hiding", (3, -1), (5, 1));
    level.room("west_hall", (-2, -6), (-2, 1)).walls(CONDUIT);
    level.room("reception", (-1, -6), (1, -4));
    level.room("east_hall", (2, -6), (2, 1));
    level.room("storage", (3, -6), (5, -4));
    level.room("west_link", (-1, -3), (-1, -2)).walls(CONDUIT);
    level
        .room("intake", (0, -3), (0, -2))
        .floor(STRIPED)
        .walls(CONDUIT);
    level.room("east_link", (1, -3), (1, -2));
    level.room("lab", (-1, -1), (1, 1));
    level.room("boiler", (-5, -1), (-3, 1)).walls(CONDUIT);

    level.exit((0, -12), North);
    level.door((0, -9), South, 75.0);
    level.door((1, -9), South, 75.0);
    level.door((3, -11), East, 75.0);
    level.door((0, -8), West, 75.0);
    level.door((0, -8), East, 75.0);
    level.door((-2, -7), South, 70.0);
    level.passage((0, -7), South);
    level.door((2, -7), South, 70.0);
    level.passage((-1, -5), West);
    level.passage((1, -5), East);
    level.door((2, -5), East, 75.0);
    level.passage((0, -4), South);
    level.door((-2, -4), West, 70.0);
    level.passage((-2, -2), East);
    level.passage((-1, -2), East);
    level.passage((0, -2), East);
    level.passage((1, -2), East);
    level.door((0, -2), South, 75.0);
    level.door((-2, 0), West, 70.0);
    level.door((-1, 0), West, 75.0);
    level.door((1, 0), East, 75.0);
    level.door((2, 0), East, 75.0);

    level.ceiling_lamp((0, -11), Cool, true);
    level.ceiling_lamp((2, -12), Cool, false);
    level.ceiling_lamp((2, -10), Dead, false);
    level.ceiling_lamp((4, -11), Cool, false);
    level.ceiling_lamp((0, -8), Dead, true);
    level.ceiling_lamp((-3, -8), Amber, false);
    level.ceiling_lamp((-4, -9), Dead, false);
    level.ceiling_lamp((2, -8), Cool, false);
    level.ceiling_lamp((-4, -4), Amber, false);
    level.ceiling_lamp((4, 0), Flicker, false);
    level.ceiling_lamp((-2, -6), Flicker, false);
    level.ceiling_lamp((0, -6), Cool, true);
    level.ceiling_lamp((0, -4), Cool, true);
    level.ceiling_lamp((3, -5), Dead, false);
    level.ceiling_lamp((-2, -4), Dead, true);
    level.ceiling_lamp((0, -2), Flicker, false);
    level.ceiling_lamp((2, -4), Dead, true);
    level.ceiling_lamp((-4, 1), Amber, false);
    level.ceiling_lamp((0, 1), Flicker, false);
    level.ceiling_lamp((2, 1), Dead, false);

    level
        .wall_fixture("exit_sign", (0, -12), North, 0.0, 2.62)
        .light(Vec3::NEG_Z * 0.2, EXIT_GREEN, 6_000.0, 5.0);
    level
        .wall_fixture("fuse_panel", (2, -12), North, 0.0, 1.85)
        .objective(Objective::FusePanel);
    level
        .wall_fixture("wall_lamp_red", (5, -5), East, 0.0, 2.2)
        .light(Vec3::NEG_Z * 0.25, FAULT_RED, 60_000.0, 5.0)
        .glow(Glow::Pulse);
    level.wall_fixture("wall_vent", (-5, -1), West, 0.4, 0.45);
    level.wall_fixture("wall_vent", (-5, -8), West, 0.0, 1.9);
    level.wall_fixture("wall_vent", (3, -8), East, -0.3, 1.9);
    level.wall_fixture("sign_label_boiler_room", (-2, 0), West, 0.0, 2.6);
    level.wall_fixture("sign_label_storage", (2, -5), East, 0.0, 2.6);
    level.wall_fixture("sign_label_maintenance", (0, -8), West, 0.0, 2.6);
    level.wall_fixture("sign_label_office", (0, -8), East, 0.0, 2.6);
    level.wall_fixture("pipe_manifold", (-5, 0), West, 0.0, 1.6);
    level.wall_fixture("tool_pegboard", (-5, 1), South, -0.4, 1.6);
    level.wall_fixture("concept_crawl_vent", (-5, 1), West, 0.6, 0.55);
    level.wall_fixture("trace_claw_marks", (-1, -3), North, 0.3, 1.6);
    level.wall_fixture("trace_claw_marks", (1, -1), East, 0.2, 1.55);
    level.wall_fixture("trace_claw_marks", (-5, -9), West, 0.5, 1.6);

    level.wall_sign(
        (-1, -6),
        North,
        0.0,
        2.04,
        &[
            ("sign_label_exit", Right),
            ("sign_label_maintenance", Right),
        ],
    );
    level.wall_sign(
        (1, -6),
        East,
        0.0,
        2.04,
        &[("sign_label_storage", Right), ("sign_label_office", Right)],
    );
    level.exit_hanger((0, -7), North, North);
    level.exit_hanger((0, -5), East, North);
    level.exit_hanger((0, -3), North, North);
    level.exit_hanger((-2, -2), North, North);
    level.exit_hanger((2, -2), North, North);

    level.prop("workbench", (-4, -8), (-0.65, 0.0), East);
    level.prop("storage_crate", (-4, -9), (-0.3, -0.4), South);
    level.prop("steel_drum", (-2, -9), (0.6, -0.6), South);
    level.prop("shelf_unit", (3, -8), (0.75, 0.0), West);
    level.prop("shelf_unit_bins", (3, -7), (0.75, 0.0), West);
    level.prop("storage_crate", (2, -9), (0.5, -0.5), South);
    level.prop("shelf_unit", (4, -5), (0.75, 0.0), West);
    level.prop("shelf_unit_bins", (4, -6), (0.75, 0.0), West);
    level.prop("storage_crate", (3, -6), (-0.3, -0.5), South);
    level.prop("workbench", (4, -12), (0.0, 0.0), South);
    level.prop("concept_locker", (4, -10), (0.5, 0.4), West);
    level.prop("concept_locker", (4, -4), (-0.5, 0.5), North);
    level.prop("concept_locker", (4, -1), (0.55, 0.3), West);
    level.prop("concept_locker", (4, 1), (0.55, -0.3), West);
    level.prop("concept_table", (-1, -4), (0.0, 0.0), South);
    level.prop("clutter_papers", (-1, -4), (0.0, 0.0), North);
    level.prop("chair_tipped", (1, -4), (0.0, 0.0), East);
    level.prop("shelf_unit_bins", (-5, -4), (-0.3, 0.0), East);
    level.prop("workbench", (-4, -6), (0.0, -0.5), South);
    level.prop("clutter_tools", (-4, -3), (0.0, 0.0), North);
    level.prop("concept_locker", (-4, -7), (0.0, 0.5), North);
    level.prop("concept_table", (-3, -9), (0.0, -0.4), South);
    level.prop("shelf_unit_low", (-1, -7), (0.0, 0.3), West);
    level
        .prop("boiler_unit", (-4, 0), (0.0, 0.0), North)
        .light(FIREBOX, FIRE, 45_000.0, 7.0)
        .glow(Glow::Flicker(-7.0));
    level.prop("vent_grille", (-5, 1), (-0.55, 0.5), East);
    level.prop("work_island", (-3, -8), (0.0, 0.0), North);
    level.prop("chair_tipped", (-4, -7), (0.6, -0.6), North);
    level.prop("drum_spilled", (-2, -9), (-0.6, 0.5), North);
    level.prop("clutter_tools", (-4, -9), (0.1, 0.7), North);
    level.prop("clutter_papers", (-3, -7), (0.6, -1.0), North);
    level.prop("steel_drum", (5, -4), (0.7, 0.3), West);
    level
        .prop("concept_containment_tank", (0, 0), (0.0, 0.0), East)
        .light(TANK_LAMP, SPECIMEN, 12_000.0, 4.5)
        .glow(Glow::Flicker(0.0));
    level.prop("lab_console", (-1, -1), (-0.6, 0.0), East);
    level.prop("chair_tipped", (0, 1), (0.2, 0.0), North);
    level.prop("clutter_papers", (-1, -1), (0.6, 0.6), North);
    level.prop("trace_drag_marks", (1, -1), (0.0, 0.0), East);

    level.start((0, -2), North);
    level
}
