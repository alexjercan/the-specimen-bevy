use clap::Parser;
use game::prelude::*;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    norender: bool,
}

fn main() {
    if Cli::parse().norender {
        AppBuilder::headless()
            .build()
            .add_systems(Update, stop_after_eight_frames)
            .run();
    } else {
        AppBuilder::new().build().add_systems(Startup, setup).run();
    }
}

// TODO(horror_game_bevy): Replace the greeting with the first game scene.
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(Text::new("Hello, world!"));
}

// TODO(horror_game_bevy): Replace this bounded placeholder with shared game flow.
fn stop_after_eight_frames(mut frames: Local<u32>, mut exit: MessageWriter<AppExit>) {
    *frames += 1;
    if *frames >= 8 {
        exit.write(AppExit::Success);
    }
}
