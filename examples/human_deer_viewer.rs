use std::path::PathBuf;
use std::sync::Arc;

use bevy::animation::AnimatedBy;
use bevy::asset::{AssetPlugin, LoadState, RecursiveDependencyLoadState};
use game::prelude::*;

const SCENE_PATH: &str = "scene.gltf#Scene0";
const GLTF_PATH: &str = "scene.gltf";
const SCREENSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/visuals/screenshots/human_deer_viewer.png"
);

const MODEL_SCALE: f32 = 0.42;
const MODEL_Y_OFFSET: f32 = 1.93;
const LOOK_AT: Vec3 = Vec3::new(0.0, 1.0, 1.3);
const ORBIT_RADIUS: f32 = 3.4;
const ORBIT_HEIGHT: f32 = 1.6;
const ORBIT_SPEED: f32 = 0.15;
const INITIAL_ANGLE: f32 = 0.6;

#[derive(Resource)]
struct DeerScene {
    scene: Handle<WorldAsset>,
    clip: Handle<AnimationClip>,
}

#[derive(Component)]
struct OrbitCamera;

fn main() {
    let asset_root = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/art/visuals/sources/sketchfab/the_human_deer"
    );
    assert!(
        PathBuf::from(asset_root).join("scene.gltf").is_file(),
        "extract the_human_deer.zip into art/visuals/sources/sketchfab/the_human_deer before running human_deer_viewer"
    );

    let capture = std::env::args().any(|arg| arg == "--capture");
    assert!(
        !capture || !PathBuf::from(SCREENSHOT_PATH).is_file(),
        "human_deer_viewer.png already exists; remove it before capturing a new one"
    );
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: asset_root.into(),
        ..default()
    }))
    .insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 10.0,
        ..default()
    })
    .add_plugins(HumanDeerViewerPlugin { capture });
    if capture {
        let script = AutopilotPlugin::new()
            .step("wait for human deer scene")
            .until(Arc::new(scene_loaded), 30.0)
            .add()
            .step("allow render to settle")
            .until(frames(90), 20.0)
            .add()
            .step("capture human deer")
            .act(|world| screenshot_start(world, SCREENSHOT_PATH))
            .until(screenshot_written_at(SCREENSHOT_PATH), 15.0)
            .add();
        app.add_plugins((CapturePlugin::new(30), script));
    }
    app.run();
}

struct HumanDeerViewerPlugin {
    capture: bool,
}

impl Plugin for HumanDeerViewerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (play_animation, measure_bounds));
        if !self.capture {
            app.add_systems(Update, orbit_camera);
        }
    }
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let scene = assets.load(SCENE_PATH);
    let clip = assets.load(GltfAssetLabel::Animation(0).from_asset(GLTF_PATH));
    commands.spawn((
        WorldAssetRoot(scene.clone()),
        Transform::from_xyz(0.0, MODEL_Y_OFFSET, 0.0).with_scale(Vec3::splat(MODEL_SCALE)),
    ));
    commands.insert_resource(DeerScene { scene, clip });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(8.0, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.015, 0.015, 0.02),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));

    commands.spawn((
        Camera3d::default(),
        OrbitCamera,
        orbit_transform(INITIAL_ANGLE),
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.85, 0.82, 1.0),
            illuminance: 4500.0,
            ..default()
        },
        Transform::from_xyz(2.0, 3.6, 2.6).looking_at(LOOK_AT, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.4, 0.55, 1.0),
            intensity: 12_000.0,
            range: 6.0,
            ..default()
        },
        Transform::from_xyz(-1.8, 1.6, 1.8),
    ));
}

fn orbit_transform(angle: f32) -> Transform {
    Transform::from_xyz(
        LOOK_AT.x + angle.cos() * ORBIT_RADIUS,
        ORBIT_HEIGHT,
        LOOK_AT.z + angle.sin() * ORBIT_RADIUS,
    )
    .looking_at(LOOK_AT, Vec3::Y)
}

fn scene_loaded(world: &World) -> bool {
    let Some(deer) = world.get_resource::<DeerScene>() else {
        return false;
    };
    let assets = world.resource::<AssetServer>();
    if let Some(LoadState::Failed(error)) = assets.get_load_state(&deer.scene) {
        panic!("human deer scene failed to load: {error}");
    }
    if let Some(RecursiveDependencyLoadState::Failed(error)) =
        assets.get_recursive_dependency_load_state(&deer.scene)
    {
        panic!("human deer scene dependency failed to load: {error}");
    }
    assets.is_loaded_with_dependencies(&deer.scene)
}

fn play_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    deer: Option<Res<DeerScene>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    let Some(deer) = deer else {
        return;
    };
    for (entity, mut player) in &mut players {
        let (graph, node) = AnimationGraph::from_clip(deer.clip.clone());
        let handle = graphs.add(graph);
        player.play(node).repeat();
        commands.entity(entity).insert(AnimationGraphHandle(handle));
    }
}

fn measure_bounds(
    mut frame: Local<u32>,
    mut done: Local<bool>,
    joints: Query<&GlobalTransform, With<AnimatedBy>>,
) {
    if *done {
        return;
    }
    if joints.is_empty() {
        return;
    }
    *frame += 1;
    if *frame < 30 {
        return;
    }
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for transform in &joints {
        let point = transform.translation();
        min = min.min(point);
        max = max.max(point);
    }
    info!(
        "human_deer_viewer: measured joint bounds min={min:?} max={max:?} height={:.3}m",
        max.y - min.y
    );
    *done = true;
}

fn orbit_camera(time: Res<Time>, mut cameras: Query<&mut Transform, With<OrbitCamera>>) {
    let transform = orbit_transform(time.elapsed_secs() * ORBIT_SPEED);
    for mut camera in &mut cameras {
        *camera = transform;
    }
}
