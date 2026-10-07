use std::f32::consts::TAU;

use bevy::{
    asset::AssetPath, ecs::query::QueryFilter, platform::collections::HashMap, prelude::*,
    state::app::StatesPlugin, world_serialization::WorldAssetRoot,
};
use bevy_asset_loader::mapped::MapKey;
use game_assets::{FacilityAssets, GameAssetsState};
use gameplay::{
    facility::{
        door_panel,
        grid::{cell_center, edge_key, half_point, Side},
        level::{EdgeKind, Glow, Level},
        spawn_level, Boundary, DoorPanel, DoorState, Doorway, Facility, FacilityPlugin, FloorCell,
        Glowing, Objective, Overhead, Passage, PlayerStart, Prop, Room, Wall, WallPost,
    },
    levels::first_floor,
};

fn headless() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FacilityPlugin::new(first_floor()));
    app.finish();
    app.update();
    app
}

fn windowed() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        AssetPlugin::default(),
        TransformPlugin,
    ))
    .init_asset::<StandardMaterial>()
    .init_state::<GameAssetsState>()
    .add_plugins(FacilityPlugin::new(first_floor()));
    app.finish();
    app.update();
    app
}

fn assets() -> FacilityAssets {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/facility/modules");
    let modules = std::fs::read_dir(dir)
        .expect("promoted modules")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension()? == "glb").then(|| path.file_stem()?.to_str().map(String::from))?
        })
        .map(|module| {
            let path = AssetPath::from(format!("facility/modules/{module}.glb#Scene0"));
            (MapKey::from_asset_path(&path), Handle::default())
        })
        .collect::<HashMap<_, _>>();
    FacilityAssets { modules }
}

fn enter(app: &mut App, state: GameAssetsState) {
    app.world_mut()
        .resource_mut::<NextState<GameAssetsState>>()
        .set(state);
    app.update();
}

fn ready(app: &mut App) {
    app.insert_resource(assets());
    enter(app, GameAssetsState::Ready);
}

fn count<F: QueryFilter>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, F>()
        .iter(app.world())
        .count()
}

fn expected_visuals(level: &Level) -> (usize, usize) {
    let cells = level.cells();
    let edges = level.edges(&cells);
    let structure: usize = edges
        .values()
        .map(|edge| match edge.kind {
            EdgeKind::Wall(_) => 1,
            EdgeKind::Passage => 0,
            EdgeKind::Door(_) => 2,
        })
        .sum();
    let pieces: usize = level.placements.iter().map(|p| p.pieces.len()).sum();
    let lights = level.placements.iter().map(|p| p.lights.len()).sum();
    (
        cells.len() * 2 + structure + Level::posts(&edges).len() + pieces,
        lights,
    )
}

#[test]
fn headless_builds_the_logical_level_without_assets_or_visuals() {
    let mut app = headless();
    let level = first_floor();
    let cells = level.cells();
    let edges = level.edges(&cells);
    assert!(!app.world().contains_resource::<AssetServer>());
    assert_eq!(count::<With<Facility>>(&mut app), 1);
    assert_eq!(count::<With<Room>>(&mut app), 16);
    assert_eq!(count::<With<FloorCell>>(&mut app), cells.len());
    assert_eq!(count::<With<Boundary>>(&mut app), edges.len());
    assert_eq!(count::<With<Doorway>>(&mut app), 15);
    assert_eq!(count::<With<Passage>>(&mut app), 8);
    assert_eq!(count::<With<Wall>>(&mut app), edges.len() - 15 - 8);
    assert_eq!(
        count::<With<WallPost>>(&mut app),
        Level::posts(&edges).len()
    );
    assert_eq!(count::<With<Prop>>(&mut app), level.placements.len());
    assert_eq!(count::<With<Visibility>>(&mut app), 0);
    assert_eq!(count::<With<WorldAssetRoot>>(&mut app), 0);
    assert_eq!(count::<With<PointLight>>(&mut app), 0);
}

#[test]
fn logical_entities_sit_at_their_declared_places() {
    let mut app = headless();
    let level = first_floor();
    let edges = level.edges(&level.cells());
    let world = app.world_mut();
    for (cell, transform) in world.query::<(&FloorCell, &Transform)>().iter(world) {
        assert_eq!(transform.translation, cell_center(cell.cell));
    }
    for (boundary, transform) in world.query::<(&Boundary, &Transform)>().iter(world) {
        assert_eq!(edges[&boundary.key].facing, boundary.facing);
        assert_eq!(transform.translation, half_point(boundary.key));
        assert_eq!(
            transform.rotation,
            Quat::from_rotation_y(boundary.facing.yaw())
        );
    }
    let mut placed: Vec<_> = world
        .query_filtered::<&Transform, With<Prop>>()
        .iter(world)
        .copied()
        .collect();
    for placement in &level.placements {
        let index = placed
            .iter()
            .position(|transform| *transform == placement.transform)
            .unwrap_or_else(|| panic!("{:?} is not spawned", placement.pieces));
        placed.swap_remove(index);
    }
    assert!(placed.is_empty());
}

#[test]
fn objectives_and_player_start_are_marked() {
    let mut app = headless();
    let world = app.world_mut();
    let exits: Vec<_> = world
        .query::<(&Objective, &DoorState, &Boundary)>()
        .iter(world)
        .map(|(objective, state, boundary)| (*objective, *state, boundary.key))
        .collect();
    assert_eq!(
        exits,
        [(
            Objective::Exit,
            DoorState::Locked,
            edge_key(IVec2::new(0, -12), Side::North)
        )]
    );
    let doors: Vec<_> = world.query::<&DoorState>().iter(world).copied().collect();
    assert_eq!(doors.iter().filter(|s| **s == DoorState::Open).count(), 14);
    let panels: Vec<_> = world
        .query_filtered::<(&Objective, &ChildOf), With<Prop>>()
        .iter(world)
        .map(|(objective, parent)| (*objective, parent.parent()))
        .collect();
    assert_eq!(panels.len(), 1);
    assert_eq!(panels[0].0, Objective::FusePanel);
    assert_eq!(world.get::<Room>(panels[0].1).unwrap().name, "exit");

    let (start, parent) = world
        .query_filtered::<(&Transform, &ChildOf), With<PlayerStart>>()
        .single(world)
        .unwrap();
    assert_eq!(start.translation, Vec3::new(0.0, 0.0, -5.0));
    assert_eq!(world.get::<Room>(parent.parent()).unwrap().name, "intake");
}

#[test]
fn despawning_the_facility_removes_every_entity() {
    let mut app = windowed();
    ready(&mut app);
    let root = app
        .world_mut()
        .query_filtered::<Entity, With<Facility>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(root).despawn();
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(count::<With<Boundary>>(&mut app), 0);
    assert_eq!(count::<With<Prop>>(&mut app), 0);
    assert_eq!(count::<With<PlayerStart>>(&mut app), 0);
    assert_eq!(count::<With<WorldAssetRoot>>(&mut app), 0);
    assert_eq!(count::<With<PointLight>>(&mut app), 0);
}

#[test]
fn windowed_build_waits_for_assets() {
    let mut app = windowed();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(count::<With<Facility>>(&mut app), 0);
    assert_eq!(count::<With<WorldAssetRoot>>(&mut app), 0);
    assert_eq!(count::<With<PlayerStart>>(&mut app), 0);
}

#[test]
fn ready_builds_logic_and_visuals_once_in_one_pass() {
    let mut app = windowed();
    ready(&mut app);
    let (pieces, lights) = expected_visuals(&first_floor());
    for _ in 0..3 {
        assert_eq!(count::<With<Facility>>(&mut app), 1);
        assert_eq!(count::<With<WorldAssetRoot>>(&mut app), pieces);
        assert_eq!(count::<With<PointLight>>(&mut app), lights);
        app.update();
    }
    let world = app.world_mut();
    let parents: Vec<_> = world
        .query_filtered::<&ChildOf, Or<(With<WorldAssetRoot>, With<PointLight>)>>()
        .iter(world)
        .map(ChildOf::parent)
        .collect();
    for parent in parents {
        let entity = world.entity(parent);
        assert!(
            entity.contains::<FloorCell>()
                || entity.contains::<Boundary>()
                || entity.contains::<WallPost>()
                || entity.contains::<Prop>(),
            "visual child of a non-logical entity"
        );
        assert!(entity.contains::<Visibility>());
    }
}

#[test]
fn rendered_placements_keep_their_declared_world_placement() {
    let mut app = windowed();
    ready(&mut app);
    app.update();
    let level = first_floor();
    let world = app.world_mut();
    let mut placed: Vec<_> = world
        .query_filtered::<(&Name, &GlobalTransform), With<WorldAssetRoot>>()
        .iter(world)
        .map(|(name, transform)| (name.as_str().to_owned(), transform.translation()))
        .collect();
    for placement in &level.placements {
        for piece in &placement.pieces {
            let expected = placement
                .transform
                .mul_transform(piece.transform)
                .translation;
            let index = placed
                .iter()
                .position(|(name, at)| name == piece.scene && at.distance(expected) < 1e-3)
                .unwrap_or_else(|| panic!("{} at {expected} is not rendered", piece.scene));
            placed.swap_remove(index);
        }
    }
    assert!(placed.iter().all(|(name, _)| [
        "floor_tile",
        "floor_tile_marked",
        "ceiling_tile",
        "wall",
        "wall_conduit",
        "wall_doorway",
        "door_panel",
        "wall_post"
    ]
    .contains(&name.as_str())));
}

#[test]
fn facility_lights_do_not_render_shadow_cubemaps() {
    let mut app = windowed();
    ready(&mut app);
    let world = app.world_mut();
    let lights: Vec<_> = world.query::<&PointLight>().iter(world).collect();
    assert_eq!(lights.len(), 17);
    assert!(lights.iter().all(|light| !light.shadow_maps_enabled));
}

#[test]
fn overhead_and_glowing_markers_follow_the_declaration() {
    let mut app = windowed();
    ready(&mut app);
    let level = first_floor();
    let overhead: usize = level
        .placements
        .iter()
        .filter(|p| p.overhead)
        .map(|p| p.pieces.len())
        .sum();
    let glowing: usize = level
        .placements
        .iter()
        .filter(|p| p.glow != Glow::Steady)
        .map(|p| p.pieces.len())
        .sum();
    assert_eq!(overhead, 20);
    assert_eq!(glowing, 7);
    assert_eq!(
        count::<With<Overhead>>(&mut app),
        level.cells().len() + overhead
    );
    assert_eq!(count::<With<Glowing>>(&mut app), glowing);
}

#[test]
fn failed_assets_build_the_level_without_visuals() {
    let mut app = windowed();
    enter(&mut app, GameAssetsState::Failed);
    app.update();
    assert_eq!(count::<With<Facility>>(&mut app), 1);
    assert_eq!(count::<With<PlayerStart>>(&mut app), 1);
    assert_eq!(count::<With<WorldAssetRoot>>(&mut app), 0);
    assert_eq!(count::<With<PointLight>>(&mut app), 0);
}

#[test]
fn spawn_level_builds_a_second_level_with_its_own_visuals() {
    let mut app = windowed();
    ready(&mut app);
    let (pieces, lights) = expected_visuals(&first_floor());
    let assets = assets();
    let mut commands = app.world_mut().commands();
    spawn_level(&mut commands, &first_floor(), Some(&assets));
    app.world_mut().flush();
    app.update();
    assert_eq!(count::<With<Facility>>(&mut app), 2);
    assert_eq!(count::<With<WorldAssetRoot>>(&mut app), pieces * 2);
    assert_eq!(count::<With<PointLight>>(&mut app), lights * 2);
}

#[test]
fn door_panel_follows_logical_door_state() {
    let mut app = windowed();
    ready(&mut app);
    let world = app.world_mut();
    let (exit, doorway, swing) = world
        .query::<(Entity, &Objective, &Transform, &Doorway)>()
        .iter(world)
        .find(|(_, objective, ..)| **objective == Objective::Exit)
        .map(|(entity, _, transform, doorway)| (entity, *transform, doorway.swing))
        .unwrap();
    let panel = |app: &mut App| {
        let world = app.world_mut();
        let children = world.get::<Children>(exit).unwrap().to_vec();
        let panels: Vec<_> = children
            .into_iter()
            .filter(|child| world.get::<DoorPanel>(*child).is_some())
            .collect();
        assert_eq!(panels.len(), 1);
        let (_, rotation, translation) = world
            .get::<GlobalTransform>(panels[0])
            .unwrap()
            .to_scale_rotation_translation();
        (translation, rotation.to_euler(EulerRot::YXZ).0)
    };
    let expect = |state| {
        let world = doorway.mul_transform(door_panel(state, swing));
        (world.translation, world.rotation.to_euler(EulerRot::YXZ).0)
    };
    let close = |a: f32, b: f32| (a - b).rem_euclid(TAU).min((b - a).rem_euclid(TAU)) < 1e-3;

    app.update();
    let (translation, yaw) = panel(&mut app);
    let (expected, expected_yaw) = expect(DoorState::Locked);
    assert!(translation.distance(expected) < 1e-3);
    assert!(close(yaw, expected_yaw));

    *app.world_mut().get_mut::<DoorState>(exit).unwrap() = DoorState::Open;
    app.update();
    app.update();
    let (translation, yaw) = panel(&mut app);
    let (expected, expected_yaw) = expect(DoorState::Open);
    assert!(translation.distance(expected) < 1e-3);
    assert!(close(yaw, expected_yaw));
    assert!(!close(yaw, expect(DoorState::Closed).1));
}
