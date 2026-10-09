use bevy::camera::ScalingMode;
use game::{
    autopilot::state_is,
    gameplay::levels::{build_first_floor, LightIntensity, RenderCeilings, Room},
    prelude::*,
};

const FRAME_MARGIN: f32 = 2.0;
const CAMERA_HEIGHT: f32 = 40.0;
const LIGHT_HEIGHT: f32 = 18.0;

const SCREENSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/visuals/screenshots/facility_layout_draft.png"
);
const DARK_SCREENSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/visuals/screenshots/facility_lighting_dark.png"
);
const LIGHTING_VIEWS: [(&str, Vec3, Vec3); 4] = [
    (
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/art/visuals/screenshots/facility_lighting_dark_exit.png"
        ),
        Vec3::new(2.5, 1.6, -27.5),
        Vec3::new(5.0, 1.6, -30.0),
    ),
    (
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/art/visuals/screenshots/facility_lighting_dark_storage.png"
        ),
        Vec3::new(8.0, 1.6, -10.0),
        Vec3::new(11.0, 1.4, -14.0),
    ),
    (
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/art/visuals/screenshots/facility_lighting_dark_lab.png"
        ),
        Vec3::new(0.0, 1.6, -2.5),
        Vec3::new(0.0, 1.4, 1.5),
    ),
    (
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/art/visuals/screenshots/facility_lighting_dark_rear.png"
        ),
        Vec3::new(5.0, 1.6, 6.0),
        Vec3::new(5.0, 1.6, 15.0),
    ),
];

fn main() {
    let dark = std::env::args().any(|arg| arg == "--dark");
    let angles_only = std::env::args().any(|arg| arg == "--angles-only");
    let screenshot = if dark {
        DARK_SCREENSHOT_PATH
    } else {
        SCREENSHOT_PATH
    };
    let mut script = AutopilotPlugin::new()
        .step("wait for facility assets")
        .until(state_is(GameAssetsState::Ready), 30.0)
        .add()
        .step("allow render to settle")
        .until(frames(180), 20.0)
        .add();
    if !angles_only {
        script = script
            .step("capture first floor")
            .act(move |world| screenshot_start(world, screenshot))
            .until(screenshot_written_at(screenshot), 15.0)
            .add();
    }
    if dark {
        for (path, position, target) in LIGHTING_VIEWS {
            script = script
                .step(format!("frame lighting view {path}"))
                .act(move |world| set_eye_view(world, position, target))
                .until(frames(4), 10.0)
                .add()
                .step(format!("capture lighting view {path}"))
                .act(move |world| screenshot_start(world, path))
                .until(screenshot_written_at(path), 15.0)
                .add();
        }
    }

    let mut app = AppBuilder::new()
        .with_main_plugin(FacilityLayoutPlugin)
        .build();
    app.insert_resource(RenderCeilings(false))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: if dark { 6.0 } else { 20.0 },
            ..default()
        })
        .add_plugins((CapturePlugin::default(), script));
    if dark {
        app.add_systems(Update, dim_new_lights);
    }
    app.run();
}

struct FacilityLayoutPlugin;

impl Plugin for FacilityLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            OnEnter(GameAssetsState::Ready),
            (build_first_floor, frame_first_floor).chain(),
        );
    }
}

fn set_eye_view(world: &mut World, position: Vec3, target: Vec3) {
    let mut cameras = world.query_filtered::<(&mut Transform, &mut Projection), With<Camera3d>>();
    for (mut transform, mut projection) in cameras.iter_mut(world) {
        *transform = Transform::from_translation(position).looking_at(target, Vec3::Y);
        *projection = Projection::Perspective(PerspectiveProjection::default());
    }
    let mut lights = world.query_filtered::<&mut PointLight, With<OverviewLight>>();
    for mut light in lights.iter_mut(world) {
        light.intensity = 0.0;
    }
}

fn dim_new_lights(
    mut lights: Query<(&mut PointLight, &mut LightIntensity), Added<LightIntensity>>,
) {
    for (mut light, mut base) in &mut lights {
        base.0 *= 0.45;
        light.intensity = base.0;
    }
}

#[derive(Component)]
struct OverviewLight;

fn floor_bounds(rooms: impl IntoIterator<Item = Rect>) -> Option<Rect> {
    rooms.into_iter().reduce(|bounds, room| bounds.union(room))
}

fn overhead_view(bounds: Rect) -> (Transform, Projection) {
    let center = bounds.center();
    let target = Vec3::new(center.x, 0.0, center.y);
    let transform =
        Transform::from_translation(target + Vec3::Y * CAMERA_HEIGHT).looking_at(target, Vec3::X);
    let projection = Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin {
            min_width: bounds.height() + 2.0 * FRAME_MARGIN,
            min_height: bounds.width() + 2.0 * FRAME_MARGIN,
        },
        ..OrthographicProjection::default_3d()
    });
    (transform, projection)
}

fn frame_first_floor(
    rooms: Query<&Room>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
    mut lights: Query<&mut Transform, (With<OverviewLight>, Without<Camera3d>)>,
) {
    let Some(bounds) = floor_bounds(rooms.iter().map(|room| room.0)) else {
        return;
    };
    let (view, projection) = overhead_view(bounds);
    for (mut transform, mut current) in &mut cameras {
        *transform = view;
        *current = projection.clone();
    }
    let center = bounds.center();
    for mut transform in &mut lights {
        transform.translation = Vec3::new(center.x, LIGHT_HEIGHT, center.y);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, CAMERA_HEIGHT, 0.0).looking_at(Vec3::ZERO, Vec3::X),
    ));
    commands.spawn((
        OverviewLight,
        PointLight {
            intensity: if std::env::args().any(|arg| arg == "--dark") {
                180_000.0
            } else {
                900_000.0
            },
            range: 70.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, LIGHT_HEIGHT, 0.0),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lighting_views_are_distinct_and_inside_authored_rooms() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, build_first_floor);
        app.update();
        let world = app.world_mut();
        let rooms: Vec<Rect> = world
            .query::<&Room>()
            .iter(world)
            .map(|room| room.0)
            .collect();
        for (index, (path, position, target)) in LIGHTING_VIEWS.iter().enumerate() {
            assert!(rooms.iter().any(|room| room.contains(position.xz())));
            assert!((target.xz() - position.xz()).length() > 1.0);
            assert!(LIGHTING_VIEWS[index + 1..]
                .iter()
                .all(|(other, _, _)| path != other));
        }
    }

    #[test]
    fn eye_views_use_perspective_without_overhead_fill_light() {
        let mut world = World::new();
        let camera = world
            .spawn((
                Camera3d::default(),
                Transform::default(),
                Projection::default(),
            ))
            .id();
        let light = world.spawn((OverviewLight, PointLight::default())).id();
        let (_, position, target) = LIGHTING_VIEWS[0];
        set_eye_view(&mut world, position, target);
        assert_eq!(
            world.get::<Transform>(camera).unwrap().translation,
            position
        );
        assert!(matches!(
            world.get::<Projection>(camera),
            Some(Projection::Perspective(_))
        ));
        assert_eq!(world.get::<PointLight>(light).unwrap().intensity, 0.0);
    }

    #[test]
    fn overhead_view_frames_the_whole_first_floor_without_cropping() {
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
        let bounds = floor_bounds(rooms.iter().map(|(_, room)| *room)).unwrap();
        let room = |wanted: &str| {
            rooms
                .iter()
                .find(|(name, _)| name == wanted)
                .unwrap_or_else(|| panic!("missing {wanted}"))
                .1
        };
        assert_eq!(bounds.min.y, room("exit").min.y);
        assert_eq!(bounds.max.y, room("rear_hall").max.y);
        assert_eq!(bounds.min.x, room("rear_hall").min.x);
        assert_eq!(bounds.max.x, room("rear_hall").max.x);

        let (transform, projection) = overhead_view(bounds);
        let view_from_world = transform.to_matrix().inverse();
        for (width, height) in [
            (1280.0, 720.0),
            (1920.0, 1080.0),
            (1024.0, 1024.0),
            (720.0, 1280.0),
        ] {
            let mut projection = projection.clone();
            projection.update(width, height);
            let clip_from_world = projection.get_clip_from_view() * view_from_world;
            for (name, room) in &rooms {
                for corner in [
                    room.min,
                    room.max,
                    Vec2::new(room.min.x, room.max.y),
                    Vec2::new(room.max.x, room.min.y),
                ] {
                    for y in [0.0, 3.0] {
                        let ndc = clip_from_world.project_point3(Vec3::new(corner.x, y, corner.y));
                        assert!(
                            ndc.x.abs() < 1.0 && ndc.y.abs() < 1.0 && (0.0..=1.0).contains(&ndc.z),
                            "{name} corner {corner} at y {y} is cropped at {width}x{height}: {ndc}"
                        );
                    }
                }
            }
        }
    }
}
