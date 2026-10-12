#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use bevy::prelude::AppExit;
use clap::{Parser, ValueEnum};
use game::prelude::{AppBuilder, Frontend, VideoProfile};

const SMOKE_FRAMES: u32 = 600;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    headless: bool,
    #[arg(long)]
    stdio: bool,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long)]
    record: Option<std::path::PathBuf>,
    #[arg(long, value_enum, default_value_t = Profile::Preview, requires = "record")]
    video_profile: Profile,
}

#[derive(Clone, Copy, ValueEnum)]
enum Profile {
    Preview,
    Final,
}

impl From<Profile> for VideoProfile {
    fn from(profile: Profile) -> Self {
        match profile {
            Profile::Preview => VideoProfile::PREVIEW,
            Profile::Final => VideoProfile::FINAL,
        }
    }
}

fn main() -> AppExit {
    let cli = Cli::parse();
    if cli.record.is_some() && (cli.headless || !cli.stdio) {
        eprintln!("--record requires --stdio without --headless");
        return AppExit::error();
    }
    let frontend = if cli.headless {
        Frontend::Headless
    } else {
        Frontend::Windowed { menu: !cli.stdio }
    };
    let mut builder = AppBuilder::new(frontend);
    if cli.stdio {
        builder = builder.with_stdio();
    } else if cli.headless {
        builder = builder.bounded(SMOKE_FRAMES);
    }
    if let Some(path) = cli.record {
        builder = builder.with_recording(path, cli.video_profile.into());
    }
    if let Some(seed) = cli.seed {
        builder = builder.with_seed(seed);
    }
    builder.build().run()
}
