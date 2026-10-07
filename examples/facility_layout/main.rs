use std::sync::Arc;

use bevy::{
    asset::RecursiveDependencyLoadState,
    camera::ScalingMode,
    pbr::{DistanceFog, FogFalloff},
};
use game::{gameplay::facility::Overhead, prelude::*};

const SHOT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/art/visuals/screenshots");
const SHOT_PREFIX: &str = "facility_floor1_v2";

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

#[derive(Component)]
struct LayoutCamera;

fn main() {
    let shot_dir = std::env::var("FACILITY_SHOT_DIR").unwrap_or_else(|_| SHOT_DIR.into());
    let mut script = AutopilotPlugin::new()
        .with_deadline_secs(600.0)
        .step("wait for module scenes")
        .until(Arc::new(modules_loaded), 60.0)
        .add()
        .step("allow render to settle")
        .until(frames(180), 60.0)
        .add();
    for (index, view) in VIEWS.iter().enumerate() {
        let path = format!("{shot_dir}/{SHOT_PREFIX}_{}.png", view.name);
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
        .build()
        .add_plugins((
            FacilityPlugin::new(first_floor()),
            CapturePlugin::new(30),
            script,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3d::default(), LayoutCamera, Transform::default()));
}

fn modules_loaded(world: &World) -> bool {
    match world.resource::<State<GameAssetsState>>().get() {
        GameAssetsState::Loading => false,
        GameAssetsState::Failed => {
            panic!("facility modules failed to load; run scripts/promote-facility-modules.sh")
        }
        GameAssetsState::Ready => {
            let assets = world.resource::<AssetServer>();
            world
                .resource::<FacilityAssets>()
                .modules
                .iter()
                .all(|(module, handle)| {
                    if let Some(RecursiveDependencyLoadState::Failed(error)) =
                        assets.get_recursive_dependency_load_state(handle)
                    {
                        panic!("{module:?} dependency failed to load: {error}");
                    }
                    assets.is_loaded_with_dependencies(handle)
                })
        }
    }
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
