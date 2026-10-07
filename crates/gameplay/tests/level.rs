use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::f32::consts::{FRAC_PI_2, PI};
use std::path::{Path, PathBuf};

use bevy::prelude::*;

use gameplay::facility::{
    door_panel,
    grid::{
        cell_center, cell_of, edge_along, edge_key, edge_vertices, half_point, wall_right, Side,
        DOOR_H, DOOR_W, FRAME_DEPTH, HALF, POST, TILE, WALL_T,
    },
    level::{Arrow, Edge, EdgeKind, Glow, Level, Piece, Placement},
    DoorState, Objective,
};
use gameplay::levels::first_floor;

const WALL_CLEARANCE: f32 = 0.2;
const DOOR_ZONE_DEPTH: f32 = 1.3;
const TRIM_BANDS: [(f32, f32); 3] = [(0.0, 0.16), (1.1, 1.16), (2.78, 3.0)];
const CONDUIT_BAND: (f32, f32) = (2.36, 2.64);
const CONDUIT: &str = "wall_conduit";
const HANGER_HALF: Vec2 = Vec2::new(0.85, 0.015);
const HANGER_ROW: f32 = 2.33;
const LABEL_HALF: Vec2 = Vec2::new(0.65, 0.1);
const ROW_ARROW_X: f32 = 0.67;
const ROW_LABEL_X: f32 = -0.12;
const ROW_PITCH: f32 = 0.24;
const LAMP_HALF: Vec2 = Vec2::new(0.7, 0.16);
const FIREBOX: Vec3 = Vec3::new(0.0, 0.5, -0.85);
const TANK_LAMP: Vec3 = Vec3::new(0.0, 2.1, 0.0);

const CEILING: f32 = 3.0;
const HEAD_CLEARANCE: f32 = 2.1;
const EYE: f32 = 1.6;
const MAX_LOOK_UP_DEG: f32 = 25.0;
const MAX_VIEW_DEG: f32 = 45.0;
const READ_PER_CAP: f32 = 40.0;
const MIN_CAP_M: f32 = 0.1;
const BODY: f32 = 0.3;
const AISLE: f32 = 0.5;
const GRID: f32 = 0.1;

const WALKABLE: [&str; 2] = ["clutter_papers", "trace_drag_marks"];

const PROP_SCENES: &[&str] = &[
    "workbench",
    "storage_crate",
    "steel_drum",
    "shelf_unit",
    "shelf_unit_bins",
    "concept_locker",
    "concept_table",
    "clutter_papers",
    "chair_tipped",
    "clutter_tools",
    "shelf_unit_low",
    "boiler_unit",
    "vent_grille",
    "work_island",
    "drum_spilled",
    "concept_containment_tank",
    "lab_console",
    "trace_drag_marks",
];

const FIXTURE_SCENES: &[&str] = &[
    "exit_sign",
    "fuse_panel",
    "wall_lamp_red",
    "wall_vent",
    "sign_label_boiler_room",
    "sign_label_storage",
    "sign_label_maintenance",
    "sign_label_office",
    "pipe_manifold",
    "tool_pegboard",
    "concept_crawl_vent",
    "trace_claw_marks",
];

const DESTINATIONS: &[(&str, Option<&str>)] = &[
    ("sign_label_boiler_room", Some("boiler")),
    ("sign_label_storage", Some("storage")),
    ("sign_label_maintenance", Some("maintenance")),
    ("sign_label_office", Some("office")),
    ("sign_label_exit", None),
];

fn half_size(scene: &str) -> Vec2 {
    match scene {
        "storage_crate" => Vec2::new(0.51, 0.415),
        "steel_drum" => Vec2::new(0.3, 0.3),
        "shelf_unit" | "shelf_unit_bins" => Vec2::new(0.9, 0.26),
        "workbench" | "lab_console" => Vec2::new(0.8, 0.35),
        "boiler_unit" => Vec2::new(0.8, 0.8),
        "shelf_unit_low" => Vec2::new(0.6, 0.23),
        "concept_locker" => Vec2::new(0.5, 0.5),
        "concept_table" => Vec2::new(0.9, 0.47),
        "vent_grille" => Vec2::new(0.43, 0.33),
        "concept_containment_tank" => Vec2::new(0.95, 0.95),
        "work_island" => Vec2::new(1.0, 0.5),
        "clutter_papers" => Vec2::new(0.6, 0.45),
        "clutter_tools" => Vec2::new(0.5, 0.35),
        "chair_tipped" => Vec2::new(0.5, 0.45),
        "drum_spilled" => Vec2::new(0.75, 0.45),
        "trace_drag_marks" => Vec2::new(0.3, 1.0),
        "wall_lamp_red" => Vec2::new(0.12, 0.16),
        "exit_sign" => Vec2::new(0.25, 0.1),
        "wall_vent" => Vec2::new(0.3, 0.2),
        "sign_label_boiler_room"
        | "sign_label_storage"
        | "sign_label_maintenance"
        | "sign_label_office"
        | "sign_label_exit" => LABEL_HALF,
        "pipe_manifold" => Vec2::new(0.95, 0.25),
        "tool_pegboard" => Vec2::new(0.6, 0.4),
        "concept_crawl_vent" => Vec2::new(0.45, 0.35),
        "trace_claw_marks" => Vec2::new(0.35, 0.3),
        "fuse_panel" => Vec2::new(0.55, 0.65),
        other => panic!("no footprint for {other}"),
    }
}

fn is_prop(p: &Placement) -> bool {
    PROP_SCENES.contains(&p.pieces[0].scene)
}

fn is_wall_fixture(p: &Placement) -> bool {
    FIXTURE_SCENES.contains(&p.pieces[0].scene)
        && !p.pieces.iter().any(|piece| piece.scene == "sign_arrow")
}

fn is_hanger(p: &Placement) -> bool {
    p.pieces.iter().any(|piece| piece.scene == "sign_hanger")
}

fn is_wall_sign(p: &Placement) -> bool {
    !is_hanger(p) && p.pieces.iter().any(|piece| piece.scene == "sign_arrow")
}

fn label_scene(scene: &str) -> bool {
    DESTINATIONS.iter().any(|(l, _)| *l == scene)
}

fn dest_area(label: &str) -> Option<&'static str> {
    DESTINATIONS
        .iter()
        .find(|(l, _)| *l == label)
        .unwrap_or_else(|| panic!("{label} is not a destination"))
        .1
}

fn roll_of(arrow: Arrow) -> f32 {
    match arrow {
        Arrow::Left => 0.0,
        Arrow::Right => PI,
        Arrow::Ahead => FRAC_PI_2,
        Arrow::Back => -FRAC_PI_2,
    }
}

fn side_from_dir(dir: Vec3) -> Side {
    Side::ALL
        .into_iter()
        .find(|s| s.dir().distance(dir) < 1e-3)
        .expect("axis-aligned direction")
}

fn facing_of(transform: &Transform) -> Side {
    side_from_dir(transform.rotation * Vec3::NEG_Z)
}

fn room(level: &Level, name: &str) -> usize {
    level
        .rooms
        .iter()
        .position(|a| a.name == name)
        .unwrap_or_else(|| panic!("no room named {name}"))
}

fn exit_door(level: &Level) -> (IVec2, Side) {
    level
        .openings
        .iter()
        .find_map(|o| {
            let door = o.door?;
            (door.objective == Some(Objective::Exit)).then_some((o.cell, o.side))
        })
        .expect("level has an exit")
}

fn prop_bounds(center: Vec3, scene: &str, facing: Side) -> (Vec2, Vec2) {
    let half = half_size(scene);
    let half = match facing {
        Side::North | Side::South => half,
        Side::East | Side::West => Vec2::new(half.y, half.x),
    };
    let c = Vec2::new(center.x, center.z);
    (c - half, c + half)
}

fn overlaps(a: (Vec2, Vec2), b: (Vec2, Vec2)) -> bool {
    a.0.x < b.1.x && b.0.x < a.1.x && a.0.y < b.1.y && b.0.y < a.1.y
}

fn door_zones(edges: &HashMap<IVec2, Edge>) -> Vec<(Vec2, Vec2)> {
    edges
        .iter()
        .filter(|(_, edge)| matches!(edge.kind, EdgeKind::Door(_)))
        .map(|(key, _)| {
            let mid = half_point(*key);
            let along = edge_along(*key);
            let half =
                along * (DOOR_W / 2.0 + 0.1) + Vec3::new(along.z, 0.0, along.x) * DOOR_ZONE_DEPTH;
            (
                Vec2::new(mid.x - half.x, mid.z - half.z),
                Vec2::new(mid.x + half.x, mid.z + half.z),
            )
        })
        .collect()
}

fn wall_slot_ok(
    edges: &HashMap<IVec2, Edge>,
    cell: IVec2,
    side: Side,
    offset: f32,
    half_w: f32,
    (bottom, top): (f32, f32),
) -> Result<Vec3, String> {
    let key = edge_key(cell, side);
    let Some(edge) = edges.get(&key).copied() else {
        return Err(format!("{cell} {side:?} has no wall"));
    };
    let face = side.opposite();
    if offset.abs() + half_w > HALF - POST / 2.0 {
        return Err(format!("{cell} {side:?} overlaps a post"));
    }
    if TRIM_BANDS.iter().any(|(lo, hi)| bottom < *hi && *lo < top) {
        return Err(format!("{cell} {side:?} crosses a wall trim"));
    }
    match edge.kind {
        EdgeKind::Passage => return Err(format!("{cell} {side:?} is on a passage")),
        EdgeKind::Door(_) if bottom < DOOR_H + 0.25 => {
            return Err(format!("{cell} {side:?} overlaps the door frame"));
        }
        EdgeKind::Wall(scene)
            if scene == CONDUIT
                && edge.facing == face
                && bottom < CONDUIT_BAND.1
                && CONDUIT_BAND.0 < top =>
        {
            return Err(format!("{cell} {side:?} overlaps the conduit"));
        }
        _ => {}
    }
    Ok(half_point(key) + face.dir() * (WALL_T / 2.0) + wall_right(face) * offset)
}

#[derive(Clone, Copy)]
struct Step {
    cell: IVec2,
    side: Side,
}

fn moves(
    cells: &HashMap<IVec2, usize>,
    edges: &HashMap<IVec2, Edge>,
    cell: IVec2,
) -> Vec<(Side, Option<IVec2>)> {
    let Some(&area) = cells.get(&cell) else {
        return Vec::new();
    };
    Side::ALL
        .into_iter()
        .filter_map(|side| {
            let n = cell + side.offset();
            let neighbor = cells.get(&n).copied();
            if neighbor == Some(area) {
                return Some((side, Some(n)));
            }
            let key = edge_key(cell, side);
            match edges.get(&key)?.kind {
                EdgeKind::Wall(_) => None,
                EdgeKind::Passage | EdgeKind::Door(_) => Some((side, neighbor.map(|_| n))),
            }
        })
        .collect()
}

fn route(
    cells: &HashMap<IVec2, usize>,
    edges: &HashMap<IVec2, Edge>,
    from: IVec2,
    dest: Option<usize>,
    exit: (IVec2, Side),
) -> Vec<Step> {
    let reached = |cell: IVec2| match dest {
        Some(index) => cells.get(&cell) == Some(&index),
        None => cell == exit.0,
    };
    let finish = |cell: IVec2, mut steps: Vec<Step>| {
        if dest.is_none() {
            steps.push(Step { cell, side: exit.1 });
        }
        steps
    };
    if reached(from) {
        return finish(from, Vec::new());
    }
    let mut prev: HashMap<IVec2, Step> = HashMap::new();
    let mut queue = VecDeque::from([from]);
    while let Some(cell) = queue.pop_front() {
        for (side, next) in moves(cells, edges, cell) {
            let Some(next) = next else { continue };
            if next == from || prev.contains_key(&next) {
                continue;
            }
            prev.insert(next, Step { cell, side });
            if reached(next) {
                let mut steps = Vec::new();
                let mut at = next;
                while at != from {
                    let step = prev[&at];
                    steps.push(step);
                    at = step.cell;
                }
                steps.reverse();
                return finish(next, steps);
            }
            queue.push_back(next);
        }
    }
    panic!("no route from {from}");
}

fn expected_arrow(
    cells: &HashMap<IVec2, usize>,
    edges: &HashMap<IVec2, Edge>,
    cell: IVec2,
    forward: Side,
    dest: Option<usize>,
    exit: (IVec2, Side),
) -> Arrow {
    let steps = route(cells, edges, cell, dest, exit);
    for step in &steps {
        if step.side != forward || moves(cells, edges, step.cell).len() >= 3 {
            return Arrow::toward(forward, step.side);
        }
    }
    Arrow::Ahead
}

struct SignRow<'a> {
    label: &'a Piece,
    arrow: &'a Piece,
}

fn rows_of(placement: &Placement) -> Vec<SignRow<'_>> {
    let marked: Vec<_> = placement
        .pieces
        .iter()
        .filter(|p| label_scene(p.scene) || p.scene == "sign_arrow")
        .collect();
    marked
        .chunks(2)
        .map(|c| SignRow {
            label: c[0],
            arrow: c[1],
        })
        .collect()
}

struct SignSpec<'a> {
    placement: &'a Placement,
    cell: IVec2,
    side: Side,
}

fn wall_signs(level: &Level) -> Vec<SignSpec<'_>> {
    level
        .placements
        .iter()
        .filter(|p| is_wall_sign(p))
        .map(|p| SignSpec {
            placement: p,
            cell: p.cell,
            side: facing_of(&p.transform).opposite(),
        })
        .collect()
}

struct HangerSpec<'a> {
    placement: &'a Placement,
    cell: IVec2,
    facing: Side,
}

fn hangers(level: &Level) -> Vec<HangerSpec<'_>> {
    level
        .placements
        .iter()
        .filter(|p| is_hanger(p))
        .map(|p| HangerSpec {
            placement: p,
            cell: p.cell,
            facing: facing_of(&p.transform),
        })
        .collect()
}

fn hanger_bounds(cell: IVec2, facing: Side) -> (Vec2, Vec2) {
    let center = cell_center(cell);
    let across = Vec3::new(facing.dir().z.abs(), 0.0, facing.dir().x.abs());
    let half = across * HANGER_HALF.x + facing.dir().abs() * HANGER_HALF.y;
    (
        Vec2::new(center.x - half.x, center.z - half.z),
        Vec2::new(center.x + half.x, center.z + half.z),
    )
}

fn lamp_bounds(cell: IVec2, along_z: bool) -> (Vec2, Vec2) {
    let c = cell_center(cell);
    let half = if along_z {
        Vec2::new(LAMP_HALF.y, LAMP_HALF.x)
    } else {
        LAMP_HALF
    };
    let c = Vec2::new(c.x, c.z);
    (c - half, c + half)
}

fn line_of_sight(
    edges: &HashMap<IVec2, Edge>,
    hangers: &[HangerSpec],
    posts: &[IVec2],
    from: Vec3,
    to: Vec3,
) -> Result<(), String> {
    let d = to - from;
    let n = (d.length() / 0.02).ceil() as usize;
    let mut last = cell_of(from);
    for i in 0..n {
        let p = from + d * (i as f32 / n as f32);
        for h in hangers {
            let (lo, hi) = hanger_bounds(h.cell, h.facing);
            if (lo.x..=hi.x).contains(&p.x) && (lo.y..=hi.y).contains(&p.z) && p.y >= 2.2 {
                return Err(format!("hanger at {} blocks the view at {p}", h.cell));
            }
        }
        for &post in posts {
            let d = Vec2::new(p.x, p.z) - Vec2::new(post.x as f32, post.y as f32) * HALF;
            if d.abs().max_element() < POST / 2.0 {
                return Err(format!("post blocks the view at {p}"));
            }
        }
        let c = cell_of(p);
        if c == last {
            continue;
        }
        let side = Side::ALL
            .into_iter()
            .find(|s| last + s.offset() == c)
            .ok_or_else(|| format!("view cuts a corner at {p}"))?;
        let key = edge_key(last, side);
        match edges.get(&key).map(|e| e.kind) {
            None | Some(EdgeKind::Passage) => {}
            Some(EdgeKind::Door(_))
                if (p - half_point(key)).dot(edge_along(key)).abs() < DOOR_W / 2.0
                    && p.y < DOOR_H => {}
            Some(_) => return Err(format!("wall blocks the view at {p}")),
        }
        last = c;
    }
    Ok(())
}

fn readable_from(
    edges: &HashMap<IVec2, Edge>,
    hangers: &[HangerSpec],
    posts: &[IVec2],
    eye: Vec3,
    center: Vec3,
    normal: Vec3,
    reach: f32,
) -> Result<(), String> {
    let flat = Vec3::new(eye.x - center.x, 0.0, eye.z - center.z);
    let distance = (eye - center).length();
    if distance > reach {
        return Err(format!("{distance} m away"));
    }
    let view = flat.normalize().dot(normal).acos().to_degrees();
    if view > MAX_VIEW_DEG {
        return Err(format!("seen at {view} degrees"));
    }
    let up = ((center.y - eye.y) / flat.length()).atan().to_degrees();
    if up >= MAX_LOOK_UP_DEG {
        return Err(format!("needs {up} degrees of look-up"));
    }
    line_of_sight(edges, hangers, posts, eye, center - normal * 0.05)
}

fn on_wall(edges: &HashMap<IVec2, Edge>, world_translation: Vec3, world_rotation: Quat) -> bool {
    let normal = world_rotation * Vec3::NEG_Z;
    let back = world_translation - normal * (WALL_T / 2.0);
    let back = Vec3::new(back.x, 0.0, back.z);
    edges.iter().any(|(&key, edge)| {
        let offset = back - half_point(key);
        edge.solid() && offset.dot(normal).abs() < 1e-4 && offset.cross(normal).length() <= HALF
    })
}

fn free(
    cells: &HashMap<IVec2, usize>,
    edges: &HashMap<IVec2, Edge>,
    props: &[(&Placement, &str)],
    p: Vec2,
) -> bool {
    let key = cell_of(Vec3::new(p.x, 0.0, p.y));
    if !cells.contains_key(&key) {
        return false;
    }
    let local = p - Vec2::new(key.x as f32, key.y as f32) * TILE;
    for side in Side::ALL {
        let Some(edge) = edges.get(&edge_key(key, side)) else {
            continue;
        };
        let o = side.offset().as_vec2();
        if HALF - local.dot(o) >= WALL_T / 2.0 + BODY {
            continue;
        }
        let along = local.dot(Vec2::new(-o.y, o.x)).abs();
        let open = match edge.kind {
            EdgeKind::Passage => true,
            EdgeKind::Door(_) => along + BODY <= DOOR_W / 2.0,
            EdgeKind::Wall(_) => false,
        };
        if !open {
            return false;
        }
    }
    props
        .iter()
        .filter(|(_, scene)| !WALKABLE.contains(scene))
        .all(|(placement, scene)| {
            let (lo, hi) = prop_bounds(
                placement.transform.translation,
                scene,
                facing_of(&placement.transform),
            );
            let d = (lo - p).max(p - hi).max(Vec2::ZERO);
            d.length() >= BODY
        })
}

fn snap(p: Vec2) -> IVec2 {
    IVec2::new((p.x / GRID).round() as i32, (p.y / GRID).round() as i32)
}

fn modules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/facility/modules")
}

fn shipped_modules() -> BTreeSet<String> {
    std::fs::read_dir(modules_dir())
        .expect("promoted modules")
        .filter_map(|entry| {
            let path = entry.expect("module entry").path();
            (path.extension()? == "glb").then(|| path.file_stem()?.to_str().map(String::from))?
        })
        .collect()
}

fn manifest() -> String {
    std::fs::read_to_string(modules_dir().join("modules.manifest.json"))
        .expect("modules manifest; run scripts/promote-facility-modules.sh")
}

fn entry<'a>(manifest: &'a str, module: &str) -> &'a str {
    let start = manifest
        .find(&format!("\n    \"{module}\": {{"))
        .unwrap_or_else(|| panic!("{module} in manifest"));
    let rest = &manifest[start + 1..];
    &rest[..rest.find("\n    }").expect("entry end")]
}

fn numbers(entry: &str, key: &str) -> Vec<f32> {
    let start = entry
        .find(&format!("\"{key}\": "))
        .unwrap_or_else(|| panic!("{key}"));
    let rest = &entry[start + key.len() + 4..];
    let rest = rest.trim_start_matches('[');
    rest[..rest.find(']').unwrap()]
        .split(',')
        .map(|n| n.trim().parse().unwrap())
        .collect()
}

fn bounds_of(manifest: &str, module: &str) -> (Vec3, Vec3) {
    let e = entry(manifest, module);
    let b = &e[e.find("\"bounds_bevy\"").unwrap()..];
    (
        Vec3::from_slice(&numbers(b, "min")),
        Vec3::from_slice(&numbers(b, "max")),
    )
}

fn cap_of(manifest: &str, module: &str) -> f32 {
    let e = entry(manifest, module);
    e[e.find("\"text_cap_m\": ").unwrap() + 14..]
        .split([',', '\n'])
        .next()
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

#[test]
fn rooms_have_expected_shapes() {
    let level = first_floor();
    for name in ["service", "intake", "west_hall", "east_hall"] {
        let area = level.rooms.iter().find(|a| a.name == name).unwrap();
        assert_eq!(area.min.x, area.max.x, "{name} is wider than one tile");
    }
    let lab = level.rooms.iter().find(|a| a.name == "lab").unwrap();
    assert_eq!(lab.max.x - lab.min.x, 2);
    let reception = level.rooms.iter().find(|a| a.name == "reception").unwrap();
    assert_eq!(
        (reception.min, reception.max),
        (IVec2::new(-1, -6), IVec2::new(1, -4))
    );
    assert!(level
        .rooms
        .iter()
        .all(|a| a.name != "cross" && !a.name.ends_with("upper_link")));
}

fn area_links(
    level: &Level,
    cells: &HashMap<IVec2, usize>,
) -> BTreeSet<(&'static str, &'static str)> {
    let mut links = BTreeSet::new();
    for opening in &level.openings {
        let other = opening.cell + opening.side.offset();
        let Some(&a) = cells.get(&other) else {
            continue;
        };
        let b = cells[&opening.cell];
        let pair = [level.rooms[a].name, level.rooms[b].name];
        links.insert((pair[0].min(pair[1]), pair[0].max(pair[1])));
    }
    links
}

#[test]
fn rooms_connect_through_twenty_two_edges_with_independent_bypasses() {
    let level = first_floor();
    let cells = level.cells();
    let links = area_links(&level, &cells);
    for (a, b) in [
        ("maintenance", "service"),
        ("service", "office"),
        ("service", "exit"),
        ("office", "exit"),
        ("exit", "security"),
        ("maintenance", "west_hall"),
        ("service", "reception"),
        ("office", "east_hall"),
        ("reception", "intake"),
        ("reception", "west_hall"),
        ("reception", "east_hall"),
        ("utility", "west_hall"),
        ("hiding", "east_hall"),
        ("east_hall", "storage"),
        ("west_hall", "west_link"),
        ("west_link", "intake"),
        ("intake", "east_link"),
        ("east_link", "east_hall"),
        ("intake", "lab"),
        ("west_hall", "lab"),
        ("lab", "east_hall"),
        ("boiler", "west_hall"),
    ] {
        assert!(links.contains(&(a.min(b), a.max(b))), "missing {a}-{b}");
    }
    assert_eq!(links.len(), 22, "unexpected area connection");
    assert!(!links.contains(&("office", "storage")));
    assert!(!links.contains(&("security", "storage")));
    assert!(!links.contains(&("office", "security")));
    assert_eq!(
        links
            .iter()
            .filter(|(a, b)| *a == "boiler" || *b == "boiler")
            .count(),
        1
    );
    let connected_without = |blocked: &[(&str, &str)]| {
        let mut visited = BTreeSet::from(["reception"]);
        let mut frontier = vec!["reception"];
        while let Some(from) = frontier.pop() {
            for &(a, b) in &links {
                if blocked
                    .iter()
                    .any(|&(x, y)| (a == x && b == y) || (a == y && b == x))
                {
                    continue;
                }
                let next = if a == from {
                    Some(b)
                } else if b == from {
                    Some(a)
                } else {
                    None
                };
                if let Some(next) = next.filter(|&next| visited.insert(next)) {
                    frontier.push(next);
                }
            }
        }
        visited.contains("intake")
    };
    assert!(connected_without(&[
        ("reception", "intake"),
        ("reception", "east_hall")
    ]));
    assert!(connected_without(&[
        ("reception", "intake"),
        ("reception", "west_hall")
    ]));
}

#[test]
fn exit_room_security_office_and_fuse_panel_are_positioned_and_every_room_reaches_exit() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let exit_idx = room(&level, "exit");
    let exit = level.rooms[exit_idx].clone();
    let security = level.rooms[room(&level, "security")].clone();
    let office = level.rooms[room(&level, "office")].clone();
    assert_eq!(
        (exit.min, exit.max),
        (IVec2::new(0, -12), IVec2::new(3, -9))
    );
    assert!(security.max.x - security.min.x < exit.max.x - exit.min.x);
    assert!(office.max.x < 4 && office.min.y > -9);

    let exit_opening = exit_door(&level);
    assert_eq!(exit_opening, (IVec2::new(0, -12), Side::North));

    let fuse = level
        .placements
        .iter()
        .find(|p| p.objective == Some(Objective::FusePanel))
        .expect("fuse panel placed");
    assert_eq!(cells[&fuse.cell], exit_idx);

    for (cell, side, to) in [
        (IVec2::new(0, -9), Side::South, "service"),
        (IVec2::new(1, -9), Side::South, "office"),
        (IVec2::new(3, -11), Side::East, "security"),
    ] {
        assert!(matches!(
            edges[&edge_key(cell, side)].kind,
            EdgeKind::Door(_)
        ));
        let n = cell + side.offset();
        assert_eq!(level.rooms[cells[&n]].name, to);
    }

    for area in &level.rooms {
        route(&cells, &edges, area.min, None, exit_opening);
    }

    for cell in [IVec2::new(-1, -5), IVec2::new(0, -5), IVec2::new(1, -5)] {
        assert_eq!(level.rooms[cells[&cell]].name, "reception");
        let p = cell_center(cell);
        assert!(!level.placements.iter().filter(|pl| is_prop(pl)).any(|pl| {
            let scene = pl.pieces[0].scene;
            if WALKABLE.contains(&scene) {
                return false;
            }
            let (lo, hi) = prop_bounds(pl.transform.translation, scene, facing_of(&pl.transform));
            p.x >= lo.x && p.x <= hi.x && p.z >= lo.y && p.z <= hi.y
        }));
    }
    assert!(level
        .placements
        .iter()
        .any(|p| p.pieces[0].scene == "concept_table" && p.cell == IVec2::new(-1, -4)));
}

#[test]
fn every_room_cell_is_claimed_by_exactly_one_area() {
    let level = first_floor();
    let total: usize = level
        .rooms
        .iter()
        .map(|a| ((a.max.x - a.min.x + 1) * (a.max.y - a.min.y + 1)) as usize)
        .sum();
    let cells = level.cells();
    assert_eq!(cells.len(), total, "two areas claim the same cell");
}

#[test]
fn every_cell_boundary_is_closed() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    for (&cell, &area) in &cells {
        for side in Side::ALL {
            let n = cell + side.offset();
            let inside = cells.get(&n) == Some(&area);
            let edge = edges.get(&edge_key(cell, side));
            assert_eq!(inside, edge.is_none(), "{cell} {side:?}");
            if !cells.contains_key(&n) {
                assert!(edge.unwrap().solid(), "open to void at {cell} {side:?}");
            }
        }
    }
}

#[test]
fn walls_face_into_an_area() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    for (&key, edge) in edges.iter().filter(|(_, e)| e.solid()) {
        let inside = half_point(key) + edge.facing.dir() * HALF;
        assert!(
            cells.contains_key(&cell_of(inside)),
            "edge {key:?} faces void"
        );
    }
}

#[test]
fn posts_cover_corners_ends_and_style_changes() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let posts: HashSet<_> = Level::posts(&edges).into_iter().collect();
    let mut joints: HashMap<IVec2, Vec<IVec2>> = HashMap::new();
    for (&key, edge) in edges.iter().filter(|(_, e)| e.solid()) {
        for v in edge_vertices(key) {
            joints.entry(v).or_default().push(key);
        }
        let _ = edge;
    }
    for (vertex, keys) in &joints {
        assert_eq!(vertex.x.rem_euclid(2), 1);
        assert_eq!(vertex.y.rem_euclid(2), 1);
        let straight = keys.len() == 2 && edge_along(keys[0]) == edge_along(keys[1]);
        let same = keys.len() == 2 && edges[&keys[0]] == edges[&keys[1]];
        if !straight || !same {
            let doorway_pair = keys.len() == 2
                && straight
                && keys
                    .iter()
                    .all(|k| matches!(edges[k].kind, EdgeKind::Door(_)));
            assert!(
                posts.contains(vertex) || doorway_pair,
                "missing post at {vertex:?}"
            );
        }
    }
    assert!(posts.iter().all(|v| joints.contains_key(v)));
    let unposted = joints.keys().filter(|v| !posts.contains(v)).count();
    assert!(unposted > 0, "straight runs should show wall-to-wall seams");
}

#[test]
fn shared_walls_are_single_and_face_the_first_area() {
    let mut level = Level::default();
    level.room("a", (0, 0), (0, 1));
    level.room("b", (1, 0), (1, 0)).walls(CONDUIT);
    let cells = level.cells();
    let edges = level.edges(&cells);
    let edge = edges[&edge_key(IVec2::new(0, 0), Side::East)];
    assert_eq!(edge.kind, EdgeKind::Wall("wall"));
    assert_eq!(edge.facing, Side::West);
    assert_eq!(edges.len(), 6 + 4 - 1);
    assert_eq!(Level::posts(&edges).len(), 7);
}

#[test]
fn passages_have_no_wall_and_link_two_areas() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let passages: Vec<_> = edges
        .iter()
        .filter(|(_, e)| e.kind == EdgeKind::Passage)
        .collect();
    assert!(passages.len() >= 8);
    for (&key, edge) in passages {
        let mid = half_point(key);
        let a = cells[&cell_of(mid + edge.facing.dir() * HALF)];
        let b = cells[&cell_of(mid - edge.facing.dir() * HALF)];
        assert_ne!(a, b);
    }
}

#[test]
fn door_panels_sit_closed_at_the_hinge_or_swing_open_by_their_angle() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let hinge = Vec3::X * -(DOOR_W / 2.0 - 0.01);
    let mut doors = 0;
    for (&key, edge) in &edges {
        let EdgeKind::Door(door) = edge.kind else {
            continue;
        };
        doors += 1;
        let panel = door_panel(door.state, door.swing);
        let boundary = Transform::from_translation(half_point(key))
            .with_rotation(Quat::from_rotation_y(edge.facing.yaw()));
        if !door.state.passable() {
            assert_eq!(panel.translation, hinge);
            assert_eq!(panel.rotation, Quat::IDENTITY);
        } else {
            assert_eq!(
                panel.translation,
                hinge + Vec3::NEG_Z * (FRAME_DEPTH + 0.035)
            );
            assert_eq!(
                panel.rotation,
                Quat::from_rotation_y(door.swing.to_radians())
            );
            let world_mid = boundary.transform_point(panel.translation);
            let world_leaf_end = boundary
                .transform_point(panel.translation + panel.rotation * Vec3::X * (DOOR_W - 0.02));
            let swing = (world_leaf_end - world_mid).dot(edge.facing.dir());
            assert!(swing > 0.0, "door at {key:?} swings out of its area");
            let expected = (DOOR_W - 0.02) * door.swing.to_radians().sin();
            assert!((swing - expected).abs() < 1e-3);
        }
    }
    assert!(doors >= 6);
}

#[test]
fn every_module_file_exists() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let dir = modules_dir();
    let m = manifest();
    let mut modules: HashSet<&str> = HashSet::new();
    for area in &level.rooms {
        modules.insert(area.floor);
    }
    modules.insert("ceiling_tile");
    for edge in edges.values() {
        match edge.kind {
            EdgeKind::Wall(scene) => {
                modules.insert(scene);
            }
            EdgeKind::Door(_) => {
                modules.insert("wall_doorway");
                modules.insert("door_panel");
            }
            EdgeKind::Passage => {}
        }
    }
    if !Level::posts(&edges).is_empty() {
        modules.insert("wall_post");
    }
    for placement in &level.placements {
        for piece in &placement.pieces {
            modules.insert(piece.scene);
        }
    }
    for module in modules {
        assert!(dir.join(format!("{module}.glb")).is_file(), "{module}.glb");
        assert!(m.contains(&format!("\"{module}\": {{")), "{module}");
    }
}

#[test]
fn lighting_states_vary_and_lights_sit_inside_their_cells() {
    let level = first_floor();
    let cells = level.cells();
    for scene in [
        "ceiling_light_cool",
        "ceiling_light_amber",
        "ceiling_light_dead",
    ] {
        assert!(
            level.placements.iter().any(|p| p.pieces[0].scene == scene),
            "{scene}"
        );
    }
    assert!(level
        .placements
        .iter()
        .any(|p| matches!(p.glow, Glow::Flicker(_))));
    assert!(level.placements.iter().any(|p| p.glow == Glow::Pulse));
    for placement in &level.placements {
        for light in &placement.lights {
            let position = placement.transform.transform_point(light.position);
            assert!(
                cells.contains_key(&cell_of(position)),
                "light outside its cell at {position}"
            );
            assert!(
                position.y > 0.3 && position.y < 3.0,
                "light height {position}"
            );
        }
    }
}

#[test]
fn props_sit_inside_rooms_clear_of_doors_and_other_props() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let zones = door_zones(&edges);
    let props: Vec<_> = level.placements.iter().filter(|p| is_prop(p)).collect();
    for (i, placement) in props.iter().enumerate() {
        let scene = placement.pieces[0].scene;
        let facing = facing_of(&placement.transform);
        let area = cells[&placement.cell];
        let (lo, hi) = prop_bounds(placement.transform.translation, scene, facing);
        for corner in [lo, hi, Vec2::new(lo.x, hi.y), Vec2::new(hi.x, lo.y)] {
            for d in [
                Vec2::new(WALL_CLEARANCE, WALL_CLEARANCE),
                Vec2::new(-WALL_CLEARANCE, -WALL_CLEARANCE),
                Vec2::new(WALL_CLEARANCE, -WALL_CLEARANCE),
                Vec2::new(-WALL_CLEARANCE, WALL_CLEARANCE),
            ] {
                let p = corner + d;
                assert_eq!(
                    cells.get(&cell_of(Vec3::new(p.x, 0.0, p.y))),
                    Some(&area),
                    "{scene} at {} is closer than {WALL_CLEARANCE} m to the wall",
                    placement.cell
                );
            }
        }
        assert!(
            zones.iter().all(|z| !overlaps(*z, (lo, hi))),
            "{scene} at {} blocks a door",
            placement.cell
        );
        if WALKABLE.contains(&scene) {
            continue;
        }
        for other in &props[..i] {
            let oscene = other.pieces[0].scene;
            if WALKABLE.contains(&oscene) {
                continue;
            }
            let (olo, ohi) = prop_bounds(
                other.transform.translation,
                oscene,
                facing_of(&other.transform),
            );
            assert!(
                !overlaps((olo, ohi), (lo, hi)),
                "{scene} at {} overlaps {oscene} at {}",
                placement.cell,
                other.cell
            );
        }
    }
}

#[test]
fn wall_fixtures_sit_on_solid_wall_faces_clear_of_trims_conduit_frames_and_posts() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let mut fixtures = 0;
    for placement in level.placements.iter().filter(|p| is_wall_fixture(p)) {
        fixtures += 1;
        let scene = placement.pieces[0].scene;
        let face = facing_of(&placement.transform);
        let side = face.opposite();
        let half = half_size(scene);
        let height = placement.transform.translation.y;
        let base = half_point(edge_key(placement.cell, side)) + face.dir() * (WALL_T / 2.0);
        let delta = placement.transform.translation - base;
        let offset = Vec3::new(delta.x, 0.0, delta.z).dot(wall_right(face));
        wall_slot_ok(
            &edges,
            placement.cell,
            side,
            offset,
            half.x,
            (height - half.y, height + half.y),
        )
        .unwrap_or_else(|e| panic!("{scene} at {}: {e}", placement.cell));
    }
    assert!(fixtures >= 10);
}

#[test]
fn wall_sign_arrows_point_along_the_route_from_the_readers_viewpoint() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let exit = exit_door(&level);
    let mut rows = 0;
    for sign in wall_signs(&level) {
        for row in rows_of(sign.placement) {
            rows += 1;
            let dest_index = dest_area(row.label.scene).map(|name| room(&level, name));
            let steps = route(&cells, &edges, sign.cell, dest_index, exit);
            assert!(
                !steps.is_empty(),
                "{} sign at {} is inside its destination",
                row.label.scene,
                sign.cell
            );
            assert_ne!(
                steps[0].side,
                sign.side.opposite(),
                "{} from {} sends the reader back",
                row.label.scene,
                sign.cell
            );
            let decision = steps
                .iter()
                .find(|s| s.side != sign.side || moves(&cells, &edges, s.cell).len() >= 3)
                .map_or(sign.side, |s| s.side);
            let world_rotation = sign.placement.transform.rotation * row.arrow.transform.rotation;
            let shown = world_rotation * Vec3::X;
            assert!(shown.y > -0.5, "down arrow at {}", sign.cell);
            let world = if shown.y > 0.5 {
                sign.side.dir()
            } else {
                shown
            };
            assert!(
                world.distance(decision.dir()) < 1e-4,
                "{} reader {} facing {:?}: arrow shows {world}, route goes {decision:?}",
                row.label.scene,
                sign.cell,
                sign.side
            );
            let label_rotation = sign.placement.transform.rotation * row.label.transform.rotation;
            let reader_left = Vec3::Y.cross(sign.side.dir());
            let read_order = (label_rotation * Vec3::NEG_X).dot(reader_left);
            assert!(read_order < -0.99, "label reads mirrored at {}", sign.cell);
        }
    }
    assert_eq!(rows, 4, "{rows} wall sign rows");
}

#[test]
fn exit_hangers_show_the_exit_direction_on_both_faces() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let exit = exit_door(&level);
    let mut faces = 0;
    for h in hangers(&level) {
        let steps = route(&cells, &edges, h.cell, None, exit);
        assert_eq!(
            (steps.last().unwrap().cell, steps.last().unwrap().side),
            exit
        );
        let readers = [h.facing.opposite(), h.facing];
        for (row, reader) in rows_of(h.placement).into_iter().zip(readers) {
            faces += 1;
            let expected = Arrow::toward(reader, steps[0].side);
            let world_rotation = h.placement.transform.rotation * row.arrow.transform.rotation;
            let shown = world_rotation * Vec3::X;
            let expected_direction = Quat::from_rotation_y(reader.opposite().yaw())
                * Quat::from_rotation_z(roll_of(expected))
                * Vec3::X;
            assert!(
                shown.distance(expected_direction) < 1e-4,
                "hanger at {} shows {shown} to a reader facing {reader:?}; expected {expected:?}",
                h.cell
            );
            let label_rotation = h.placement.transform.rotation * row.label.transform.rotation;
            let reader_left = Vec3::Y.cross(reader.dir());
            let read_order = (label_rotation * Vec3::NEG_X).dot(reader_left);
            assert!(read_order < -0.99, "EXIT reads mirrored at {}", h.cell);
            let center_world = h.placement.transform.translation;
            let label_world = h
                .placement
                .transform
                .transform_point(row.label.transform.translation);
            let normal = -reader.dir();
            let face = (label_world - center_world).dot(normal);
            assert!(
                (face - HANGER_HALF.y).abs() < 1e-4,
                "EXIT is not on the board face at {}",
                h.cell
            );
            let arrow_world = h
                .placement
                .transform
                .transform_point(row.arrow.transform.translation);
            let across = (arrow_world - label_world).dot(reader_left);
            if expected == Arrow::Right {
                assert!(across < 0.0, "right arrow is not right at {}", h.cell);
            } else {
                assert!(across > 0.0, "arrow is not left at {}", h.cell);
            }
        }
    }
    assert!(faces >= 8, "{faces} hanger faces");
    let cross = IVec2::new(0, -5);
    let steps = route(&cells, &edges, cross, None, exit);
    assert_eq!(Arrow::toward(Side::East, steps[0].side), Arrow::Left);
    assert_eq!(Arrow::toward(Side::West, steps[0].side), Arrow::Right);
}

#[test]
fn only_exit_signs_hang_overhead() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let mut exit_labels = Vec::new();
    let mut other_signs = Vec::new();
    for placement in level
        .placements
        .iter()
        .filter(|p| is_wall_sign(p) || is_hanger(p))
    {
        for piece in &placement.pieces {
            if piece.scene != "sign_arrow"
                && !label_scene(piece.scene)
                && piece.scene != "sign_hanger"
            {
                continue;
            }
            let world = placement.transform.mul_transform(piece.transform);
            if piece.scene == "sign_hanger" || on_wall(&edges, world.translation, world.rotation) {
                continue;
            }
            if piece.scene == "sign_label_exit" {
                exit_labels.push(world);
            } else {
                other_signs.push((piece.scene, world));
            }
        }
    }
    let mut arrows = 0;
    for (scene, world) in &other_signs {
        assert_eq!(*scene, "sign_arrow", "{scene} hangs");
        let normal = world.rotation * Vec3::NEG_Z;
        assert!(
            exit_labels.iter().any(|label| {
                let offset = world.translation - label.translation;
                offset.y.abs() < 1e-4
                    && offset.dot(normal).abs() < 1e-4
                    && (label.rotation * Vec3::NEG_Z).dot(normal) > 0.999
                    && offset.length() < HANGER_HALF.x
            }),
            "a hanging arrow at {} is not on an EXIT row",
            world.translation
        );
        arrows += 1;
        assert!(world.translation.y - LABEL_HALF.y >= HEAD_CLEARANCE);
    }
    for label in &exit_labels {
        assert!(label.translation.y - LABEL_HALF.y >= HEAD_CLEARANCE);
    }
    assert!(arrows >= 8, "{arrows} hanging arrows");
    let m = manifest();
    let hanging: Vec<_> = shipped_modules()
        .into_iter()
        .filter(|module| entry(&m, module).contains("\"snap\": \"ceiling_hang\""))
        .collect();
    assert_eq!(hanging, ["sign_hanger"]);
}

#[test]
fn hangers_clear_heads_lamps_walls_and_doors() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let m = manifest();
    let (lo, hi) = bounds_of(&m, "sign_hanger");
    assert!(lo.y >= HEAD_CLEARANCE);
    assert!((hi.y - CEILING).abs() < 1e-3);
    assert!(hi.x <= HANGER_HALF.x + 1e-3 && hi.z <= HANGER_HALF.y + 1e-3);
    assert!(HANGER_ROW - LABEL_HALF.y >= lo.y);
    let (llo, lhi) = bounds_of(&m, "sign_label_exit");
    assert!((lhi.x - LABEL_HALF.x).abs() < 1e-3 && (lhi.y - LABEL_HALF.y).abs() < 1e-3);
    assert!(llo.z >= -0.05);
    const { assert!(ROW_ARROW_X + 0.1 <= HANGER_HALF.x) };
    const { assert!(ROW_LABEL_X - LABEL_HALF.x >= -HANGER_HALF.x) };
    const { assert!(ROW_ARROW_X - 0.1 > ROW_LABEL_X + LABEL_HALF.x) };
    let zones = door_zones(&edges);
    let ceiling_lamps: Vec<_> = level
        .placements
        .iter()
        .filter(|p| p.pieces[0].scene.starts_with("ceiling_light_"))
        .collect();
    for h in hangers(&level) {
        let b = hanger_bounds(h.cell, h.facing);
        let area = cells[&h.cell];
        for corner in [b.0, b.1] {
            for d in [-WALL_CLEARANCE, WALL_CLEARANCE] {
                let p = Vec3::new(corner.x + d, 0.0, corner.y + d);
                assert_eq!(cells.get(&cell_of(p)), Some(&area));
            }
        }
        for lamp in &ceiling_lamps {
            let along_z = (lamp.transform.rotation * Vec3::NEG_Z).distance(Side::West.dir()) < 1e-3;
            let l = lamp_bounds(lamp.cell, along_z);
            assert!(
                b.1.x <= l.0.x || l.1.x <= b.0.x || b.1.y <= l.0.y || l.1.y <= b.0.y,
                "hanger at {} touches lamp at {}",
                h.cell,
                lamp.cell
            );
        }
        for z in &zones {
            assert!(b.1.x <= z.0.x || z.1.x <= b.0.x || b.1.y <= z.0.y || z.1.y <= b.0.y);
        }
    }
}

#[test]
fn wall_signs_are_readable_from_their_viewpoint() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let posts = Level::posts(&edges);
    let hanger_specs = hangers(&level);
    let m = manifest();
    for sign in wall_signs(&level) {
        let walk: Vec<_> = [-0.9, 0.0, 0.9]
            .iter()
            .map(|d| cell_center(sign.cell) + sign.side.dir() * *d + Vec3::Y * EYE)
            .collect();
        let normal = sign.side.opposite().dir();
        for row in rows_of(sign.placement) {
            let label_world = sign
                .placement
                .transform
                .mul_transform(row.label.transform)
                .translation;
            let arrow_world = sign
                .placement
                .transform
                .mul_transform(row.arrow.transform)
                .translation;
            let reach = cap_of(&m, row.label.scene) * READ_PER_CAP;
            let tries: Vec<_> = walk
                .iter()
                .map(|&eye| {
                    readable_from(
                        &edges,
                        &hanger_specs,
                        &posts,
                        eye,
                        label_world,
                        normal,
                        reach,
                    )
                })
                .collect();
            assert!(
                tries.iter().any(Result::is_ok),
                "{} at {} is not readable from {}: {tries:?}",
                row.label.scene,
                sign.cell,
                sign.cell
            );
            for part in [label_world, arrow_world] {
                assert!(
                    walk.iter().any(|&eye| line_of_sight(
                        &edges,
                        &hanger_specs,
                        &posts,
                        eye,
                        part - normal * 0.05
                    )
                    .is_ok()),
                    "{} at {}: part at {part} is hidden",
                    row.label.scene,
                    sign.cell
                );
            }
        }
    }
}

#[test]
fn reception_signs_follow_the_new_routes() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let exit = exit_door(&level);
    let signs = wall_signs(&level);
    for (cell, expected) in [
        (
            IVec2::new(-1, -6),
            [
                ("sign_label_exit", Arrow::Right),
                ("sign_label_maintenance", Arrow::Right),
            ],
        ),
        (
            IVec2::new(1, -6),
            [
                ("sign_label_storage", Arrow::Right),
                ("sign_label_office", Arrow::Right),
            ],
        ),
    ] {
        let sign = signs.iter().find(|s| s.cell == cell).unwrap();
        for ((dest_label, arrow), row) in expected.into_iter().zip(rows_of(sign.placement)) {
            assert_eq!(row.label.scene, dest_label);
            let dest_index = dest_area(dest_label).map(|name| room(&level, name));
            assert_eq!(
                expected_arrow(&cells, &edges, cell, sign.side, dest_index, exit),
                arrow
            );
        }
    }
    assert!(hangers(&level)
        .iter()
        .any(|h| h.cell == IVec2::new(0, -5) && h.facing == Side::East));
}

#[test]
fn readable_text_and_mount_heights() {
    let level = first_floor();
    let m = manifest();
    for (label, _) in DESTINATIONS {
        let cap = cap_of(&m, label);
        assert!(cap >= MIN_CAP_M, "{label} cap {cap}");
    }
    for placement in level
        .placements
        .iter()
        .filter(|p| is_wall_fixture(p) && label_scene(p.pieces[0].scene))
    {
        assert!(placement.transform.translation.y - LABEL_HALF.y >= HEAD_CLEARANCE);
    }
    for sign in wall_signs(&level) {
        let height = sign.placement.transform.translation.y;
        let rows = rows_of(sign.placement).len();
        let drop = (rows - 1) as f32 * ROW_PITCH;
        let bottom = height - drop - LABEL_HALF.y;
        let top = height + LABEL_HALF.y;
        assert!(bottom >= TRIM_BANDS[1].1);
        assert!(top <= TRIM_BANDS[2].0);
    }
}

#[test]
fn door_plaques_name_the_room_behind_their_door() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let mut plaques = 0;
    for placement in level
        .placements
        .iter()
        .filter(|p| is_wall_fixture(p) && label_scene(p.pieces[0].scene))
    {
        plaques += 1;
        let face = facing_of(&placement.transform);
        let side = face.opposite();
        let scene = placement.pieces[0].scene;
        let edge = edges[&edge_key(placement.cell, side)];
        assert!(matches!(edge.kind, EdgeKind::Door(_)), "{scene}");
        let n = placement.cell + side.offset();
        let behind = level.rooms[cells[&n]].name;
        assert_eq!(Some(behind), dest_area(scene));
    }
    assert!(plaques >= 4);
}

#[test]
fn route_colors_match_sign_chips() {
    let m = manifest();
    for (line, label, material) in [
        (
            "route_line_orange",
            "sign_label_boiler_room",
            "route_orange",
        ),
        ("route_line_blue", "sign_label_storage", "route_blue"),
        ("route_line_green", "sign_label_exit", "route_green"),
    ] {
        assert!(
            entry(&m, line).contains(&format!("\"{material}\"")),
            "{line}"
        );
        assert!(
            entry(&m, label).contains(&format!("\"{material}\"")),
            "{label} chip"
        );
    }
    for label in ["sign_label_maintenance", "sign_label_office"] {
        assert!(!entry(&m, label).contains("\"route_"));
    }
}

#[test]
fn hiding_affordances_and_the_tank_are_labeled_as_visual_concepts() {
    let level = first_floor();
    let m = manifest();
    for module in [
        "concept_locker",
        "concept_table",
        "concept_crawl_vent",
        "vent_grille",
    ] {
        let e = entry(&m, module);
        assert!(e.contains("\"category\": \"concept\""), "{module}");
        assert!(
            e.contains("No hiding, collision, or interaction."),
            "{module}"
        );
    }
    let tank = entry(&m, "concept_containment_tank");
    assert!(tank.contains("\"category\": \"concept\""));
    assert!(tank.contains("first-encounter"));
    assert!(tank.contains("No monster, encounter, collision, or interaction."));
    for module in [
        "concept_locker",
        "concept_table",
        "concept_crawl_vent",
        "concept_containment_tank",
    ] {
        assert!(
            level
                .placements
                .iter()
                .any(|p| p.pieces.iter().any(|piece| piece.scene == module)),
            "{module} placed"
        );
    }
}

#[test]
fn the_broken_tank_faces_the_east_lab_door_with_its_lamp() {
    let level = first_floor();
    let cells = level.cells();
    let lab = room(&level, "lab");
    let tank = level
        .placements
        .iter()
        .find(|p| p.pieces[0].scene == "concept_containment_tank")
        .expect("tank placed");
    assert_eq!(cells[&tank.cell], lab);
    let door = level
        .openings
        .iter()
        .find(|o| {
            cells.get(&o.cell) == Some(&lab)
                && o.side == Side::East
                && o.door.is_some_and(|d| d.state == DoorState::Open)
        })
        .expect("open east lab door");
    let outside = door.cell + door.side.offset();
    assert_eq!(level.rooms[cells[&outside]].name, "east_hall");
    assert!(level
        .openings
        .iter()
        .any(|o| o.cell == IVec2::new(0, -2) && o.side == Side::South && o.door.is_some()));
    let center = tank.transform.translation;
    let to_door = (half_point(edge_key(door.cell, door.side)) - center).normalize();
    let front = tank.transform.rotation * Vec3::NEG_Z;
    assert!(
        front.dot(to_door) > 0.8,
        "tank breach faces away from the door"
    );
    assert!(tank.lights.iter().any(|l| tank
        .transform
        .transform_point(l.position)
        .distance(center + TANK_LAMP)
        < 1e-4));
    let traces = level
        .placements
        .iter()
        .filter(|p| cells.get(&p.cell) == Some(&lab) && p.pieces[0].scene == "trace_drag_marks")
        .count();
    assert!(traces >= 1);
}

#[test]
fn clutter_keeps_every_room_walkable_from_its_doors() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let props: Vec<_> = level
        .placements
        .iter()
        .filter(|p| is_prop(p))
        .map(|p| (p, p.pieces[0].scene))
        .collect();
    let (mut lo, mut hi) = (IVec2::splat(i32::MAX), IVec2::splat(i32::MIN));
    for &cell in cells.keys() {
        let c = cell_center(cell);
        let a = snap(Vec2::new(c.x - HALF, c.z - HALF));
        let b = snap(Vec2::new(c.x + HALF, c.z + HALF));
        lo = lo.min(a);
        hi = hi.max(b);
    }
    let point = |k: IVec2| Vec2::new(k.x as f32, k.y as f32) * GRID;
    let mut seeds = Vec::new();
    for (&key, edge) in &edges {
        if matches!(edge.kind, EdgeKind::Wall(_)) {
            continue;
        }
        let mid = half_point(key);
        for sign in [1.0, -1.0] {
            let p = mid + edge.facing.dir() * sign * (WALL_T / 2.0 + BODY + GRID);
            if cells.contains_key(&cell_of(p)) {
                let k = snap(Vec2::new(p.x, p.z));
                assert!(
                    free(&cells, &edges, &props, point(k)),
                    "doorway at {key:?} is blocked"
                );
                seeds.push(k);
            }
        }
    }
    let mut reached = HashSet::from([seeds[0]]);
    let mut queue = vec![seeds[0]];
    while let Some(k) = queue.pop() {
        for d in [
            IVec2::new(1, 0),
            IVec2::new(-1, 0),
            IVec2::new(0, 1),
            IVec2::new(0, -1),
        ] {
            let n = k + d;
            if n.x < lo.x || n.x > hi.x || n.y < lo.y || n.y > hi.y || reached.contains(&n) {
                continue;
            }
            if free(&cells, &edges, &props, point(n)) {
                reached.insert(n);
                queue.push(n);
            }
        }
    }
    for seed in &seeds {
        assert!(reached.contains(seed), "doorway {seed:?} is cut off");
    }
    for &cell in cells.keys() {
        let c = cell_center(cell);
        let k = snap(Vec2::new(c.x, c.z));
        if free(&cells, &edges, &props, point(k)) {
            assert!(reached.contains(&k), "cell {cell} is cut off by props");
        }
    }
}

#[test]
fn the_workshop_island_stands_free_with_aisles() {
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    let maintenance = room(&level, "maintenance");
    let room_cells: Vec<_> = cells
        .iter()
        .filter(|(_, &a)| a == maintenance)
        .map(|(&k, _)| k)
        .collect();
    let island = level
        .placements
        .iter()
        .find(|p| p.pieces[0].scene == "work_island")
        .expect("work island");
    assert!(room_cells.contains(&island.cell));
    let (lo, hi) = prop_bounds(
        island.transform.translation,
        "work_island",
        facing_of(&island.transform),
    );
    let center = (lo + hi) / 2.0;
    let (min, max) = room_cells.iter().fold(
        (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)),
        |(a, b), &c| {
            let p = Vec2::new(c.x as f32, c.y as f32) * TILE;
            (a.min(p - HALF), b.max(p + HALF))
        },
    );
    let margin = (center - min).min(max - center);
    assert!(margin.min_element() >= 1.5, "island is not central");
    let (lo, hi) = (lo - AISLE, hi + AISLE);
    let n = ((hi - lo) / GRID).ceil();
    let other_props: Vec<_> = level
        .placements
        .iter()
        .filter(|p| is_prop(p) && p.pieces[0].scene != "work_island")
        .map(|p| (p, p.pieces[0].scene))
        .collect();
    for i in 0..=n.x as i32 {
        for j in 0..=n.y as i32 {
            if i != 0 && j != 0 && i != n.x as i32 && j != n.y as i32 {
                continue;
            }
            let p = (lo + Vec2::new(i as f32, j as f32) * GRID).min(hi);
            assert!(
                free(&cells, &edges, &other_props, p),
                "island aisle blocked at {p}"
            );
        }
    }
    let clutter = level
        .placements
        .iter()
        .filter(|p| cells.get(&p.cell) == Some(&maintenance))
        .filter(|p| {
            [
                "clutter_papers",
                "clutter_tools",
                "chair_tipped",
                "drum_spilled",
            ]
            .contains(&p.pieces[0].scene)
        })
        .count();
    assert!(clutter >= 3, "{clutter} clutter props");
}

#[test]
fn walkable_props_match_the_manifest() {
    let m = manifest();
    for scene in [
        "clutter_papers",
        "trace_drag_marks",
        "clutter_tools",
        "chair_tipped",
        "drum_spilled",
        "work_island",
        "lab_console",
        "concept_containment_tank",
    ] {
        let e = entry(&m, scene);
        let walkable = WALKABLE.contains(&scene);
        assert_eq!(e.contains("\"walkable\": true"), walkable, "{scene}");
        if walkable {
            assert!(bounds_of(&m, scene).1.y <= 0.03, "{scene}");
        }
    }
}

#[test]
fn prop_footprints_cover_the_generated_meshes() {
    let m = manifest();
    for scene in [
        "boiler_unit",
        "shelf_unit_low",
        "shelf_unit_bins",
        "concept_locker",
        "concept_table",
        "vent_grille",
        "concept_containment_tank",
        "lab_console",
        "work_island",
        "clutter_papers",
        "clutter_tools",
        "chair_tipped",
        "drum_spilled",
        "trace_drag_marks",
    ] {
        let (lo, hi) = bounds_of(&m, scene);
        let half = half_size(scene);
        assert!(
            -lo.x <= half.x + 1e-3
                && hi.x <= half.x + 1e-3
                && -lo.z <= half.y + 1e-3
                && hi.z <= half.y + 1e-3,
            "{scene} mesh {lo}..{hi} exceeds {half}"
        );
        assert!(lo.y >= -1e-3 && hi.y <= CEILING + 1e-3);
    }
    for scene in [
        "pipe_manifold",
        "tool_pegboard",
        "concept_crawl_vent",
        "trace_claw_marks",
        "fuse_panel",
    ] {
        let (lo, hi) = bounds_of(&m, scene);
        let half = half_size(scene);
        assert!(
            -lo.x <= half.x + 1e-3
                && hi.x <= half.x + 1e-3
                && -lo.y <= half.y + 1e-3
                && hi.y <= half.y + 1e-3,
            "{scene} mesh {lo}..{hi} exceeds {half}"
        );
    }
    let fire = numbers(entry(&m, "boiler_unit"), "light_anchor_bevy");
    assert!(Vec3::from_slice(&fire).distance(FIREBOX) < 0.15);
    let (_, tank) = bounds_of(&m, "concept_containment_tank");
    assert!(TANK_LAMP.y < tank.y);
}

#[test]
fn player_start_is_inside_an_area() {
    let level = first_floor();
    let cells = level.cells();
    let (cell, _) = level.start.expect("level has a player start");
    assert!(cells.contains_key(&cell));
}
