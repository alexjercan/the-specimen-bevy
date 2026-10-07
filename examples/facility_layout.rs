use game::{
    autopilot::state_is,
    gameplay::levels::{build_first_floor, RenderCeilings},
    prelude::*,
};

const SCREENSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/visuals/screenshots/facility_layout_draft.png"
);

fn main() {
    let script = AutopilotPlugin::new()
        .step("wait for facility assets")
        .until(state_is(GameAssetsState::Ready), 30.0)
        .add()
        .step("allow render to settle")
        .until(frames(180), 20.0)
        .add()
        .step("capture first floor")
        .act(|world| screenshot_start(world, SCREENSHOT_PATH))
        .until(screenshot_written_at(SCREENSHOT_PATH), 15.0)
        .add();

    let mut app = AppBuilder::new().build();
    app.insert_resource(RenderCeilings(false))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 20.0,
            ..default()
        })
        .add_plugins((CapturePlugin::default(), script))
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(GameAssetsState::Ready), build_first_floor)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 35.0, 18.0).looking_at(Vec3::new(0.0, 0.0, -14.0), Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 900_000.0,
            range: 70.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 18.0, -14.0),
    ));
}
