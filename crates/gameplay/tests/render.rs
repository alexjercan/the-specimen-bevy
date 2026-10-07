use std::f32::consts::TAU;

use bevy::{
    asset::AssetPath, platform::collections::HashMap, prelude::*, state::app::StatesPlugin,
    world_serialization::WorldAssetRoot,
};
use bevy_asset_loader::mapped::MapKey;
use game_assets::{FacilityAssets, GameAssetsState};
use gameplay::facility::{
    layout::{authored, derive, door_panel, edge_key, Door, Side},
    render::{DoorPanel, FacilityRenderPlugin, Rendered},
    spawn_facility, DoorState, Facility, FacilityGrid, FacilityPart, FacilityPlugin, Objective,
};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        AssetPlugin::default(),
        TransformPlugin,
    ))
    .init_asset::<StandardMaterial>()
    .init_state::<GameAssetsState>()
    .add_plugins((FacilityPlugin, FacilityRenderPlugin));
    app.update();
    app
}

fn ready(app: &mut App) {
    let plan = derive(&authored()).unwrap();
    let modules = plan
        .pieces
        .iter()
        .map(|piece| {
            let path = AssetPath::from(format!("facility/modules/{}.glb#Scene0", piece.module));
            (MapKey::from_asset_path(&path), Handle::default())
        })
        .collect::<HashMap<_, _>>();
    app.insert_resource(FacilityAssets { modules })
        .world_mut()
        .resource_mut::<NextState<GameAssetsState>>()
        .set(GameAssetsState::Ready);
    app.update();
}

fn count<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, F>()
        .iter(app.world())
        .count()
}

fn plan_counts() -> (usize, usize) {
    let plan = derive(&authored()).unwrap();
    (plan.pieces.len(), plan.lights.len())
}

#[test]
fn nothing_renders_until_assets_are_ready() {
    let mut app = app();
    for _ in 0..3 {
        app.update();
    }
    assert!(count::<With<FacilityPart>>(&mut app) > 0);
    assert_eq!(count::<With<WorldAssetRoot>>(&mut app), 0);
    assert_eq!(count::<With<PointLight>>(&mut app), 0);
    assert_eq!(count::<With<Rendered>>(&mut app), 0);
}

#[test]
fn ready_renders_every_piece_once_under_its_logical_owner() {
    let mut app = app();
    ready(&mut app);
    let (pieces, lights) = plan_counts();
    for _ in 0..3 {
        app.update();
        assert_eq!(count::<With<WorldAssetRoot>>(&mut app), pieces);
        assert_eq!(count::<With<PointLight>>(&mut app), lights);
    }
    assert_eq!(
        count::<(With<FacilityPart>, Without<Rendered>)>(&mut app),
        0
    );
    let world = app.world_mut();
    let parents: Vec<_> = world
        .query_filtered::<&ChildOf, With<WorldAssetRoot>>()
        .iter(world)
        .map(ChildOf::parent)
        .collect();
    for parent in parents {
        assert!(world.get::<FacilityPart>(parent).is_some());
    }
}

#[test]
fn facility_lights_do_not_render_shadow_cubemaps() {
    let mut app = app();
    ready(&mut app);
    let world = app.world_mut();
    let lights: Vec<_> = world.query::<&PointLight>().iter(world).collect();
    assert!(!lights.is_empty());
    assert!(lights.iter().all(|light| !light.shadow_maps_enabled));
}

#[test]
fn rendered_pieces_keep_their_authored_world_placement() {
    let mut app = app();
    ready(&mut app);
    app.update();
    let plan = derive(&authored()).unwrap();
    let world = app.world_mut();
    let mut placed: Vec<_> = world
        .query_filtered::<(&Name, &GlobalTransform), With<WorldAssetRoot>>()
        .iter(world)
        .map(|(name, transform)| (name.as_str().to_owned(), transform.translation()))
        .collect();
    for piece in &plan.pieces {
        let found = placed.iter().position(|(name, translation)| {
            name == piece.module && translation.distance(piece.translation) < 1e-3
        });
        let index = found
            .unwrap_or_else(|| panic!("{} at {} is not rendered", piece.module, piece.translation));
        placed.swap_remove(index);
    }
    assert!(placed.is_empty());
}

#[test]
fn late_facility_spawn_renders_without_duplicates() {
    let mut app = app();
    ready(&mut app);
    let (pieces, lights) = plan_counts();
    let layout = authored();
    let plan = derive(&layout).unwrap();
    let mut commands = app.world_mut().commands();
    spawn_facility(&mut commands, layout, plan);
    app.world_mut().flush();
    app.update();
    app.update();
    assert_eq!(count::<With<WorldAssetRoot>>(&mut app), pieces * 2);
    assert_eq!(count::<With<PointLight>>(&mut app), lights * 2);

    let first = app
        .world_mut()
        .query_filtered::<Entity, With<Facility>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut().entity_mut(first).despawn();
    app.update();
    assert_eq!(count::<With<WorldAssetRoot>>(&mut app), pieces);
    assert_eq!(count::<With<PointLight>>(&mut app), lights);
}

#[test]
fn spawning_parts_on_entering_ready_renders_them_once() {
    let mut app = app();
    app.add_systems(OnEnter(GameAssetsState::Ready), |mut commands: Commands| {
        let layout = authored();
        let plan = derive(&layout).unwrap();
        spawn_facility(&mut commands, layout, plan);
    });
    ready(&mut app);
    let (pieces, lights) = plan_counts();
    for _ in 0..3 {
        app.update();
        assert_eq!(count::<With<WorldAssetRoot>>(&mut app), pieces * 2);
        assert_eq!(count::<With<PointLight>>(&mut app), lights * 2);
    }
}

#[test]
fn reinserting_a_rendered_part_does_not_render_it_again() {
    let mut app = app();
    ready(&mut app);
    let (pieces, _) = plan_counts();
    let world = app.world_mut();
    let parts: Vec<_> = world
        .query::<(Entity, &FacilityPart)>()
        .iter(world)
        .map(|(entity, part)| (entity, part.0))
        .collect();
    for (entity, owner) in parts {
        world
            .entity_mut(entity)
            .remove::<FacilityPart>()
            .insert(FacilityPart(owner));
    }
    app.update();
    assert_eq!(count::<With<WorldAssetRoot>>(&mut app), pieces);
}

#[test]
#[should_panic(expected = "needs GameAssetsState")]
fn render_plugin_requires_the_asset_state() {
    App::new()
        .add_plugins((MinimalPlugins, FacilityPlugin, FacilityRenderPlugin))
        .finish();
}

#[test]
fn door_panel_follows_logical_door_state() {
    let mut app = app();
    ready(&mut app);
    let world = app.world_mut();
    let (exit, facing) = world
        .query::<(Entity, &Objective, &gameplay::facility::Boundary)>()
        .iter(world)
        .find(|(_, objective, _)| **objective == Objective::Exit)
        .map(|(entity, _, boundary)| (entity, boundary.facing))
        .unwrap();
    let key = edge_key(IVec2::new(0, -12), Side::North);
    let grid = world.query::<&FacilityGrid>().single(world).unwrap();
    assert_eq!(grid.boundaries[&key], exit);
    let panel = |app: &mut App| {
        let world = app.world_mut();
        let children = world.get::<Children>(exit).unwrap().to_vec();
        let panels: Vec<_> = children
            .into_iter()
            .filter(|child| world.get::<DoorPanel>(*child).is_some())
            .collect();
        assert_eq!(panels.len(), 1);
        let global = world.get::<GlobalTransform>(panels[0]).unwrap();
        let (_, rotation, translation) = global.to_scale_rotation_translation();
        (translation, rotation.to_euler(EulerRot::YXZ).0)
    };
    let close = |a: f32, b: f32| (a - b).rem_euclid(TAU).min((b - a).rem_euclid(TAU)) < 1e-3;

    app.update();
    let (translation, yaw) = panel(&mut app);
    let (expected, expected_yaw) = door_panel(key, facing, Door::Closed);
    assert!(translation.distance(expected) < 1e-3);
    assert!(close(yaw, expected_yaw));

    *app.world_mut().get_mut::<DoorState>(exit).unwrap() = DoorState::Open;
    app.update();
    app.update();
    let (translation, yaw) = panel(&mut app);
    let swing = app
        .world()
        .get::<gameplay::facility::Doorway>(exit)
        .unwrap()
        .swing;
    let (expected, expected_yaw) = door_panel(key, facing, Door::Open(swing));
    assert!(translation.distance(expected) < 1e-3);
    assert!(close(yaw, expected_yaw));
    assert!(!close(yaw, door_panel(key, facing, Door::Closed).1));
}
