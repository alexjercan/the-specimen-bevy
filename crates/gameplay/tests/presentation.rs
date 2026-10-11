use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy, world_serialization::WorldAssetRoot};
use bevy_asset_loader::mapped::{AssetFileStem, MapKey};
use game_assets::FacilityAssets;
use gameplay::{
    controller::{PlayerController, PlayerControlsEnabled},
    levels::{
        FacilityPower, FuseInventory, FusePanel, FusePlugin, InstallFuses, LevelRenderPlugin,
        FUSE_COUNT, FUSE_MODULE,
    },
};

mod common;

fn facility_assets() -> FacilityAssets {
    let mut modules = bevy::platform::collections::HashMap::default();
    modules.insert(
        AssetFileStem::from_asset_path(
            &format!("facility/modules/{FUSE_MODULE}.glb#Scene0").into(),
        ),
        Handle::default(),
    );
    FacilityAssets { modules }
}

fn visuals(app: &mut App, panel: Entity) -> Vec<(Entity, Vec3, Visibility)> {
    let world = app.world_mut();
    let mut children = world.query::<(Entity, &ChildOf, &Transform, &Visibility)>();
    children
        .iter(world)
        .filter(|(_, parent, _, _)| parent.parent() == panel)
        .map(|(entity, _, transform, visibility)| (entity, transform.translation, *visibility))
        .collect()
}

#[test]
fn installing_fuses_spawns_one_scene_child_per_fuse_and_slides_them_in_one_by_one() {
    let mut app = common::level_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        10,
    )))
    .insert_resource(facility_assets())
    .insert_resource(FacilityPower::default())
    .insert_resource(PlayerControlsEnabled::default())
    .init_resource::<Assets<Mesh>>()
    .init_resource::<Assets<StandardMaterial>>()
    .add_plugins((FusePlugin, LevelRenderPlugin));
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, 1.6, 1.0),
            FuseInventory(FUSE_COUNT),
        ))
        .id();
    let panel = app
        .world_mut()
        .spawn((FusePanel::default(), Transform::IDENTITY))
        .id();
    app.update();
    assert!(visuals(&mut app, panel).is_empty());
    app.world_mut()
        .write_message(InstallFuses { player, panel });
    app.update();
    app.update();

    let started = visuals(&mut app, panel);
    assert_eq!(started.len(), FUSE_COUNT);
    for (visual, _, _) in &started {
        let world = app.world_mut();
        let scenes: Vec<Entity> = world
            .query_filtered::<(Entity, &ChildOf), With<WorldAssetRoot>>()
            .iter(world)
            .filter(|(_, parent)| parent.parent() == *visual)
            .map(|(entity, _)| entity)
            .collect();
        assert_eq!(scenes.len(), 1);
    }
    assert_eq!(
        started
            .iter()
            .filter(|(_, _, visibility)| *visibility == Visibility::Visible)
            .count(),
        1
    );
    let waiting: Vec<&(Entity, Vec3, Visibility)> = started
        .iter()
        .filter(|(_, _, visibility)| *visibility == Visibility::Hidden)
        .collect();
    assert_eq!(waiting.len(), FUSE_COUNT - 1);
    let origin = waiting[0].1;
    assert!(waiting.iter().all(|(_, position, _)| *position == origin));
    let sliding = started
        .iter()
        .find(|(_, _, visibility)| *visibility == Visibility::Visible)
        .unwrap();
    assert!(sliding.1.distance(origin) < 0.1);

    let mut frames = 0;
    while app.world().get::<FusePanel>(panel).unwrap().installed != FUSE_COUNT {
        app.update();
        frames += 1;
        assert!(frames < 1000, "installation must finish");
    }
    app.update();
    let finished = visuals(&mut app, panel);
    assert_eq!(finished.len(), FUSE_COUNT);
    assert!(finished
        .iter()
        .all(|(_, _, visibility)| *visibility == Visibility::Visible));
    for (index, (_, position, _)) in finished.iter().enumerate() {
        assert_ne!(*position, origin);
        for (_, other, _) in &finished[index + 1..] {
            assert_ne!(position, other);
        }
    }
}
