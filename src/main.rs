use bevy::prelude::AppExit;
use clap::Parser;
use game::prelude::AppBuilder;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    norender: bool,
    #[arg(long)]
    transport: bool,
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
    builder.build().run()
}
