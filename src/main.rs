use bevy::prelude::AppExit;
use clap::Parser;
use game::prelude::AppBuilder;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    norender: bool,
}

fn main() -> AppExit {
    let cli = Cli::parse();
    if cli.norender {
        AppBuilder::headless().build().run()
    } else {
        AppBuilder::new().build().run()
    }
}
