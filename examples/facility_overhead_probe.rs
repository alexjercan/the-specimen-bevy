#[path = "support/probe.rs"]
mod probe;

use game::prelude::*;

fn main() -> AppExit {
    probe::app(false)
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 20.0,
            ..default()
        })
        .add_systems(Startup, spawn_camera)
        .run()
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
