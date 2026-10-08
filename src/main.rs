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
    #[arg(long)]
    record: Option<std::path::PathBuf>,
    #[arg(long)]
    mute: bool,
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
    if cli.record.is_some() && (cli.norender || !cli.transport) {
        eprintln!("--record requires rendered --transport");
        return AppExit::error();
    }
    let builder = match cli.record {
        Some(path) => builder.with_recording(path),
        None => builder,
    };
    let builder = match cli.seed {
        Some(seed) => builder.with_seed(seed),
        None => builder,
    };
    let builder = if cli.mute {
        builder.with_muted_audio()
    } else {
        builder
    };
    builder.build().run()
}
