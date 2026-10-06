use std::path::PathBuf;
use std::sync::Arc;

use bevy::asset::{LoadState, RecursiveDependencyLoadState};
use game::prelude::*;

const SCENE_PATH: &str = "../art/visuals/generated/facility.glb#Scene0";
const SCREENSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/visuals/screenshots/facility_gallery.png"
);
const VIDEO_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/visuals/screenshots/facility_gallery.webm"
);

#[derive(Resource)]
struct FacilityScene(Handle<WorldAsset>);

#[derive(Component)]
struct GalleryCamera;

#[derive(Resource)]
struct VideoMotion;

fn main() {
    let asset_root = concat!(env!("CARGO_MANIFEST_DIR"), "/art/visuals/generated");
    assert!(
        PathBuf::from(asset_root).join("facility.glb").is_file(),
        "generate art/visuals/generated/facility.glb before running facility_gallery"
    );

    let script = AutopilotPlugin::new()
        .step("wait for facility scene")
        .until(Arc::new(scene_loaded), 30.0)
        .add()
        .step("allow render to settle")
        .until(frames(180), 20.0)
        .add()
        .step("capture facility")
        .act(|world| screenshot_start(world, SCREENSHOT_PATH))
        .until(screenshot_written_at(SCREENSHOT_PATH), 15.0)
        .add()
        .step("start corridor video")
        .act(|world| {
            world.insert_resource(VideoMotion);
            loop_start(world, VIDEO_PATH);
        })
        .add()
        .step("record corridor walk")
        .until(frames(60), 30.0)
        .add()
        .step("encode corridor video")
        .act(loop_end)
        .until(loop_written_at(VIDEO_PATH), 60.0)
        .add();
    AppBuilder::new()
        .build()
        .add_plugins((CapturePlugin::new(30), script))
        .add_systems(Startup, setup)
        .add_systems(Update, move_camera)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let scene = assets.load(SCENE_PATH);
    commands.insert_resource(FacilityScene(scene.clone()));
    commands.spawn(WorldAssetRoot(scene));
    commands.spawn((
        Camera3d::default(),
        GalleryCamera,
        Transform::from_xyz(0.0, 1.6, -0.6).looking_at(Vec3::new(0.0, 1.3, -16.5), Vec3::Y),
    ));
    for z in [-1.25, -3.75, -8.75] {
        commands.spawn((
            PointLight {
                color: Color::srgb(0.68, 0.84, 1.0),
                intensity: 75_000.0,
                range: 5.0,
                ..default()
            },
            Transform::from_xyz(0.0, 2.7, z),
        ));
    }
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.62, 0.25),
            intensity: 100_000.0,
            range: 8.0,
            ..default()
        },
        Transform::from_xyz(0.0, 2.7, -16.25),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.12, 0.08),
            intensity: 30_000.0,
            range: 4.0,
            ..default()
        },
        Transform::from_xyz(2.2, 2.2, -19.65),
    ));
}

fn scene_loaded(world: &World) -> bool {
    let Some(scene) = world.get_resource::<FacilityScene>() else {
        return false;
    };
    let assets = world.resource::<AssetServer>();
    if let Some(LoadState::Failed(error)) = assets.get_load_state(&scene.0) {
        panic!("facility scene failed to load: {error}");
    }
    if let Some(RecursiveDependencyLoadState::Failed(error)) =
        assets.get_recursive_dependency_load_state(&scene.0)
    {
        panic!("facility scene dependency failed to load: {error}");
    }
    assets.is_loaded_with_dependencies(&scene.0)
}

fn move_camera(
    time: Res<Time>,
    motion: Option<Res<VideoMotion>>,
    mut camera: Query<&mut Transform, With<GalleryCamera>>,
) {
    if motion.is_some() {
        for mut transform in &mut camera {
            transform.translation.z -= time.delta_secs() * 0.8;
        }
    }
}
