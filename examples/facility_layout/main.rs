mod layout;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use bevy::{
    asset::{LoadState, RecursiveDependencyLoadState},
    camera::ScalingMode,
    gltf::GltfMaterialName,
    pbr::{DistanceFog, FogFalloff},
    world_serialization::{WorldAsset, WorldAssetRoot, WorldInstanceReady},
};
use game::prelude::*;
use layout::{authored, derive, Glow, Part, MODULES};

const ASSET_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/art/visuals/generated");
const SHOT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/art/visuals/screenshots");
const SHOT_PREFIX: &str = "facility_floor1_v2";
const GLOWING_MATERIALS: [&str; 4] = ["lamp_cool", "lamp_red", "lamp_fire", "specimen_fluid"];

struct View {
    name: &'static str,
    eye: Vec3,
    target: Vec3,
    plan: bool,
    fov_deg: f32,
}

const VIEWS: [View; 16] = [
    View {
        name: "plan",
        eye: Vec3::new(0.0, 45.0, -14.0),
        target: Vec3::new(0.0, 0.0, -14.0),
        plan: true,
        fov_deg: 45.0,
    },
    View {
        name: "intake",
        eye: Vec3::new(0.0, 1.6, -5.0),
        target: Vec3::new(0.0, 1.4, -20.0),
        plan: false,
        fov_deg: 45.0,
    },
    View {
        name: "reception",
        eye: Vec3::new(1.6, 1.6, -14.0),
        target: Vec3::new(-1.5, 1.1, -11.0),
        plan: false,
        fov_deg: 70.0,
    },
    View {
        name: "maintenance",
        eye: Vec3::new(-5.2, 1.6, -19.0),
        target: Vec3::new(-9.0, 1.0, -20.5),
        plan: false,
        fov_deg: 55.0,
    },
    View {
        name: "storage",
        eye: Vec3::new(5.2, 1.6, -12.5),
        target: Vec3::new(10.0, 1.0, -14.5),
        plan: false,
        fov_deg: 55.0,
    },
    View {
        name: "service",
        eye: Vec3::new(0.0, 1.6, -17.4),
        target: Vec3::new(0.0, 1.5, -26.0),
        plan: false,
        fov_deg: 55.0,
    },
    View {
        name: "exit",
        eye: Vec3::new(2.5, 1.6, -24.0),
        target: Vec3::new(5.0, 1.8, -31.0),
        plan: false,
        fov_deg: 60.0,
    },
    View {
        name: "security",
        eye: Vec3::new(12.0, 1.6, -26.0),
        target: Vec3::new(10.2, 1.1, -29.0),
        plan: false,
        fov_deg: 70.0,
    },
    View {
        name: "utility",
        eye: Vec3::new(-8.0, 1.6, -10.0),
        target: Vec3::new(-10.0, 1.1, -12.5),
        plan: false,
        fov_deg: 55.0,
    },
    View {
        name: "hiding",
        eye: Vec3::new(8.0, 1.6, -3.0),
        target: Vec3::new(10.5, 1.3, 1.5),
        plan: false,
        fov_deg: 75.0,
    },
    View {
        name: "signage",
        eye: Vec3::new(0.0, 1.6, -12.5),
        target: Vec3::new(-2.5, 2.0, -15.9),
        plan: false,
        fov_deg: 55.0,
    },
    View {
        name: "boiler",
        eye: Vec3::new(-8.0, 1.6, 2.0),
        target: Vec3::new(-10.0, 1.1, 0.0),
        plan: false,
        fov_deg: 55.0,
    },
    View {
        name: "lab",
        eye: Vec3::new(0.0, 1.65, -2.5),
        target: Vec3::new(0.0, 1.0, 0.0),
        plan: false,
        fov_deg: 60.0,
    },
    View {
        name: "workshop",
        eye: Vec3::new(-5.2, 1.75, -20.0),
        target: Vec3::new(-7.5, 0.5, -20.0),
        plan: false,
        fov_deg: 55.0,
    },
    View {
        name: "west_hall",
        eye: Vec3::new(-5.0, 1.6, 0.0),
        target: Vec3::new(-5.0, 1.4, -15.0),
        plan: false,
        fov_deg: 55.0,
    },
    View {
        name: "east_hall",
        eye: Vec3::new(5.0, 1.6, 0.0),
        target: Vec3::new(5.0, 1.4, -15.0),
        plan: false,
        fov_deg: 55.0,
    },
];

#[derive(Resource)]
struct ModuleScenes(BTreeMap<&'static str, Handle<WorldAsset>>);

#[derive(Component)]
struct LayoutCamera;

#[derive(Component)]
struct Overhead;

#[derive(Component)]
struct Glowing(Glow);

#[derive(Component)]
struct GlowSurface {
    glow: Glow,
    base: LinearRgba,
}

#[derive(Component)]
struct GlowLight {
    glow: Glow,
    base: f32,
}

fn main() {
    let modules = PathBuf::from(ASSET_ROOT).join("modules");
    for module in MODULES {
        assert!(
            modules.join(format!("{module}.glb")).is_file(),
            "missing {module}.glb; run scripts/generate-facility.sh before facility_layout"
        );
    }

    let mut script = AutopilotPlugin::new()
        .with_deadline_secs(600.0)
        .step("wait for module scenes")
        .until(Arc::new(modules_loaded), 60.0)
        .add()
        .step("allow render to settle")
        .until(frames(180), 60.0)
        .add();
    for (index, view) in VIEWS.iter().enumerate() {
        let path = format!("{SHOT_DIR}/{SHOT_PREFIX}_{}.png", view.name);
        let shot = path.clone();
        script = script
            .step(format!("frame {} view", view.name))
            .act(move |world| apply_view(world, &VIEWS[index]))
            .until(frames(45), 30.0)
            .add()
            .step(format!("capture {} view", view.name))
            .act(move |world| screenshot_start(world, shot.clone()))
            .until(screenshot_written_at(path), 30.0)
            .add();
    }

    AppBuilder::new()
        .with_asset_path(ASSET_ROOT)
        .build()
        .add_plugins((CapturePlugin::new(30), script))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.55, 0.6, 0.7),
            brightness: 20.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, animate_glow)
        .add_observer(attach_glow_surfaces)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let plan = derive(&authored()).unwrap_or_else(|error| panic!("invalid layout: {error}"));
    let scenes: BTreeMap<_, _> = MODULES
        .iter()
        .map(|&module| (module, assets.load(format!("modules/{module}.glb#Scene0"))))
        .collect();

    for piece in &plan.pieces {
        let mut entity = commands.spawn((
            Name::new(piece.module),
            WorldAssetRoot(scenes[piece.module].clone()),
            Transform::from_translation(piece.translation)
                .with_rotation(piece.rotation())
                .with_scale(piece.scale),
        ));
        if matches!(piece.part, Part::Ceiling | Part::Light) {
            entity.insert(Overhead);
        }
        if let Some(glow @ (Glow::Flicker(_) | Glow::Pulse)) = piece.glow {
            entity.insert(Glowing(glow));
        }
    }

    for light in &plan.lights {
        let mut entity = commands.spawn((
            PointLight {
                color: Color::linear_rgb(light.color[0], light.color[1], light.color[2]),
                intensity: light.intensity,
                range: light.range,
                radius: 0.08,
                shadow_maps_enabled: light.shadows,
                ..default()
            },
            Transform::from_translation(light.position),
        ));
        if matches!(light.glow, Glow::Flicker(_) | Glow::Pulse) {
            entity.insert(GlowLight {
                glow: light.glow,
                base: light.intensity,
            });
        }
    }

    commands.insert_resource(ModuleScenes(scenes));
    commands.spawn((Camera3d::default(), LayoutCamera, Transform::default()));
}

fn modules_loaded(world: &World) -> bool {
    let Some(scenes) = world.get_resource::<ModuleScenes>() else {
        return false;
    };
    let assets = world.resource::<AssetServer>();
    scenes.0.iter().all(|(module, handle)| {
        if let Some(LoadState::Failed(error)) = assets.get_load_state(handle) {
            panic!("{module} failed to load: {error}");
        }
        if let Some(RecursiveDependencyLoadState::Failed(error)) =
            assets.get_recursive_dependency_load_state(handle)
        {
            panic!("{module} dependency failed to load: {error}");
        }
        assets.is_loaded_with_dependencies(handle)
    })
}

fn apply_view(world: &mut World, view: &View) {
    let camera = world
        .query_filtered::<Entity, With<LayoutCamera>>()
        .single(world)
        .expect("one layout camera");
    let up = if view.plan { Vec3::NEG_Z } else { Vec3::Y };
    let mut entity = world.entity_mut(camera);
    entity.insert(Transform::from_translation(view.eye).looking_at(view.target, up));
    if view.plan {
        entity
            .insert(Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 42.0,
                },
                ..OrthographicProjection::default_3d()
            }))
            .remove::<DistanceFog>();
    } else {
        entity.insert((
            Projection::Perspective(PerspectiveProjection {
                fov: view.fov_deg.to_radians(),
                ..default()
            }),
            DistanceFog {
                color: Color::srgb(0.015, 0.018, 0.022),
                falloff: FogFalloff::ExponentialSquared { density: 0.045 },
                ..default()
            },
        ));
    }
    world.resource_mut::<GlobalAmbientLight>().brightness = if view.plan { 400.0 } else { 20.0 };
    let visibility = if view.plan {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    let mut overhead = world.query_filtered::<&mut Visibility, With<Overhead>>();
    for mut v in overhead.iter_mut(world) {
        *v = visibility;
    }
}

fn attach_glow_surfaces(
    ready: On<WorldInstanceReady>,
    roots: Query<&Glowing>,
    children: Query<&Children>,
    surfaces: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok(Glowing(glow)) = roots.get(ready.entity) else {
        return;
    };
    for entity in children.iter_descendants(ready.entity) {
        let Ok((name, material)) = surfaces.get(entity) else {
            continue;
        };
        if !GLOWING_MATERIALS.contains(&name.0.as_str()) {
            continue;
        }
        let Some(source) = materials.get(&material.0).cloned() else {
            continue;
        };
        let base = source.emissive;
        commands.entity(entity).insert((
            MeshMaterial3d(materials.add(source)),
            GlowSurface { glow: *glow, base },
        ));
    }
}

fn glow_factor(glow: Glow, t: f32) -> f32 {
    match glow {
        Glow::Steady => 1.0,
        Glow::Flicker(phase) => {
            let s = (t * 1.3 + phase).sin() + 0.6 * (t * 4.1 + phase * 2.0).sin();
            if s > 1.1 {
                0.06
            } else if s > 0.9 {
                0.5
            } else {
                1.0
            }
        }
        Glow::Pulse => 0.55 + 0.45 * (t * 2.0).sin(),
    }
}

fn animate_glow(
    time: Res<Time>,
    mut lights: Query<(&GlowLight, &mut PointLight)>,
    surfaces: Query<(&GlowSurface, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();
    for (glow, mut light) in &mut lights {
        light.intensity = glow.base * glow_factor(glow.glow, t);
    }
    for (surface, material) in &surfaces {
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.emissive = surface.base * glow_factor(surface.glow, t);
        }
    }
}
