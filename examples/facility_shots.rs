use std::{path::PathBuf, sync::Arc};

use bevy::{camera::ScalingMode, world_serialization::WorldAssetRoot};
use game::{
    gameplay::levels::{build_first_floor, RenderCeilings, Room},
    prelude::*,
    probe::world_instances_ready,
};

const OUTPUT_DIR: &str = "target/facility-shots";
const SHOTS: &[Shot] = &[
    Shot::top("top"),
    Shot::overhead("overhead"),
    Shot::eye("exit", [2.5, 1.6, -27.5], [5.0, 1.4, -30.0]),
    Shot::eye("prep", [10.5, 1.6, -7.3], [7.0, 1.1, -7.3]),
    Shot::eye("office", [6.4, 1.6, -19.5], [3.7, 1.1, -19.0]),
    Shot::eye("storage", [8.0, 1.6, -10.0], [11.0, 1.4, -14.0]),
    Shot::eye("lab", [0.0, 1.6, -2.5], [0.0, 1.4, 1.5]),
    Shot::eye("boiler", [-10.0, 1.6, -3.5], [-10.0, 1.2, 0.0]),
    Shot::eye("utility", [-8.0, 1.6, -10.8], [-12.5, 0.9, -8.2]),
    Shot::eye("utility_bins", [-8.0, 1.6, -10.0], [-9.5, 1.2, -7.0]),
    Shot::eye("maintenance", [-7.0, 1.6, -19.0], [-12.4, 1.1, -18.0]),
    Shot::eye(
        "maintenance_claw",
        [-11.1, 2.15, -23.1],
        [-13.65, 1.6, -22.0],
    ),
    Shot::eye("workshop", [-10.0, 1.6, 8.3], [-7.5, 1.1, 9.7]),
    Shot::eye("archive", [-9.0, 1.6, 13.4], [-12.4, 1.1, 15.6]),
    Shot::eye("rear_hall", [-2.0, 1.6, 17.6], [-12.6, 0.8, 17.0]),
    Shot::eye("maintenance_drum", [-4.0, 1.6, -20.5], [-4.4, 0.9, -23.1]),
    Shot::eye("boiler_drum", [-10.0, 1.6, -1.5], [-12.5, 0.9, -2.5]),
    Shot::eye("fabrication_drum", [9.5, 1.6, 9.0], [7.0, 0.9, 10.5]),
    Shot::eye("rear_hall_drum", [-10.0, 1.6, 17.3], [-12.8, 0.9, 18.2]),
    Shot::eye("lab_crate", [0.5, 1.6, 0.5], [2.8, 0.7, 2.6]),
];

#[derive(Clone, Copy)]
enum View {
    Top,
    Overhead,
    Eye { position: Vec3, target: Vec3 },
}

#[derive(Clone, Copy)]
struct Shot {
    name: &'static str,
    view: View,
}

impl Shot {
    const fn top(name: &'static str) -> Self {
        Self {
            name,
            view: View::Top,
        }
    }

    const fn overhead(name: &'static str) -> Self {
        Self {
            name,
            view: View::Overhead,
        }
    }

    const fn eye(name: &'static str, position: [f32; 3], target: [f32; 3]) -> Self {
        Self {
            name,
            view: View::Eye {
                position: Vec3::from_array(position),
                target: Vec3::from_array(target),
            },
        }
    }
}

#[derive(Component)]
struct ReviewLight;

fn main() {
    let output = std::env::var_os("FACILITY_SHOTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(OUTPUT_DIR));
    let mut script = AutopilotPlugin::new()
        .step("load the authored floor and all prop scenes")
        .until(Arc::new(floor_ready), 60.0)
        .add()
        .step("settle the renderer")
        .until(frames(90), 20.0)
        .add();
    for shot in SHOTS {
        let path = output.join(format!("{}.png", shot.name));
        let view = shot.view;
        script = script
            .step(format!("frame {}", shot.name))
            .act(move |world| frame_view(world, view))
            .until(frames(20), 15.0)
            .add()
            .step(format!("capture {}", shot.name))
            .act({
                let path = path.clone();
                move |world| screenshot_start(world, path.clone())
            })
            .until(screenshot_written_at(path), 20.0)
            .add();
    }
    let mut app = AppBuilder::new().with_main_plugin(FloorShotsPlugin).build();
    app.insert_resource(RenderCeilings(false))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 14.0,
            ..default()
        })
        .add_plugins((CapturePlugin::default(), script))
        .run();
}

struct FloorShotsPlugin;

impl Plugin for FloorShotsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(GameAssetsState::Ready), build_first_floor);
    }
}

fn floor_ready(world: &World) -> bool {
    if world.resource::<State<GameAssetsState>>().get() != &GameAssetsState::Ready {
        return false;
    }
    let Some(mut rooms) = world.try_query::<&Room>() else {
        return false;
    };
    let Some(mut roots) = world.try_query::<&WorldAssetRoot>() else {
        return false;
    };
    rooms.iter(world).count() > 0 && roots.iter(world).count() > 0 && world_instances_ready(world)
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Transform::default()));
    commands.spawn((
        ReviewLight,
        PointLight {
            intensity: 900_000.0,
            range: 80.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 18.0, -6.0),
    ));
}

fn frame_view(world: &mut World, view: View) {
    let bounds = world
        .query::<&Room>()
        .iter(world)
        .map(|room| room.0)
        .reduce(|a, b| a.union(b))
        .expect("the authored first floor has rooms");
    let center = bounds.center();
    let center = Vec3::new(center.x, 0.0, center.y);
    let (transform, projection, review_light) = match view {
        View::Top => (
            Transform::from_translation(center + Vec3::Y * 42.0).looking_at(center, Vec3::X),
            Projection::from(OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin {
                    min_width: bounds.height() + 4.0,
                    min_height: bounds.width() + 4.0,
                },
                ..OrthographicProjection::default_3d()
            }),
            true,
        ),
        View::Overhead => (
            Transform::from_translation(center + Vec3::new(21.0, 38.0, 27.0))
                .looking_at(center, Vec3::Y),
            Projection::from(PerspectiveProjection {
                fov: 60.0_f32.to_radians(),
                ..default()
            }),
            true,
        ),
        View::Eye { position, target } => (
            Transform::from_translation(position).looking_at(target, Vec3::Y),
            Projection::from(PerspectiveProjection {
                fov: 75.0_f32.to_radians(),
                ..default()
            }),
            false,
        ),
    };
    for (mut camera, mut current) in world
        .query::<(&mut Transform, &mut Projection)>()
        .iter_mut(world)
    {
        *camera = transform;
        *current = projection.clone();
    }
    for (mut light, mut position) in world
        .query_filtered::<(&mut PointLight, &mut Transform), With<ReviewLight>>()
        .iter_mut(world)
    {
        light.intensity = if review_light { 900_000.0 } else { 180_000.0 };
        position.translation = if review_light {
            center + Vec3::Y * 18.0
        } else {
            transform.translation + Vec3::Y * 1.2
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn shots_are_distinct_and_eye_cameras_start_inside_rooms() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, build_first_floor);
        app.update();
        let world = app.world_mut();
        let rooms: Vec<(String, Rect)> = world
            .query::<(&Name, &Room)>()
            .iter(world)
            .map(|(name, room)| (name.as_str().to_owned(), room.0))
            .collect();
        let mut names = HashSet::new();
        for shot in SHOTS {
            assert!(names.insert(shot.name));
            if let View::Eye { position, target } = shot.view {
                assert!(
                    rooms.iter().any(|(name, room)| (name == shot.name
                        || matches!(
                            shot.name.strip_prefix(name.as_str()),
                            Some("_bins" | "_claw" | "_drum" | "_crate")
                        ))
                        && room.contains(position.xz())),
                    "{}",
                    shot.name
                );
                assert!(position.distance(target) > 1.0);
            }
        }
        assert!(matches!(SHOTS[0].view, View::Top));
        assert!(matches!(SHOTS[1].view, View::Overhead));
        assert!(SHOTS.len() >= 6);
    }
}
