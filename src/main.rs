use bevy::pbr::{DistanceFog, FogFalloff};
use clap::Parser;
use game::prelude::*;

const EYE_HEIGHT: f32 = 1.6;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    norender: bool,
}

fn main() {
    if Cli::parse().norender {
        AppBuilder::headless()
            .build()
            .add_plugins(FacilityPlugin)
            .add_systems(Update, stop_after_eight_frames)
            .run();
    } else {
        AppBuilder::new()
            .build()
            .add_plugins((FacilityPlugin, FacilityRenderPlugin))
            .add_observer(view_from_start)
            .run();
    }
}

fn view_from_start(start: On<Add, PlayerStart>, starts: Query<&Transform>, mut commands: Commands) {
    let Ok(start) = starts.get(start.entity) else {
        return;
    };
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 55.0_f32.to_radians(),
            ..default()
        }),
        Transform::from_translation(start.translation + Vec3::Y * EYE_HEIGHT)
            .with_rotation(start.rotation),
        DistanceFog {
            color: Color::srgb(0.015, 0.018, 0.022),
            falloff: FogFalloff::ExponentialSquared { density: 0.045 },
            ..default()
        },
    ));
}

// TODO(horror_game_bevy): Replace this bounded placeholder with shared game flow.
fn stop_after_eight_frames(mut frames: Local<u32>, mut exit: MessageWriter<AppExit>) {
    *frames += 1;
    if *frames >= 8 {
        exit.write(AppExit::Success);
    }
}
