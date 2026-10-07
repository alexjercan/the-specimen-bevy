use game::{
    autopilot::state_is,
    gameplay::levels::{build_first_floor, LightIntensity, RenderCeilings},
    prelude::*,
};

const SCREENSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/visuals/screenshots/facility_layout_draft.png"
);
const DARK_SCREENSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/visuals/screenshots/facility_lighting_dark.png"
);

fn main() {
    let dark = std::env::args().any(|arg| arg == "--dark");
    let screenshot = if dark {
        DARK_SCREENSHOT_PATH
    } else {
        SCREENSHOT_PATH
    };
    let script = AutopilotPlugin::new()
        .step("wait for facility assets")
        .until(state_is(GameAssetsState::Ready), 30.0)
        .add()
        .step("allow render to settle")
        .until(frames(180), 20.0)
        .add()
        .step("capture first floor")
        .act(move |world| screenshot_start(world, screenshot))
        .until(screenshot_written_at(screenshot), 15.0)
        .add();

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
        app.add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(GameAssetsState::Ready), build_first_floor);
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 35.0, 18.0).looking_at(Vec3::new(0.0, 0.0, -14.0), Vec3::Y),
    ));
    commands.spawn((
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
        Transform::from_xyz(0.0, 18.0, -14.0),
    ));
}
