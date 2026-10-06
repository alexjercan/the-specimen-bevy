use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    norender: bool,
}

fn main() {
    let mut app = App::new();
    if Cli::parse().norender {
        app.add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(10))),
        )
        .add_systems(Update, stop_after_eight_frames);
    } else {
        app.add_plugins(DefaultPlugins).add_systems(Startup, setup);
    }
    app.run();
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
