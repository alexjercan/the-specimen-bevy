use std::{
    io::{self, BufRead, Write},
    time::Duration,
};

use bevy::{
    app::{App, AppExit, Plugin, PluginsState},
    input::mouse::MouseMotion,
    prelude::{ButtonInput, EulerRot, KeyCode, Transform, With, World},
    time::TimeUpdateStrategy,
};
use gameplay::{
    controller::{PlayerController, PlayerInput},
    levels::Escaped,
};
use serde::{Deserialize, Serialize};

use crate::TransportTimeline;

pub(super) const FRAME_TIME: Duration = Duration::from_micros(16_667);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Command {
    pub(super) tick: u64,
    #[serde(default)]
    pub(super) input: Controls,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Controls {
    w: Option<bool>,
    a: Option<bool>,
    s: Option<bool>,
    d: Option<bool>,
    shift: Option<bool>,
    f: Option<bool>,
    look: [f32; 2],
}

impl Controls {
    pub(super) fn valid(&self) -> bool {
        self.look.iter().all(|value| value.is_finite())
    }

    pub(super) fn apply(&self, world: &mut World) {
        let mut keyboard = world.resource_mut::<ButtonInput<KeyCode>>();
        for (key, held) in [
            (KeyCode::KeyW, self.w),
            (KeyCode::KeyA, self.a),
            (KeyCode::KeyS, self.s),
            (KeyCode::KeyD, self.d),
            (KeyCode::ShiftLeft, self.shift),
            (KeyCode::KeyF, self.f),
        ] {
            match held {
                Some(true) => keyboard.press(key),
                Some(false) => keyboard.release(key),
                None => continue,
            }
        }
        world.write_message(MouseMotion {
            delta: self.look.into(),
        });
    }
}

#[derive(Serialize)]
pub(super) struct Snapshot {
    tick: u64,
    player: Option<PlayerSnapshot>,
    won: bool,
}

#[derive(Serialize)]
struct PlayerSnapshot {
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
    movement: [f32; 2],
    running: bool,
}

pub(super) fn snapshot(world: &mut World, tick: u64) -> Snapshot {
    let mut players = world.query_filtered::<(&Transform, &PlayerInput), With<PlayerController>>();
    let player = players.iter(world).next().map(|(pose, input)| {
        let (yaw, pitch, _) = pose.rotation.to_euler(EulerRot::YXZ);
        PlayerSnapshot {
            position: pose.translation.to_array(),
            yaw,
            pitch,
            movement: input.movement.to_array(),
            running: input.running,
        }
    });
    let won = world
        .query_filtered::<(), (With<PlayerController>, With<Escaped>)>()
        .iter(world)
        .next()
        .is_some();
    Snapshot { tick, player, won }
}

#[derive(Serialize)]
pub(super) struct Error<'a> {
    pub(super) error: &'a str,
}

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(FRAME_TIME))
            .init_resource::<TransportTimeline>();
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
    while app
        .world()
        .resource::<TransportTimeline>()
        .first_tick
        .is_none()
    {
        app.update();
        app.world_mut()
            .resource_mut::<TransportTimeline>()
            .advance();
        if let Some(exit) = app.should_exit() {
            return exit;
        }
    }

    let mut tick = app.world().resource::<TransportTimeline>().tick().unwrap();
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
            Ok(command) if command.tick <= tick => serde_json::to_string(&Error {
                error: "tick must be greater than the current tick",
            })
            .expect("error serializes"),
            Ok(command) if !command.input.valid() => serde_json::to_string(&Error {
                error: "look values must be finite",
            })
            .expect("error serializes"),
            Ok(command) => {
                command.input.apply(app.world_mut());
                while tick < command.tick {
                    app.update();
                    app.world_mut()
                        .resource_mut::<TransportTimeline>()
                        .advance();
                    tick = app.world().resource::<TransportTimeline>().tick().unwrap();
                    if let Some(exit) = app.should_exit() {
                        return exit;
                    }
                }
                serde_json::to_string(&snapshot(app.world_mut(), tick))
                    .expect("snapshot serializes")
            }
            Err(_) => serde_json::to_string(&Error {
                error: "expected a JSON object with tick and optional input controls",
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
