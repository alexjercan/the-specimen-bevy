use bevy::{ecs::query::QueryFilter, world_serialization::WorldAssetRoot};
use clap::{error::ErrorKind, CommandFactory, Parser};
use game::{
    gameplay::levels::{build_first_floor, RenderCeilings},
    prelude::*,
    probe::world_instances_ready,
};

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    probe: ProbeArgs,
}

pub fn app(render_ceilings: bool) -> App {
    let cli = Cli::parse();
    let config = cli.probe.config();
    let plugin = ProbePlugin::new(config, facility_ready).unwrap_or_else(|error| {
        Cli::command()
            .error(ErrorKind::ValueValidation, error)
            .exit()
    });
    let mut app = AppBuilder::new().build();
    app.insert_resource(RenderCeilings(render_ceilings))
        .add_plugins(plugin)
        .add_systems(OnEnter(GameAssetsState::Ready), build_first_floor);
    app
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
    let ready = count::<With<Room>>(world) > 0
        && count::<With<Camera3d>>(world) > 0
        && count::<With<WorldAssetRoot>>(world) > 0
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
