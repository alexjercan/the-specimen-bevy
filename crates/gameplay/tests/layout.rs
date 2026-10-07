use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bevy::math::{IVec2, Quat, Vec2, Vec3};
use gameplay::facility::layout::*;

fn modules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/facility/modules")
}

fn plan() -> Plan {
    derive(&authored()).expect("authored layout derives")
}

fn pieces(plan: &Plan, part: Part) -> Vec<&Piece> {
    plan.pieces.iter().filter(|p| p.part == part).collect()
}

fn key_of(point: Vec3) -> (i32, i32) {
    (
        (point.x / HALF).round() as i32,
        (point.z / HALF).round() as i32,
    )
}

fn on_grid(value: f32) -> bool {
    (value / HALF - (value / HALF).round()).abs() < 1e-4
}

#[test]
fn authored_layout_has_narrow_halls_and_independent_bypasses() {
    let layout = authored();
    let plan = plan();
    for name in ["service", "intake", "west_hall", "east_hall"] {
        let area = layout.areas.iter().find(|a| a.name == name).unwrap();
        assert_eq!(area.min.x, area.max.x, "{name} is wider than one tile");
    }
    let lab = layout.areas.iter().find(|a| a.name == "lab").unwrap();
    assert_eq!(lab.max.x - lab.min.x, 2);
    let reception = layout.areas.iter().find(|a| a.name == "reception").unwrap();
    assert_eq!(
        (reception.min, reception.max),
        (IVec2::new(-1, -6), IVec2::new(1, -4))
    );
    assert!(layout
        .areas
        .iter()
        .all(|a| a.name != "cross" && !a.name.ends_with("upper_link")));
    let mut edges = BTreeSet::new();
    for opening in &layout.openings {
        let cell = opening.cell;
        let other = cell + opening.side.offset();
        let Some(&a) = plan.cells.get(&(other.x, other.y)) else {
            continue;
        };
        let b = plan.cells[&(cell.x, cell.y)];
        let pair = [layout.areas[a].name, layout.areas[b].name];
        edges.insert((pair[0].min(pair[1]), pair[0].max(pair[1])));
    }
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
        assert!(edges.contains(&(a.min(b), a.max(b))), "missing {a}-{b}");
    }
    assert_eq!(edges.len(), 22, "unexpected area connection");
    assert!(!edges.contains(&("office", "storage")));
    assert!(!edges.contains(&("security", "storage")));
    assert!(!edges.contains(&("office", "security")));
    assert_eq!(
        edges
            .iter()
            .filter(|(a, b)| *a == "boiler" || *b == "boiler")
            .count(),
        1
    );
    let connected_without = |blocked: &[(&str, &str)]| {
        let mut visited = BTreeSet::from(["reception"]);
        let mut frontier = vec!["reception"];
        while let Some(from) = frontier.pop() {
            for &(a, b) in &edges {
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
fn exit_objective_and_reception_are_connected_and_clear() {
    let layout = authored();
    let plan = plan();
    let exit = layout.areas.iter().find(|a| a.name == "exit").unwrap();
    let security = layout.areas.iter().find(|a| a.name == "security").unwrap();
    let office = layout.areas.iter().find(|a| a.name == "office").unwrap();
    assert_eq!(
        (exit.min, exit.max),
        (IVec2::new(0, -12), IVec2::new(3, -9))
    );
    assert!(security.max.x - security.min.x < exit.max.x - exit.min.x);
    assert!(office.max.x < 4 && office.min.y > -9);
    assert_eq!(layout.exit, Some((IVec2::new(0, -12), Side::North)));
    assert!(layout.fixtures.iter().any(|fixture| matches!(fixture,
        Fixture::Wall { cell, mount: Mount::FusePanel, .. }
            if plan.cells[&(cell.x, cell.y)] == layout.areas.iter().position(|a| a.name == "exit").unwrap()
    )));
    let (lo, hi) = bounds_of(&manifest(), "fuse_panel");
    assert_eq!(Vec2::new(-lo.x, hi.y), Mount::FusePanel.half_size());
    assert!(hi.z <= 0.0 && lo.z >= -0.1);
    for (cell, side, to) in [
        (IVec2::new(0, -9), Side::South, "service"),
        (IVec2::new(1, -9), Side::South, "office"),
        (IVec2::new(3, -11), Side::East, "security"),
    ] {
        assert!(matches!(
            plan.edges[&edge_key(cell, side)].kind,
            EdgeKind::Doorway(_)
        ));
        let n = cell + side.offset();
        assert_eq!(layout.areas[plan.cells[&(n.x, n.y)]].name, to);
    }
    for area in &layout.areas {
        let start = area.min;
        assert!(
            path(&plan, &layout, start, Dest::Exit).is_ok(),
            "{} cannot reach EXIT",
            area.name
        );
    }
    for cell in [IVec2::new(-1, -5), IVec2::new(0, -5), IVec2::new(1, -5)] {
        assert_eq!(
            layout.areas[plan.cells[&(cell.x, cell.y)]].name,
            "reception"
        );
        assert!(!layout.props.iter().any(|prop| !prop.kind.walkable() && {
            let (lo, hi) = prop_bounds(prop);
            let p = cell_center(cell);
            p.x >= lo.x && p.x <= hi.x && p.z >= lo.y && p.z <= hi.y
        }));
    }
    assert!(layout
        .props
        .iter()
        .any(|prop| prop.kind == PropKind::Table && prop.cell == IVec2::new(-1, -4)));
}

#[test]
fn every_cell_has_one_floor_and_one_ceiling() {
    let plan = plan();
    for part in [Part::Floor, Part::Ceiling] {
        let keys: BTreeSet<_> = pieces(&plan, part)
            .iter()
            .map(|p| cell_of(p.translation))
            .collect();
        assert_eq!(keys.len(), plan.cells.len());
        assert_eq!(pieces(&plan, part).len(), plan.cells.len());
        assert!(keys.iter().all(|k| plan.cells.contains_key(k)));
    }
}

#[test]
fn boundary_is_closed_by_walls_doorways_or_passages() {
    let plan = plan();
    for (&(x, z), &area) in &plan.cells {
        for side in Side::ALL {
            let n = IVec2::new(x, z) + side.offset();
            let inside = plan.cells.get(&(n.x, n.y)) == Some(&area);
            let edge = plan.edges.get(&edge_key(IVec2::new(x, z), side));
            assert_eq!(inside, edge.is_none(), "cell ({x}, {z}) {side:?}");
            if !plan.cells.contains_key(&(n.x, n.y)) {
                assert!(edge.unwrap().solid(), "open to void at ({x}, {z}) {side:?}");
            }
        }
    }
}

#[test]
fn each_solid_edge_has_exactly_one_piece_at_its_center() {
    let plan = plan();
    let mut seen = BTreeMap::new();
    for piece in plan
        .pieces
        .iter()
        .filter(|p| matches!(p.part, Part::Wall | Part::Doorway))
    {
        assert_eq!(piece.translation.y, 0.0);
        let key = key_of(piece.translation);
        assert!(half_point(key).distance(piece.translation) < 1e-4);
        assert!(
            *seen.entry(key).and_modify(|n| *n += 1).or_insert(1) == 1,
            "two pieces on {key:?}"
        );
        let edge = plan.edges[&key];
        assert!(edge.solid());
        assert!((piece.yaw - edge.facing.yaw()).abs() < 1e-6);
    }
    let solid = plan.edges.values().filter(|e| e.solid()).count();
    assert_eq!(seen.len(), solid);
}

#[test]
fn walls_face_into_an_area() {
    let plan = plan();
    for (&key, edge) in plan.edges.iter().filter(|(_, e)| e.solid()) {
        let inside = half_point(key) + edge.facing.dir() * HALF;
        assert!(
            plan.cells.contains_key(&cell_of(inside)),
            "edge {key:?} faces void"
        );
    }
}

#[test]
fn posts_cover_corners_ends_and_style_changes() {
    let plan = plan();
    let posts: BTreeSet<_> = plan.posts.iter().copied().collect();
    let mut joints: BTreeMap<(i32, i32), Vec<(i32, i32)>> = BTreeMap::new();
    for (&key, _) in plan.edges.iter().filter(|(_, e)| e.solid()) {
        for v in edge_vertices(key) {
            joints.entry(v).or_default().push(key);
        }
    }
    for (vertex, keys) in &joints {
        assert!(vertex.0.rem_euclid(2) == 1 && vertex.1.rem_euclid(2) == 1);
        let straight = keys.len() == 2 && edge_along(keys[0]) == edge_along(keys[1]);
        let same = keys.len() == 2 && plan.edges[&keys[0]] == plan.edges[&keys[1]];
        if !straight || !same {
            let doorway_pair = keys.len() == 2
                && straight
                && keys
                    .iter()
                    .all(|k| matches!(plan.edges[k].kind, EdgeKind::Doorway(_)));
            assert!(
                posts.contains(vertex) || doorway_pair,
                "missing post at {vertex:?}"
            );
        }
    }
    assert!(posts.iter().all(|v| joints.contains_key(v)));
    assert_eq!(pieces(&plan, Part::Post).len(), posts.len());
    let unposted = joints.keys().filter(|v| !posts.contains(v)).count();
    assert!(unposted > 0, "straight runs should show wall-to-wall seams");
}

#[test]
fn modules_snap_to_the_grid() {
    let plan = plan();
    for piece in &plan.pieces {
        match piece.part {
            Part::Floor | Part::Ceiling | Part::Wall | Part::Doorway | Part::Post => {
                assert!(on_grid(piece.translation.x) && on_grid(piece.translation.z));
                assert_eq!(piece.translation.y, 0.0);
                let quarter = piece.yaw / std::f32::consts::FRAC_PI_2;
                assert!((quarter - quarter.round()).abs() < 1e-5);
            }
            _ => {}
        }
    }
}

#[test]
fn door_leaves_hang_in_their_openings() {
    let plan = plan();
    let doorways: Vec<_> = plan
        .edges
        .iter()
        .filter(|(_, e)| matches!(e.kind, EdgeKind::Doorway(_)))
        .collect();
    let doors = pieces(&plan, Part::Door);
    assert_eq!(doors.len(), doorways.len());
    for (&key, edge) in doorways {
        let mid = half_point(key);
        let right = wall_right(edge.facing);
        let door = doors
            .iter()
            .find(|d| d.translation.distance(mid) < DOOR_W)
            .expect("door leaf near doorway");
        let hinge = mid - right * (DOOR_W / 2.0 - 0.01);
        let leaf_end = door.translation + door.rotation() * Vec3::X * (DOOR_W - 0.02);
        match edge.kind {
            EdgeKind::Doorway(Door::Closed) => {
                assert!(door.translation.distance(hinge) < 1e-4);
                assert!(leaf_end.distance(mid + right * (DOOR_W / 2.0 - 0.01)) < 1e-3);
            }
            EdgeKind::Doorway(Door::Open(degrees)) => {
                assert!((door.translation - hinge).dot(right).abs() < 1e-4);
                let swing = (leaf_end - door.translation).dot(edge.facing.dir());
                assert!(swing > 0.0, "door at {key:?} swings out of its area");
                let expected = (DOOR_W - 0.02) * degrees.to_radians().sin();
                assert!((swing - expected).abs() < 1e-3);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn passages_have_no_wall_and_link_two_areas() {
    let plan = plan();
    let passages: Vec<_> = plan
        .edges
        .iter()
        .filter(|(_, e)| e.kind == EdgeKind::Passage)
        .collect();
    assert!(passages.len() >= 8);
    for (&key, edge) in passages {
        let mid = half_point(key);
        let a = plan.cells[&cell_of(mid + edge.facing.dir() * HALF)];
        let b = plan.cells[&cell_of(mid - edge.facing.dir() * HALF)];
        assert_ne!(a, b);
        assert!(plan
            .pieces
            .iter()
            .filter(|p| matches!(p.part, Part::Wall | Part::Doorway))
            .all(|p| p.translation.distance(mid) > 1e-3));
    }
}

#[test]
fn lighting_states_vary() {
    let plan = plan();
    let lights = pieces(&plan, Part::Light);
    for module in [
        "ceiling_light_cool",
        "ceiling_light_amber",
        "ceiling_light_dead",
    ] {
        assert!(lights.iter().any(|p| p.module == module), "{module}");
    }
    assert!(lights
        .iter()
        .any(|p| matches!(p.glow, Some(Glow::Flicker(_)))));
    assert!(plan.lights.iter().any(|l| l.glow == Glow::Pulse));
    let dead = lights.iter().filter(|p| p.glow.is_none()).count();
    let lit = plan
        .lights
        .iter()
        .filter(|l| (l.position.y - LAMP_HEIGHT).abs() < 1e-4)
        .count();
    assert_eq!(lights.len(), dead + lit);
    for light in &plan.lights {
        assert!(plan.cells.contains_key(&cell_of(light.position)));
        assert!(light.position.y > 0.3 && light.position.y < 3.0);
    }
}

#[test]
fn props_stay_inside_and_clear_of_doors() {
    let plan = plan();
    let layout = authored();
    let zones = door_zones(&plan);
    for prop in &layout.props {
        let (lo, hi) = prop_bounds(prop);
        for zone in &zones {
            assert!(
                hi.x <= zone.0.x || zone.1.x <= lo.x || hi.y <= zone.0.y || zone.1.y <= lo.y,
                "{prop:?} blocks a door"
            );
        }
    }
    assert_eq!(pieces(&plan, Part::Prop).len(), layout.props.len());
    let kinds: BTreeSet<_> = layout.props.iter().map(|p| p.kind.module()).collect();
    assert!(kinds.len() >= 4);
}

#[test]
fn wall_mounts_sit_on_wall_faces() {
    let plan = plan();
    for piece in pieces(&plan, Part::Mount) {
        let face = Quat::from_rotation_y(piece.yaw) * Vec3::NEG_Z;
        let back = piece.translation - face * (WALL_T / 2.0);
        let key = key_of(Vec3::new(back.x, 0.0, back.z));
        let mid = half_point(key);
        let normal_distance = (Vec3::new(back.x, 0.0, back.z) - mid).dot(face).abs();
        assert!(
            normal_distance < 1e-4,
            "{} off the wall plane",
            piece.module
        );
        assert!(plan.edges[&key].solid());
    }
}

#[test]
fn every_module_file_exists() {
    let dir = modules_dir();
    let manifest = manifest();
    let modules: BTreeSet<_> = plan().pieces.iter().map(|p| p.module).collect();
    for module in modules {
        assert!(dir.join(format!("{module}.glb")).is_file(), "{module}.glb");
        assert!(manifest.contains(&format!("\"{module}\": {{")), "{module}");
    }
}

#[test]
fn every_piece_and_light_belongs_to_an_authored_owner() {
    let layout = authored();
    let plan = plan();
    let valid = |owner: Owner| match owner {
        Owner::Cell(key) => plan.cells.contains_key(&key),
        Owner::Edge(key) => plan.edges.contains_key(&key),
        Owner::Post(key) => plan.posts.contains(&key),
        Owner::Fixture(index) => index < layout.fixtures.len(),
        Owner::Prop(index) => index < layout.props.len(),
        Owner::Line(index) => index < layout.lines.len(),
    };
    assert!(plan.pieces.iter().all(|p| valid(p.owner)));
    assert!(plan.lights.iter().all(|l| valid(l.owner)));
    for (index, fixture) in layout.fixtures.iter().enumerate() {
        let owned = plan
            .pieces
            .iter()
            .filter(|p| p.owner == Owner::Fixture(index))
            .count();
        assert!(owned > 0, "{fixture:?} has no pieces");
    }
    for piece in plan.pieces.iter().filter(|p| p.part == Part::Door) {
        let Owner::Edge(key) = piece.owner else {
            panic!("door panel is not owned by an edge");
        };
        let edge = plan.edges[&key];
        let EdgeKind::Doorway(door) = edge.kind else {
            panic!("door panel at {key:?} is not in a doorway");
        };
        let (translation, yaw) = door_panel(key, edge.facing, door);
        assert!(translation.distance(piece.translation) < 1e-5);
        assert!((yaw - piece.yaw).abs() < 1e-5);
    }
}

fn tiny() -> Layout {
    Layout {
        areas: vec![
            Area {
                name: "a",
                min: IVec2::new(0, 0),
                max: IVec2::new(0, 1),
                floor: Floor::Concrete,
                walls: WallStyle::Plain,
            },
            Area {
                name: "b",
                min: IVec2::new(1, 0),
                max: IVec2::new(1, 0),
                floor: Floor::Concrete,
                walls: WallStyle::Conduit,
            },
        ],
        ..Layout::default()
    }
}

#[test]
fn shared_walls_are_single_and_face_the_first_area() {
    let plan = derive(&tiny()).unwrap();
    let edge = plan.edges[&edge_key(IVec2::new(0, 0), Side::East)];
    assert_eq!(edge.kind, EdgeKind::Wall(WallStyle::Plain));
    assert_eq!(edge.facing, Side::West);
    assert_eq!(plan.edges.len(), 6 + 4 - 1);
    assert_eq!(plan.posts.len(), 7);
}

#[test]
fn invalid_layouts_are_rejected() {
    let mut overlap = tiny();
    overlap.areas[1].min = IVec2::new(0, 0);
    assert!(derive(&overlap).unwrap_err().contains("in both"));

    let mut inner = tiny();
    inner.openings.push(Opening {
        cell: IVec2::new(0, 0),
        side: Side::South,
        kind: OpeningKind::Passage,
    });
    assert!(derive(&inner).unwrap_err().contains("inside"));

    let mut void = tiny();
    void.openings.push(Opening {
        cell: IVec2::new(0, 0),
        side: Side::North,
        kind: OpeningKind::Passage,
    });
    assert!(derive(&void).unwrap_err().contains("void"));

    let mut passage_mount = tiny();
    passage_mount.openings.push(Opening {
        cell: IVec2::new(0, 0),
        side: Side::East,
        kind: OpeningKind::Passage,
    });
    passage_mount.fixtures.push(Fixture::Wall {
        cell: IVec2::new(0, 0),
        side: Side::East,
        offset: 0.0,
        height: 1.8,
        mount: Mount::Vent,
    });
    assert!(derive(&passage_mount).unwrap_err().contains("passage"));

    let mut trim = tiny();
    trim.fixtures.push(Fixture::Wall {
        cell: IVec2::new(0, 0),
        side: Side::West,
        offset: 0.0,
        height: 1.1,
        mount: Mount::Vent,
    });
    assert!(derive(&trim).unwrap_err().contains("trim"));

    let mut conduit = tiny();
    conduit.fixtures.push(Fixture::Wall {
        cell: IVec2::new(1, 0),
        side: Side::East,
        offset: 0.0,
        height: 2.5,
        mount: Mount::ExitSign,
    });
    assert!(derive(&conduit).unwrap_err().contains("conduit"));

    let mut frame = tiny();
    frame.openings.push(Opening {
        cell: IVec2::new(0, 0),
        side: Side::West,
        kind: OpeningKind::Door(Door::Closed),
    });
    frame.fixtures.push(Fixture::Wall {
        cell: IVec2::new(0, 0),
        side: Side::West,
        offset: 0.0,
        height: 2.0,
        mount: Mount::ExitSign,
    });
    assert!(derive(&frame).unwrap_err().contains("door frame"));

    let mut blocked = tiny();
    blocked.openings.push(Opening {
        cell: IVec2::new(1, 0),
        side: Side::West,
        kind: OpeningKind::Door(Door::Open(90.0)),
    });
    blocked.props.push(Prop {
        kind: PropKind::Drum,
        cell: IVec2::new(1, 0),
        offset: Vec2::new(-0.5, 0.0),
        facing: Side::North,
    });
    assert!(derive(&blocked).unwrap_err().contains("blocks a door"));

    let mut tight = tiny();
    tight.props.push(Prop {
        kind: PropKind::Shelf,
        cell: IVec2::new(0, 0),
        offset: Vec2::new(0.0, -0.9),
        facing: Side::South,
    });
    assert!(derive(&tight).unwrap_err().contains("closer than"));
}

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
const LINE_H: f32 = 0.003;

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
    let lo = numbers(b, "min");
    let hi = numbers(b, "max");
    (Vec3::from_slice(&lo), Vec3::from_slice(&hi))
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

struct WallSignSpec {
    cell: IVec2,
    side: Side,
    offset: f32,
    height: f32,
    rows: [Option<Row>; 3],
    reader: IVec2,
}

impl WallSignSpec {
    fn row_center(&self, drop: f32) -> Vec3 {
        let face = self.side.opposite();
        half_point(edge_key(self.cell, self.side))
            + face.dir() * (WALL_T / 2.0)
            + wall_right(face) * self.offset
            + Vec3::Y * (self.height - drop)
    }
}

fn wall_signs(layout: &Layout) -> Vec<WallSignSpec> {
    layout
        .fixtures
        .iter()
        .filter_map(|f| match *f {
            Fixture::WallSign {
                cell,
                side,
                offset,
                height,
                rows,
                reader,
            } => Some(WallSignSpec {
                cell,
                side,
                offset,
                height,
                rows,
                reader,
            }),
            _ => None,
        })
        .collect()
}

struct HangerSpec {
    cell: IVec2,
    facing: Side,
    shift: f32,
}

fn hangers(layout: &Layout) -> Vec<HangerSpec> {
    layout
        .fixtures
        .iter()
        .filter_map(|f| match *f {
            Fixture::ExitHanger {
                cell,
                facing,
                shift,
            } => Some(HangerSpec {
                cell,
                facing,
                shift,
            }),
            _ => None,
        })
        .collect()
}

fn sign_piece<'a>(plan: &'a Plan, module: &str, center: Vec3, normal: Vec3) -> &'a Piece {
    let found: Vec<_> = plan
        .pieces
        .iter()
        .filter(|p| p.module == module && p.part == Part::Sign)
        .filter(|p| (p.translation.y - center.y).abs() < 1e-4)
        .filter(|p| {
            Vec2::new(p.translation.x - center.x, p.translation.z - center.z).length() < 1.0
        })
        .filter(|p| (p.rotation() * Vec3::NEG_Z).distance(normal) < 1e-4)
        .collect();
    assert_eq!(found.len(), 1, "{module} at {center}");
    found[0]
}

fn assert_row(
    plan: &Plan,
    layout: &Layout,
    row: Row,
    reader_cell: IVec2,
    reader: Side,
    center: Vec3,
) {
    let steps = path(plan, layout, reader_cell, row.dest).unwrap();
    let end = steps.last().unwrap();
    match row.dest.area() {
        Some(name) => {
            let n = end.cell + end.side.offset();
            assert_eq!(layout.areas[plan.cells[&(n.x, n.y)]].name, name);
        }
        None => assert_eq!(Some((end.cell, end.side)), layout.exit),
    }
    assert_ne!(
        steps[0].side,
        reader.opposite(),
        "{:?} from {reader_cell} sends the reader back",
        row.dest
    );
    let decision = steps
        .iter()
        .find(|s| s.side != reader || moves(plan, s.cell).len() >= 3)
        .map_or(reader, |s| s.side);
    let normal = -reader.dir();
    let label = sign_piece(plan, row.dest.label(), center, normal);
    let arrow = sign_piece(plan, "sign_arrow", center, normal);
    let shown = arrow.rotation() * Vec3::X;
    assert!(shown.y > -0.5, "down arrow at {center}");
    let world = if shown.y > 0.5 { reader.dir() } else { shown };
    assert!(
        world.distance(decision.dir()) < 1e-4,
        "{:?} for a reader at {reader_cell} facing {reader:?}: arrow shows {world}, route goes {decision:?}",
        row.dest
    );
    if shown.y.abs() < 0.5 {
        let toward = (arrow.translation - label.translation).dot(shown);
        assert!(
            toward > 0.0,
            "arrow at {center} is not on the side it points to"
        );
    }
    let reader_left = Vec3::Y.cross(reader.dir());
    let read_order = (label.rotation() * Vec3::NEG_X).dot(reader_left);
    assert!(read_order < -0.99, "label reads mirrored at {center}");
}

#[test]
fn wall_sign_arrows_point_along_the_route_from_their_viewpoint() {
    let layout = authored();
    let plan = plan();
    let mut rows = 0;
    for sign in wall_signs(&layout) {
        for (row, drop) in sign_rows(&sign.rows) {
            rows += 1;
            assert_row(
                &plan,
                &layout,
                row,
                sign.reader,
                sign.side,
                sign.row_center(drop),
            );
        }
    }
    assert_eq!(rows, 4, "{rows} wall sign rows");
}

fn line_of_sight(plan: &Plan, layout: &Layout, from: Vec3, to: Vec3) -> Result<(), String> {
    let d = to - from;
    let n = (d.length() / 0.02).ceil() as usize;
    let mut last = cell_of(from);
    for i in 0..n {
        let p = from + d * (i as f32 / n as f32);
        for h in hangers(layout) {
            let (lo, hi) = hanger_bounds(h.cell, h.facing, h.shift);
            if (lo.x..=hi.x).contains(&p.x) && (lo.y..=hi.y).contains(&p.z) && p.y >= 2.2 {
                return Err(format!("hanger at {} blocks the view at {p}", h.cell));
            }
        }
        for &post in &plan.posts {
            let d = Vec2::new(p.x, p.z) - Vec2::new(post.0 as f32, post.1 as f32) * HALF;
            if d.abs().max_element() < POST / 2.0 {
                return Err(format!("post blocks the view at {p}"));
            }
        }
        let c = cell_of(p);
        if c == last {
            continue;
        }
        let a = IVec2::new(last.0, last.1);
        let side = Side::ALL
            .into_iter()
            .find(|s| a + s.offset() == IVec2::new(c.0, c.1))
            .ok_or(format!("view cuts a corner at {p}"))?;
        let key = edge_key(a, side);
        match plan.edges.get(&key).map(|e| e.kind) {
            None | Some(EdgeKind::Passage) => {}
            Some(EdgeKind::Doorway(_))
                if (p - half_point(key)).dot(edge_along(key)).abs() < DOOR_W / 2.0
                    && p.y < DOOR_H => {}
            Some(_) => return Err(format!("wall blocks the view at {p}")),
        }
        last = c;
    }
    Ok(())
}

fn readable_from(
    plan: &Plan,
    layout: &Layout,
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
    line_of_sight(plan, layout, eye, center - normal * 0.05)
}

#[test]
fn wall_signs_are_readable_from_their_viewpoint() {
    let layout = authored();
    let plan = plan();
    let m = manifest();
    for sign in wall_signs(&layout) {
        let walk: Vec<_> = [-0.9, 0.0, 0.9]
            .iter()
            .map(|d| cell_center(sign.reader) + sign.side.dir() * *d + Vec3::Y * EYE)
            .collect();
        let normal = sign.side.opposite().dir();
        for (row, drop) in sign_rows(&sign.rows) {
            let center = sign.row_center(drop);
            let reach = cap_of(&m, row.dest.label()) * READ_PER_CAP;
            let tries: Vec<_> = walk
                .iter()
                .map(|eye| readable_from(&plan, &layout, *eye, center, normal, reach))
                .collect();
            assert!(
                tries.iter().any(Result::is_ok),
                "{:?} at {} is not readable from {}: {tries:?}",
                row.dest,
                sign.cell,
                sign.reader
            );
            let label = sign_piece(&plan, row.dest.label(), center, -sign.side.dir());
            let arrow = sign_piece(&plan, "sign_arrow", center, -sign.side.dir());
            for part in [label.translation, arrow.translation] {
                assert!(
                    walk.iter().any(|eye| line_of_sight(
                        &plan,
                        &layout,
                        *eye,
                        part - normal * 0.05
                    )
                    .is_ok()),
                    "{:?} at {}: part at {part} is hidden",
                    row.dest,
                    sign.cell
                );
            }
        }
    }
}

#[test]
fn exit_hangers_show_the_exit_direction_on_both_faces() {
    let layout = authored();
    let plan = plan();
    let mut faces = 0;
    for h in hangers(&layout) {
        let center = cell_center(h.cell) + h.facing.dir() * h.shift;
        let steps = path(&plan, &layout, h.cell, Dest::Exit).unwrap();
        assert_eq!(
            Some((steps.last().unwrap().cell, steps.last().unwrap().side)),
            layout.exit
        );
        for reader in hanger_readers(h.facing) {
            faces += 1;
            let rot = Quat::from_rotation_y(reader.opposite().yaw());
            let row_center = center + rot * Vec3::new(0.0, HANGER_ROW, -HANGER_HALF.y);
            let normal = -reader.dir();
            let label = sign_piece(&plan, "sign_label_exit", row_center, normal);
            let arrow = sign_piece(&plan, "sign_arrow", row_center, normal);
            let expected = relative(reader, steps[0].side).unwrap_or(Arrow::Back);
            let shown = arrow.rotation() * Vec3::X;
            let expected_direction = Quat::from_rotation_y(reader.opposite().yaw())
                * Quat::from_rotation_z(expected.roll())
                * Vec3::X;
            assert!(
                shown.distance(expected_direction) < 1e-4,
                "hanger at {} shows {shown} to a reader facing {reader:?}; expected {expected:?}",
                h.cell
            );
            let reader_left = Vec3::Y.cross(reader.dir());
            let read_order = (label.rotation() * Vec3::NEG_X).dot(reader_left);
            assert!(read_order < -0.99, "EXIT reads mirrored at {row_center}");
            let face = (label.translation - center).dot(normal);
            assert!(
                (face - HANGER_HALF.y).abs() < 1e-4,
                "EXIT at {row_center} is not on the board face"
            );
            let across = (arrow.translation - label.translation).dot(reader_left);
            if expected == Arrow::Right {
                assert!(across < 0.0, "right arrow is not right at {}", h.cell);
            } else {
                assert!(across > 0.0, "arrow is not left at {}", h.cell);
            }
        }
    }
    assert!(faces >= 8, "{faces} hanger faces");
    let cross = IVec2::new(0, -5);
    assert_eq!(
        relative(
            Side::East,
            path(&plan, &layout, cross, Dest::Exit).unwrap()[0].side
        ),
        Some(Arrow::Left)
    );
    assert_eq!(
        relative(
            Side::West,
            path(&plan, &layout, cross, Dest::Exit).unwrap()[0].side
        ),
        Some(Arrow::Right)
    );
}

#[test]
fn only_exit_signs_hang_overhead() {
    let plan = plan();
    let on_wall = |piece: &Piece| {
        let normal = piece.rotation() * Vec3::NEG_Z;
        let back = piece.translation - normal * (WALL_T / 2.0);
        let back = Vec3::new(back.x, 0.0, back.z);
        plan.edges.iter().any(|(&key, edge)| {
            let offset = back - half_point(key);
            edge.solid() && offset.dot(normal).abs() < 1e-4 && offset.cross(normal).length() <= HALF
        })
    };
    let signs = pieces(&plan, Part::Sign);
    let exit_labels: Vec<_> = signs
        .iter()
        .filter(|p| p.module == "sign_label_exit" && !on_wall(p))
        .collect();
    let mut arrows = 0;
    for piece in &signs {
        if piece.module == "sign_hanger" || on_wall(piece) {
            continue;
        }
        if piece.module == "sign_arrow" {
            let normal = piece.rotation() * Vec3::NEG_Z;
            assert!(
                exit_labels.iter().any(|label| {
                    let offset = piece.translation - label.translation;
                    offset.y.abs() < 1e-4
                        && offset.dot(normal).abs() < 1e-4
                        && (label.rotation() * Vec3::NEG_Z).dot(normal) > 0.999
                        && offset.length() < HANGER_HALF.x
                }),
                "a hanging arrow at {} is not on an EXIT row",
                piece.translation
            );
            arrows += 1;
        } else {
            assert_eq!(piece.module, "sign_label_exit", "{} hangs", piece.module);
        }
        assert!(piece.translation.y - LABEL_HALF.y >= HEAD_CLEARANCE);
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
fn reception_signs_follow_the_new_routes() {
    let layout = authored();
    let plan = plan();
    let signs = wall_signs(&layout);
    for (cell, expected) in [
        (
            IVec2::new(-1, -6),
            [
                (Dest::Exit, Arrow::Right),
                (Dest::Maintenance, Arrow::Right),
            ],
        ),
        (
            IVec2::new(1, -6),
            [(Dest::Storage, Arrow::Right), (Dest::Office, Arrow::Right)],
        ),
    ] {
        let sign = signs.iter().find(|sign| sign.reader == cell).unwrap();
        for ((dest, arrow), (row, _)) in expected.into_iter().zip(sign_rows(&sign.rows)) {
            assert_eq!((row.dest, row.arrow), (dest, arrow));
            assert_eq!(
                expected_arrow(&plan, &layout, cell, sign.side, dest).unwrap(),
                arrow
            );
        }
    }
    assert!(hangers(&layout)
        .iter()
        .any(|h| h.cell == IVec2::new(0, -5) && h.facing == Side::East));
}

#[test]
fn wrong_way_and_backward_arrows_are_rejected() {
    let mut flipped = authored();
    for f in &mut flipped.fixtures {
        if let Fixture::WallSign { rows, .. } = f {
            let row = rows[0].as_mut().unwrap();
            row.arrow = match row.arrow {
                Arrow::Left => Arrow::Right,
                Arrow::Right | Arrow::Ahead | Arrow::Back => Arrow::Left,
            };
            break;
        }
    }
    assert!(derive(&flipped).unwrap_err().contains("wrong-way"));

    let mut backward = authored();
    backward.fixtures.push(Fixture::ExitHanger {
        cell: IVec2::new(0, -2),
        facing: Side::North,
        shift: 0.95,
    });
    let plan = derive(&backward).unwrap();
    let reader = Side::South;
    let center = cell_center(IVec2::new(0, -2)) + Side::North.dir() * 0.95;
    let yaw = reader.opposite().yaw();
    let row_center =
        center + Quat::from_rotation_y(yaw) * Vec3::new(0.0, HANGER_ROW, -HANGER_HALF.y);
    let arrow = sign_piece(&plan, "sign_arrow", row_center, -reader.dir());
    assert!((arrow.rotation() * Vec3::X).distance(Vec3::NEG_Y) < 1e-4);

    let mut backward_sign = authored();
    backward_sign.fixtures.push(Fixture::WallSign {
        cell: IVec2::new(0, 1),
        side: Side::South,
        offset: 0.0,
        height: 2.0,
        rows: [
            Some(Row {
                dest: Dest::Exit,
                arrow: Arrow::Ahead,
            }),
            None,
            None,
        ],
        reader: IVec2::new(0, 1),
    });
    assert!(derive(&backward_sign).unwrap_err().contains("behind"));
}

#[test]
fn hangers_clear_heads_lamps_walls_and_doors() {
    let layout = authored();
    let plan = plan();
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
    let zones = door_zones(&plan);
    for h in hangers(&layout) {
        let b = hanger_bounds(h.cell, h.facing, h.shift);
        let area = plan.cells[&(h.cell.x, h.cell.y)];
        for corner in [b.0, b.1] {
            for d in [-WALL_CLEARANCE, WALL_CLEARANCE] {
                let p = Vec3::new(corner.x + d, 0.0, corner.y + d);
                assert_eq!(plan.cells.get(&cell_of(p)), Some(&area));
            }
        }
        for f in &layout.fixtures {
            if let Fixture::Ceiling { cell, along_z, .. } = *f {
                let l = lamp_bounds(cell, along_z);
                assert!(
                    b.1.x <= l.0.x || l.1.x <= b.0.x || b.1.y <= l.0.y || l.1.y <= b.0.y,
                    "hanger at {} touches lamp at {cell}",
                    h.cell
                );
            }
        }
        for z in &zones {
            assert!(b.1.x <= z.0.x || z.1.x <= b.0.x || b.1.y <= z.0.y || z.1.y <= b.0.y);
        }
    }
}

#[test]
fn bad_hanger_mounts_are_rejected() {
    let place = |cell: (i32, i32), facing: Side, shift: f32| {
        let mut layout = authored();
        layout.fixtures.push(Fixture::ExitHanger {
            cell: IVec2::new(cell.0, cell.1),
            facing,
            shift,
        });
        derive(&layout).unwrap_err()
    };
    assert!(place((0, -2), Side::South, 0.0).contains("ceiling light"));
    const { assert!(HANGER_HALF.x + WALL_CLEARANCE + WALL_T / 2.0 <= HALF) };
    assert!(place((2, -7), Side::North, 0.0).contains("door swing"));
    assert!(place((0, -1), Side::South, 1.2).contains("leaves its cell"));
}

#[test]
fn bad_wall_signs_are_rejected() {
    let place = |cell: (i32, i32), side: Side, offset: f32, height: f32| {
        let mut layout = authored();
        layout.fixtures.push(Fixture::WallSign {
            cell: IVec2::new(cell.0, cell.1),
            side,
            offset,
            height,
            rows: [
                Some(Row {
                    dest: Dest::Exit,
                    arrow: Arrow::Ahead,
                }),
                None,
                None,
            ],
            reader: IVec2::new(0, -3),
        });
        derive(&layout).unwrap_err()
    };
    assert!(
        place((0, -3), Side::West, 0.5, 2.0).contains("post")
            || place((0, -3), Side::West, 0.5, 2.0).contains("wall")
    );
    assert!(place((0, -3), Side::West, 0.0, 1.2).contains("trim"));
    assert!(place((-5, -4), Side::West, 0.0, 2.4).contains("conduit"));
    assert!(place((0, -2), Side::South, 0.0, 2.0).contains("door frame"));
    assert!(place((0, -3), Side::North, 0.0, 2.0).contains("passage"));
}

#[test]
fn readable_text_and_mount_heights() {
    let m = manifest();
    for dest in [
        Dest::Boiler,
        Dest::Storage,
        Dest::Maintenance,
        Dest::Office,
        Dest::Exit,
    ] {
        let cap = cap_of(&m, dest.label());
        assert!(cap >= MIN_CAP_M, "{dest:?} cap {cap}");
    }
    let layout = authored();
    for f in &layout.fixtures {
        if let Fixture::Wall {
            height,
            mount: Mount::Label(_),
            ..
        } = *f
        {
            assert!(height - LABEL_HALF.y >= HEAD_CLEARANCE);
        }
    }
    for sign in wall_signs(&layout) {
        let (bottom, top) = row_span(&sign.rows, sign.height);
        assert!(bottom >= TRIM_BANDS[1].1);
        assert!(top <= TRIM_BANDS[2].0);
    }
}

#[test]
fn door_plaques_name_the_room_behind_their_door() {
    let layout = authored();
    let plan = plan();
    let mut plaques = 0;
    for f in &layout.fixtures {
        if let Fixture::Wall {
            cell,
            side,
            mount: Mount::Label(dest),
            ..
        } = *f
        {
            plaques += 1;
            let edge = plan.edges[&edge_key(cell, side)];
            assert!(matches!(edge.kind, EdgeKind::Doorway(_)), "{dest:?}");
            let n = cell + side.offset();
            let behind = layout.areas[plan.cells[&(n.x, n.y)]].name;
            assert_eq!(Some(behind), dest.area());
        }
    }
    assert!(plaques >= 4);
}

fn line_pieces<'a>(plan: &'a Plan, line: &RouteLine) -> Vec<&'a Piece> {
    let found: Vec<_> = line
        .cells()
        .map(|cell| {
            let c = cell_center(cell);
            let found: Vec<_> = pieces(plan, Part::Route)
                .into_iter()
                .filter(|p| Some(p.module) == line.dest.line())
                .filter(|p| {
                    let d = p.translation - c;
                    d.x.abs() < HALF
                        && d.z.abs() < HALF
                        && (d.dot(line.side.dir()) - LINE_OFFSET).abs() < 1e-4
                })
                .collect();
            assert_eq!(found.len(), 1, "{:?} line at {cell}", line.dest);
            found[0]
        })
        .collect();
    found
}

fn line_study() -> Layout {
    let mut layout = authored();
    layout.lines = vec![
        RouteLine {
            dest: Dest::Boiler,
            start: IVec2::new(-2, -2),
            heading: Side::South,
            cells: 3,
            side: Side::West,
        },
        RouteLine {
            dest: Dest::Boiler,
            start: IVec2::new(-2, 0),
            heading: Side::West,
            cells: 1,
            side: Side::North,
        },
        RouteLine {
            dest: Dest::Exit,
            start: IVec2::new(-1, -6),
            heading: Side::East,
            cells: 2,
            side: Side::North,
        },
        RouteLine {
            dest: Dest::Exit,
            start: IVec2::new(0, -6),
            heading: Side::North,
            cells: 2,
            side: Side::West,
        },
        RouteLine {
            dest: Dest::Storage,
            start: IVec2::new(2, -5),
            heading: Side::East,
            cells: 1,
            side: Side::South,
        },
    ];
    layout
}

#[test]
fn route_lines_run_on_the_floor_along_walkable_paths() {
    assert!(authored().lines.is_empty());
    assert!(pieces(&plan(), Part::Route).is_empty());
    let layout = line_study();
    let plan = derive(&layout).unwrap();
    assert_eq!(
        pieces(&plan, Part::Route).len(),
        layout.lines.iter().map(|l| l.cells as usize).sum::<usize>()
    );
    for line in &layout.lines {
        assert!(relative(line.heading, line.side).is_some_and(|a| a != Arrow::Ahead));
        for (cell, piece) in line.cells().zip(line_pieces(&plan, line)) {
            let steps = path(&plan, &layout, cell, line.dest).unwrap();
            if cell != line.last() {
                assert_eq!(
                    steps[0].side, line.heading,
                    "{:?} line at {cell}",
                    line.dest
                );
            }
            assert_eq!(piece.translation.y, 0.0);
            let along = piece.rotation() * Vec3::X;
            assert!(along.abs().distance(line.heading.dir().abs()) < 1e-4);
            let (lo, hi) = line_bounds(piece);
            let c = cell_center(cell);
            assert!(lo.x >= c.x - HALF - 1e-4 && hi.x <= c.x + HALF + 1e-4);
            assert!(lo.y >= c.z - HALF - 1e-4 && hi.y <= c.z + HALF + 1e-4);
        }
        let last = line.last();
        let steps = path(&plan, &layout, last, line.dest).unwrap();
        if let Some(turn) = steps.iter().find(|s| s.side != line.heading) {
            assert_eq!(
                turn.side, line.side,
                "{:?} line ends outside its turn",
                line.dest
            );
        }
    }
}

#[test]
fn route_lines_turn_on_the_inside_without_gaps() {
    let layout = line_study();
    let plan = derive(&layout).unwrap();
    let mut turns = 0;
    for a in &layout.lines {
        let first = path(&plan, &layout, a.last(), a.dest).unwrap()[0].side;
        if first != a.side {
            continue;
        }
        turns += 1;
        let b = layout
            .lines
            .iter()
            .find(|b| b.dest == a.dest && b.start == a.last())
            .unwrap_or_else(|| panic!("{:?} line does not resume at {}", a.dest, a.last()));
        assert_eq!(b.heading, a.side);
        assert_eq!(b.side, a.heading.opposite(), "{:?} turns outside", a.dest);
        let (alo, ahi) = line_bounds(line_pieces(&plan, a).last().unwrap());
        let (blo, bhi) = line_bounds(line_pieces(&plan, b)[0]);
        let gap_x = (blo.x - ahi.x).max(alo.x - bhi.x);
        let gap_z = (blo.y - ahi.y).max(alo.y - bhi.y);
        assert!(
            gap_x.max(gap_z).abs() < 1e-4 && gap_x.min(gap_z) < 0.0,
            "{:?} corner at {} has a gap or overlap",
            a.dest,
            a.last()
        );
    }
    assert!(turns >= 2, "{turns} turns");
}

#[test]
fn route_lines_cover_turning_decisions_in_the_study() {
    let layout = line_study();
    let plan = derive(&layout).unwrap();
    let lined: BTreeSet<_> = layout
        .lines
        .iter()
        .flat_map(|l| l.cells().map(move |c| (l.dest, c.x, c.y)))
        .collect();
    for (dest, approach, junction, departure) in [
        (
            Dest::Boiler,
            IVec2::new(-2, -1),
            IVec2::new(-2, 0),
            IVec2::new(-3, 0),
        ),
        (
            Dest::Exit,
            IVec2::new(-1, -6),
            IVec2::new(0, -6),
            IVec2::new(0, -7),
        ),
    ] {
        assert!(
            moves(&plan, junction).len() >= 3,
            "{junction} is not a decision"
        );
        let steps = path(&plan, &layout, approach, dest).unwrap();
        assert!(steps.iter().any(|s| s.cell == junction
            && s.side
                == if dest == Dest::Exit {
                    Side::North
                } else {
                    Side::West
                }));
        assert!(lined.contains(&(dest, approach.x, approach.y)));
        assert!(lined.contains(&(dest, junction.x, junction.y)));
        if dest == Dest::Exit {
            assert!(lined.contains(&(dest, departure.x, departure.y)));
        } else {
            assert_eq!(
                layout.areas[plan.cells[&(departure.x, departure.y)]].name,
                "boiler"
            );
        }
    }
}

#[test]
fn route_lines_stay_flat_clear_and_restrained() {
    let layout = line_study();
    let plan = derive(&layout).unwrap();
    let mut colors: BTreeMap<(i32, i32), BTreeSet<Dest>> = BTreeMap::new();
    for line in &layout.lines {
        for cell in line.cells() {
            colors
                .entry((cell.x, cell.y))
                .or_default()
                .insert(line.dest);
        }
    }
    for (cell, dests) in &colors {
        assert!(dests.len() <= 2, "{dests:?} share the cell {cell:?}");
    }
    const { assert!(LINE_OFFSET + LINE_HALF_W < STRIPE_X - STRIPE_HALF_W) };
    const { assert!(LINE_OFFSET + LINE_HALF_W < HALF - WALL_T / 2.0 - 0.1) };
    let m = manifest();
    let lines = pieces(&plan, Part::Route);
    for (i, piece) in lines.iter().enumerate() {
        let (lo, hi) = bounds_of(&m, piece.module);
        assert!(lo.y.abs() < 1e-4 && (hi.y - LINE_H).abs() < 1e-4);
        assert!((hi.z - LINE_HALF_W).abs() < 1e-3 && (lo.z + LINE_HALF_W).abs() < 1e-3);
        let e = entry(&m, piece.module);
        assert!(e.contains("\"snap\": \"floor_line\""));
        assert!(e.contains("\"walkable\": true"));
        let b = line_bounds(piece);
        for prop in &layout.props {
            let p = prop_bounds(prop);
            assert!(
                b.1.x <= p.0.x || p.1.x <= b.0.x || b.1.y <= p.0.y || p.1.y <= b.0.y,
                "line at {} touches {:?}",
                piece.translation,
                prop.kind
            );
        }
        for (&(x, z), &area) in &plan.cells {
            if layout.areas[area].floor == Floor::Striped {
                for s in stripe_bounds(IVec2::new(x, z)) {
                    assert!(
                        b.1.x <= s.0.x || s.1.x <= b.0.x || b.1.y <= s.0.y || s.1.y <= b.0.y,
                        "line at {} touches a floor stripe",
                        piece.translation
                    );
                }
            }
        }
        for other in &lines[..i] {
            let o = line_bounds(other);
            assert!(b.1.x <= o.0.x || o.1.x <= b.0.x || b.1.y <= o.0.y || o.1.y <= b.0.y);
        }
        let mid = Vec2::new(piece.translation.x, piece.translation.z);
        assert!(
            free(&plan, &layout, mid),
            "line at {mid} is not on free floor"
        );
    }
}

#[test]
fn bad_lines_are_rejected() {
    let try_line = |dest: Dest, start: (i32, i32), heading: Side, cells: i32, side: Side| {
        let mut layout = authored();
        layout.lines.push(RouteLine {
            dest,
            start: IVec2::new(start.0, start.1),
            heading,
            cells,
            side,
        });
        derive(&layout).unwrap_err()
    };
    use Side::*;
    assert!(try_line(Dest::Exit, (0, -5), South, 1, West).contains("along the route"));
    assert!(try_line(Dest::Exit, (-1, -5), East, 1, South).contains("outside of the turn"));
    assert!(try_line(Dest::Exit, (0, -5), North, 1, North).contains("one side"));
    assert!(try_line(Dest::Office, (0, -6), North, 1, West).contains("no route color"));
    let mut overlap = line_study();
    overlap.lines.push(RouteLine {
        dest: Dest::Exit,
        start: IVec2::new(0, -6),
        heading: North,
        cells: 1,
        side: West,
    });
    assert!(derive(&overlap)
        .unwrap_err()
        .contains("overlaps another line"));
    let mut lines = line_study();
    lines
        .lines
        .retain(|l| !(l.dest == Dest::Boiler && l.heading == West));
    assert!(derive(&lines).unwrap_err().contains("does not resume"));
    let mut wide = authored();
    wide.lines.push(RouteLine {
        dest: Dest::Exit,
        start: IVec2::new(0, -6),
        heading: North,
        cells: 1,
        side: West,
    });
    wide.props.push(Prop {
        kind: PropKind::Papers,
        cell: IVec2::new(0, -6),
        offset: Vec2::new(-0.55, 0.0),
        facing: North,
    });
    assert!(derive(&wide).unwrap_err().contains("crosses Papers"));
}

#[test]
fn route_colors_match_sign_chips() {
    let m = manifest();
    for (dest, material) in [
        (Dest::Boiler, "route_orange"),
        (Dest::Storage, "route_blue"),
        (Dest::Exit, "route_green"),
    ] {
        let line = entry(&m, dest.line().unwrap());
        assert!(line.contains(&format!("\"{material}\"")), "{dest:?} line");
        assert!(
            entry(&m, dest.label()).contains(&format!("\"{material}\"")),
            "{dest:?} label chip"
        );
    }
    for dest in [Dest::Maintenance, Dest::Office] {
        assert!(dest.line().is_none());
        assert!(!entry(&m, dest.label()).contains("\"route_"));
    }
}

#[test]
fn hiding_affordances_and_the_tank_are_labeled_as_visual_concepts() {
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
    let plan = plan();
    for module in [
        "concept_locker",
        "concept_table",
        "concept_crawl_vent",
        "concept_containment_tank",
    ] {
        assert!(
            plan.pieces.iter().any(|p| p.module == module),
            "{module} placed"
        );
    }
}

fn area_named(layout: &Layout, plan: &Plan, name: &str) -> Vec<IVec2> {
    let index = layout.areas.iter().position(|a| a.name == name).unwrap();
    plan.cells
        .iter()
        .filter(|(_, a)| **a == index)
        .map(|(k, _)| IVec2::new(k.0, k.1))
        .collect()
}

#[test]
fn the_broken_tank_faces_the_east_lab_door() {
    let layout = authored();
    let plan = plan();
    let lab = area_named(&layout, &plan, "lab");
    let tank = layout
        .props
        .iter()
        .find(|p| p.kind == PropKind::Tank)
        .unwrap();
    assert!(lab.contains(&tank.cell));
    let door = layout
        .openings
        .iter()
        .find(|o| {
            lab.contains(&o.cell)
                && o.side == Side::East
                && matches!(o.kind, OpeningKind::Door(Door::Open(_)))
        })
        .expect("open east lab door");
    let outside = door.cell + door.side.offset();
    assert_eq!(
        layout.areas[plan.cells[&(outside.x, outside.y)]].name,
        "east_hall"
    );
    assert!(layout.openings.iter().any(|o| o.cell == IVec2::new(0, -2)
        && o.side == Side::South
        && matches!(o.kind, OpeningKind::Door(_))));
    let center = cell_center(tank.cell) + Vec3::new(tank.offset.x, 0.0, tank.offset.y);
    let to_door = (half_point(edge_key(door.cell, door.side)) - center).normalize();
    let front = Quat::from_rotation_y(tank.facing.yaw()) * Vec3::NEG_Z;
    assert!(
        front.dot(to_door) > 0.8,
        "tank breach faces away from the door"
    );
    assert!(plan
        .lights
        .iter()
        .any(|l| l.position.distance(center + TANK_LAMP) < 1e-4));
    let traces = layout
        .props
        .iter()
        .filter(|p| lab.contains(&p.cell) && p.kind == PropKind::DragMarks)
        .count();
    assert!(traces >= 1);
}

fn free(plan: &Plan, layout: &Layout, p: Vec2) -> bool {
    let key = cell_of(Vec3::new(p.x, 0.0, p.y));
    if !plan.cells.contains_key(&key) {
        return false;
    }
    let cell = IVec2::new(key.0, key.1);
    let local = p - Vec2::new(cell.x as f32, cell.y as f32) * TILE;
    for side in Side::ALL {
        let Some(edge) = plan.edges.get(&edge_key(cell, side)) else {
            continue;
        };
        let o = side.offset().as_vec2();
        if HALF - local.dot(o) >= WALL_T / 2.0 + BODY {
            continue;
        }
        let along = local.dot(Vec2::new(-o.y, o.x)).abs();
        let open = match edge.kind {
            EdgeKind::Passage => true,
            EdgeKind::Doorway(_) => along + BODY <= DOOR_W / 2.0,
            EdgeKind::Wall(_) => false,
        };
        if !open {
            return false;
        }
    }
    layout
        .props
        .iter()
        .filter(|prop| !prop.kind.walkable())
        .all(|prop| {
            let (lo, hi) = prop_bounds(prop);
            let d = (lo - p).max(p - hi).max(Vec2::ZERO);
            d.length() >= BODY
        })
}

fn snap(p: Vec2) -> (i32, i32) {
    ((p.x / GRID).round() as i32, (p.y / GRID).round() as i32)
}

#[test]
fn clutter_keeps_every_room_walkable_from_its_doors() {
    let layout = authored();
    let plan = plan();
    let (mut lo, mut hi) = ((i32::MAX, i32::MAX), (i32::MIN, i32::MIN));
    for &(x, z) in plan.cells.keys() {
        let c = cell_center(IVec2::new(x, z));
        let a = snap(Vec2::new(c.x - HALF, c.z - HALF));
        let b = snap(Vec2::new(c.x + HALF, c.z + HALF));
        lo = (lo.0.min(a.0), lo.1.min(a.1));
        hi = (hi.0.max(b.0), hi.1.max(b.1));
    }
    let point = |k: (i32, i32)| Vec2::new(k.0 as f32, k.1 as f32) * GRID;
    let mut seeds = Vec::new();
    for (&key, edge) in &plan.edges {
        if matches!(edge.kind, EdgeKind::Wall(_)) {
            continue;
        }
        let mid = half_point(key);
        for sign in [1.0, -1.0] {
            let p = mid + edge.facing.dir() * sign * (WALL_T / 2.0 + BODY + GRID);
            if plan.cells.contains_key(&cell_of(p)) {
                let k = snap(Vec2::new(p.x, p.z));
                assert!(
                    free(&plan, &layout, point(k)),
                    "doorway at {key:?} is blocked"
                );
                seeds.push(k);
            }
        }
    }
    let mut reached = BTreeSet::from([seeds[0]]);
    let mut queue = vec![seeds[0]];
    while let Some(k) = queue.pop() {
        for d in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let n = (k.0 + d.0, k.1 + d.1);
            if n.0 < lo.0 || n.0 > hi.0 || n.1 < lo.1 || n.1 > hi.1 || reached.contains(&n) {
                continue;
            }
            if free(&plan, &layout, point(n)) {
                reached.insert(n);
                queue.push(n);
            }
        }
    }
    for seed in &seeds {
        assert!(reached.contains(seed), "doorway {seed:?} is cut off");
    }
    for &(x, z) in plan.cells.keys() {
        let c = cell_center(IVec2::new(x, z));
        let k = snap(Vec2::new(c.x, c.z));
        if free(&plan, &layout, point(k)) {
            assert!(reached.contains(&k), "cell ({x}, {z}) is cut off by props");
        }
    }
}

#[test]
fn the_workshop_island_stands_free_with_aisles() {
    let layout = authored();
    let plan = plan();
    let room = area_named(&layout, &plan, "maintenance");
    let island = layout
        .props
        .iter()
        .find(|p| p.kind == PropKind::Island)
        .expect("work island");
    assert!(room.contains(&island.cell));
    let (lo, hi) = prop_bounds(island);
    let center = (lo + hi) / 2.0;
    let (min, max) = room.iter().fold(
        (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)),
        |(a, b), c| {
            let p = Vec2::new(c.x as f32, c.y as f32) * TILE;
            (a.min(p - HALF), b.max(p + HALF))
        },
    );
    let margin = (center - min).min(max - center);
    assert!(margin.min_element() >= 1.5, "island is not central");
    let (lo, hi) = (lo - AISLE, hi + AISLE);
    let n = ((hi - lo) / GRID).ceil();
    let others = Layout {
        props: layout
            .props
            .iter()
            .filter(|p| p.kind != PropKind::Island)
            .copied()
            .collect(),
        ..layout.clone()
    };
    for i in 0..=n.x as i32 {
        for j in 0..=n.y as i32 {
            if i != 0 && j != 0 && i != n.x as i32 && j != n.y as i32 {
                continue;
            }
            let p = lo + Vec2::new(i as f32, j as f32) * GRID;
            let p = p.min(hi);
            assert!(free(&plan, &others, p), "island aisle blocked at {p}");
        }
    }
    let clutter = layout
        .props
        .iter()
        .filter(|p| room.contains(&p.cell))
        .filter(|p| {
            matches!(
                p.kind,
                PropKind::Papers | PropKind::Tools | PropKind::ChairTipped | PropKind::DrumSpilled
            )
        })
        .count();
    assert!(clutter >= 3, "{clutter} clutter props");
}

#[test]
fn walkable_props_match_the_manifest() {
    let m = manifest();
    for kind in [
        PropKind::Papers,
        PropKind::DragMarks,
        PropKind::Tools,
        PropKind::ChairTipped,
        PropKind::DrumSpilled,
        PropKind::Island,
        PropKind::Console,
        PropKind::Tank,
    ] {
        let e = entry(&m, kind.module());
        assert_eq!(
            e.contains("\"walkable\": true"),
            kind.walkable(),
            "{kind:?}"
        );
        if kind.walkable() {
            assert!(bounds_of(&m, kind.module()).1.y <= 0.03, "{kind:?}");
        }
    }
}

#[test]
fn prop_footprints_cover_the_generated_meshes() {
    let m = manifest();
    for kind in [
        PropKind::Boiler,
        PropKind::ShelfLow,
        PropKind::ShelfBins,
        PropKind::Locker,
        PropKind::Table,
        PropKind::VentGrille,
        PropKind::Tank,
        PropKind::Console,
        PropKind::Island,
        PropKind::Papers,
        PropKind::Tools,
        PropKind::ChairTipped,
        PropKind::DrumSpilled,
        PropKind::DragMarks,
    ] {
        let (lo, hi) = bounds_of(&m, kind.module());
        let half = kind.half_size();
        assert!(
            -lo.x <= half.x + 1e-3
                && hi.x <= half.x + 1e-3
                && -lo.z <= half.y + 1e-3
                && hi.z <= half.y + 1e-3,
            "{kind:?} mesh {lo}..{hi} exceeds {half}"
        );
        assert!(lo.y >= -1e-3 && hi.y <= CEILING + 1e-3);
    }
    for mount in [
        Mount::PipeManifold,
        Mount::Pegboard,
        Mount::CrawlVent,
        Mount::ClawMarks,
        Mount::FusePanel,
    ] {
        let (lo, hi) = bounds_of(&m, mount.module());
        let half = mount.half_size();
        assert!(
            -lo.x <= half.x + 1e-3
                && hi.x <= half.x + 1e-3
                && -lo.y <= half.y + 1e-3
                && hi.y <= half.y + 1e-3,
            "{mount:?} mesh {lo}..{hi} exceeds {half}"
        );
    }
    let fire = numbers(entry(&m, "boiler_unit"), "light_anchor_bevy");
    assert!(Vec3::from_slice(&fire).distance(FIREBOX) < 0.15);
    let (_, tank) = bounds_of(&m, "concept_containment_tank");
    assert!(TANK_LAMP.y < tank.y);
}

#[test]
fn player_start_must_be_inside_an_area() {
    let mut layout = tiny();
    layout.start = Some((IVec2::new(0, 1), Side::North));
    assert!(derive(&layout).is_ok());
    layout.start = Some((IVec2::new(5, 5), Side::North));
    let error = derive(&layout).unwrap_err();
    assert!(error.contains("player start"), "{error}");
}
