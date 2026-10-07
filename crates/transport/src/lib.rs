use std::{
    io::{self, BufRead, Write},
    time::Duration,
};

use bevy::{
    app::{App, AppExit, Plugin, PluginsState},
    time::TimeUpdateStrategy,
};
use serde::{Deserialize, Serialize};

const FRAME_TIME: Duration = Duration::from_micros(16_667);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    tick: u64,
}

#[derive(Serialize)]
struct Snapshot {
    tick: u64,
}

#[derive(Serialize)]
struct Error<'a> {
    error: &'a str,
}

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(FRAME_TIME));
        app.set_runner(|app| run(app, io::stdin().lock(), io::stdout().lock()));
    }
}

pub fn run<R: BufRead, W: Write>(mut app: App, input: R, mut output: W) -> AppExit {
    if app.plugins_state() != PluginsState::Cleaned {
        while app.plugins_state() == PluginsState::Adding {
            bevy::tasks::tick_global_task_pools_on_main_thread();
        }
        app.finish();
        app.cleanup();
    }
    app.update();
    if let Some(exit) = app.should_exit() {
        return exit;
    }

    let mut tick = 0_u64;
    for line in input.lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("transport read failed: {error}");
                return AppExit::error();
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let result = match serde_json::from_str::<Command>(&line) {
            Ok(command) if command.tick > tick => {
                while tick < command.tick {
                    app.update();
                    tick += 1;
                    if let Some(exit) = app.should_exit() {
                        return exit;
                    }
                }
                serde_json::to_string(&Snapshot { tick }).expect("snapshot serializes")
            }
            Ok(_) => serde_json::to_string(&Error {
                error: "tick must be greater than the current tick",
            })
            .expect("error serializes"),
            Err(_) => serde_json::to_string(&Error {
                error: "expected a JSON object with only a nonnegative integer tick",
            })
            .expect("error serializes"),
        };
        if writeln!(output, "{result}")
            .and_then(|()| output.flush())
            .is_err()
        {
            return AppExit::error();
        }
    }
    AppExit::Success
}
