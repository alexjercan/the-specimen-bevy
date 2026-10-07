#[path = "support/probe.rs"]
mod probe;

use bevy::pbr::{DistanceFog, FogFalloff};
use game::prelude::*;

fn main() -> AppExit {
    probe::app(true).add_systems(Startup, spawn_camera).run()
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 55.0_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 1.6, -5.0),
        DistanceFog {
            color: Color::srgb(0.015, 0.018, 0.022),
            falloff: FogFalloff::ExponentialSquared { density: 0.045 },
            ..default()
        },
    ));
}
