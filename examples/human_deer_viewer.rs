use std::path::PathBuf;
use std::sync::Arc;

use bevy::animation::{graph::AnimationNodeIndex, AnimatedBy};
use bevy::asset::{AssetPlugin, LoadState, RecursiveDependencyLoadState};
use game::prelude::*;

const GLB_PATH: &str = "human_deer_animated.glb";
const SCENE_PATH: &str = "human_deer_animated.glb#Scene0";
const CLIP_NAMES: [&str; 4] = ["IDLE", "WALK", "CHASE", "ATTACK"];
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
    graph: Handle<AnimationGraph>,
    nodes: Vec<AnimationNodeIndex>,
    selected: usize,
}

#[derive(Component)]
struct AnimationLabel;

#[derive(Component)]
struct OrbitCamera;

fn main() {
    let asset_root = concat!(env!("CARGO_MANIFEST_DIR"), "/art/visuals/generated/monster");
    assert!(
        PathBuf::from(asset_root).join(GLB_PATH).is_file(),
        "generate art/visuals/generated/monster/human_deer_animated.glb before running human_deer_viewer"
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
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let scene = assets.load(SCENE_PATH);
    let clips = (0..CLIP_NAMES.len())
        .map(|index| assets.load(GltfAssetLabel::Animation(index).from_asset(GLB_PATH)));
    let (graph, nodes) = AnimationGraph::from_clips(clips);
    let graph = graphs.add(graph);
    commands.spawn((
        WorldAssetRoot(scene.clone()),
        Transform::from_xyz(0.0, MODEL_Y_OFFSET, 0.0).with_scale(Vec3::splat(MODEL_SCALE)),
    ));
    commands.insert_resource(DeerScene {
        scene,
        graph,
        nodes,
        selected: 0,
    });
    commands.spawn((
        Text::new("IDLE"),
        TextFont {
            font_size: bevy::text::FontSize::Px(28.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            top: px(24),
            ..default()
        },
        AnimationLabel,
    ));

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
    keys: Res<ButtonInput<KeyCode>>,
    mut deer: ResMut<DeerScene>,
    mut players: Query<(Entity, &mut AnimationPlayer, Has<AnimationGraphHandle>)>,
    mut label: Query<&mut Text, With<AnimationLabel>>,
) {
    let previous = deer.selected;
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::Comma) {
        deer.selected = (deer.selected + CLIP_NAMES.len() - 1) % CLIP_NAMES.len();
    } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::Period) {
        deer.selected = (deer.selected + 1) % CLIP_NAMES.len();
    }
    let changed = previous != deer.selected;
    if changed {
        for mut text in &mut label {
            **text = CLIP_NAMES[deer.selected].into();
        }
    }
    for (entity, mut player, has_graph) in &mut players {
        if !has_graph {
            commands
                .entity(entity)
                .insert(AnimationGraphHandle(deer.graph.clone()));
        }
        if changed || player.is_added() {
            player.stop_all();
            let active = player.play(deer.nodes[deer.selected]);
            if deer.selected != 3 {
                active.repeat();
            }
        }
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

#[cfg(test)]
#[path = "../tests/unit/human_deer_viewer.rs"]
mod tests;

fn orbit_camera(time: Res<Time>, mut cameras: Query<&mut Transform, With<OrbitCamera>>) {
    let transform = orbit_transform(time.elapsed_secs() * ORBIT_SPEED);
    for mut camera in &mut cameras {
        *camera = transform;
    }
}
