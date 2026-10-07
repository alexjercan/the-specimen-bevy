use bevy::prelude::AppExit;
use clap::Parser;
use game::prelude::AppBuilder;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    norender: bool,
    #[arg(long)]
    transport: bool,
    #[arg(long)]
    seed: Option<u64>,
}

fn main() -> AppExit {
    let cli = Cli::parse();
    let builder = if cli.norender {
        AppBuilder::headless()
    } else {
        AppBuilder::new().with_menu()
    };
    let builder = if cli.transport {
        builder.with_transport()
    } else {
        builder
    };
    let builder = match cli.seed {
        Some(seed) => builder.with_seed(seed),
        None => builder,
    };
    builder.build().run()
}
