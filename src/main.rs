use bevy::{
    ecs::query::QueryFilter,
    pbr::{DistanceFog, FogFalloff},
};
use clap::{error::ErrorKind, CommandFactory, Parser};
use game::{prelude::*, probe::world_instances_ready};

const EYE_HEIGHT: f32 = 1.6;

#[derive(Parser)]
struct Cli {
    #[arg(long, conflicts_with = "probe")]
    norender: bool,
    #[command(flatten)]
    probe: ProbeArgs,
}

fn main() -> AppExit {
    let cli = Cli::parse();
    if cli.norender {
        return AppBuilder::headless()
            .build()
            .add_plugins(FacilityPlugin::new(first_floor()))
            .add_systems(Update, stop_after_eight_frames)
            .run();
    }
    let mut app = AppBuilder::new().build();
    app.add_plugins((FacilityPlugin::new(first_floor()), ControllerPlugin))
        .add_observer(view_from_start);
    if let Some(config) = cli.probe.config() {
        match ProbePlugin::new(config, facility_ready) {
            Ok(plugin) => app.add_plugins(plugin),
            Err(error) => Cli::command()
                .error(ErrorKind::ValueValidation, error)
                .exit(),
        };
    }
    app.run()
}

fn facility_ready(world: &World) -> Readiness {
    match world
        .get_resource::<State<GameAssetsState>>()
        .map(State::get)
    {
        Some(GameAssetsState::Ready) => {}
        Some(GameAssetsState::Failed) => {
            return Readiness::Failed("facility assets failed to load".to_string())
        }
        _ => return Readiness::Waiting,
    }
    let ready = count::<With<Facility>>(world) > 0
        && count::<With<Camera3d>>(world) > 0
        && world_instances_ready(world);
    if ready {
        Readiness::Ready
    } else {
        Readiness::Waiting
    }
}

fn count<F: QueryFilter>(world: &World) -> usize {
    world
        .try_query_filtered::<(), F>()
        .map_or(0, |mut query| query.iter(world).count())
}

fn view_from_start(start: On<Add, PlayerStart>, starts: Query<&Transform>, mut commands: Commands) {
    let Ok(start) = starts.get(start.entity) else {
        return;
    };
    commands.spawn((
        PlayerController,
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
