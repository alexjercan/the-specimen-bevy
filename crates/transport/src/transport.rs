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
use game_settings::{parse_binding, GameSettings, InputBinding};
use gameplay::{
    controller::{Flashlight, PlayerController, PlayerInput, Stamina},
    levels::{Caught, Detector, DetectorReading, Escaped, Flashbangs, Flashed},
};
use serde::{Deserialize, Serialize};

use crate::{
    perception::{self, Heard, Map, Seen},
    TransportTimeline,
};

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
    flashlight: Option<bool>,
    flashbang: Option<bool>,
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
        let keys = world
            .get_resource::<GameSettings>()
            .map(|settings| settings.keys.clone())
            .unwrap_or_default();
        for (binding, held) in [
            (&keys.flashlight, self.flashlight),
            (&keys.flashbang, self.flashbang),
        ] {
            let Some(held) = held else {
                continue;
            };
            match parse_binding(binding) {
                Some(InputBinding::Key(key)) => press(world, key, held),
                Some(InputBinding::Mouse(button)) => press(world, button, held),
                None => {}
            }
        }
        world.write_message(MouseMotion {
            delta: self.look.into(),
        });
    }
}

fn press<T: Copy + Eq + std::hash::Hash + Send + Sync + 'static>(
    world: &mut World,
    input: T,
    held: bool,
) {
    let mut buttons = world.resource_mut::<ButtonInput<T>>();
    if held {
        buttons.press(input);
    } else {
        buttons.release(input);
    }
}

#[derive(Serialize)]
struct Snapshot {
    tick: u64,
    player: Option<PlayerSnapshot>,
    won: bool,
    game_over: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    map: Option<Map>,
    power_on: Option<bool>,
    visible: Vec<Seen>,
    heard: Vec<Heard>,
}

#[derive(Serialize)]
struct PlayerSnapshot {
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
    movement: [f32; 2],
    running: bool,
    flashlight_on: bool,
    flashlight_charge: f32,
    flashbangs: usize,
    flash_remaining: f32,
    has_detector: bool,
    detector: Option<DetectorSnapshot>,
    stamina_charge: f32,
    stamina_exhausted: bool,
}

#[derive(Serialize)]
struct DetectorSnapshot {
    distance_m: f32,
    bearing_deg: f32,
}

fn detector_snapshot(reading: DetectorReading) -> DetectorSnapshot {
    DetectorSnapshot {
        distance_m: reading.distance,
        bearing_deg: reading.bearing.to_degrees(),
    }
}

pub fn snapshot(world: &mut World, tick: u64) -> String {
    let mut players = world.query_filtered::<(
        &Transform,
        &PlayerInput,
        &Flashlight,
        &Stamina,
        Option<&Flashbangs>,
        Option<&Flashed>,
        Option<&Detector>,
    ), With<PlayerController>>();
    let player = players.iter(world).next().map(
        |(pose, input, flashlight, stamina, flashbangs, flashed, detector)| {
            let (yaw, pitch, _) = pose.rotation.to_euler(EulerRot::YXZ);
            PlayerSnapshot {
                position: pose.translation.to_array(),
                yaw,
                pitch,
                movement: input.movement.to_array(),
                running: stamina.sprinting,
                flashlight_on: flashlight.on,
                flashlight_charge: flashlight.charge,
                flashbangs: flashbangs.map_or(0, |flashbangs| flashbangs.0),
                flash_remaining: flashed.map_or(0.0, |flashed| flashed.remaining),
                has_detector: detector.is_some(),
                detector: detector
                    .and_then(|detector| detector.reading)
                    .map(detector_snapshot),
                stamina_charge: stamina.charge,
                stamina_exhausted: stamina.exhausted,
            }
        },
    );
    let won = world
        .query_filtered::<(), (With<PlayerController>, With<Escaped>)>()
        .iter(world)
        .next()
        .is_some();
    let game_over = world
        .query_filtered::<(), (With<PlayerController>, With<Caught>)>()
        .iter(world)
        .next()
        .is_some();
    let view = perception::view(world);
    serde_json::to_string(&Snapshot {
        tick,
        player,
        won,
        game_over,
        map: view.map,
        power_on: view.power_on,
        visible: view.visible,
        heard: view.heard,
    })
    .expect("snapshot serializes")
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
        perception::build(app);
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
    if writeln!(output, "{}", snapshot(app.world_mut(), tick))
        .and_then(|()| output.flush())
        .is_err()
    {
        return AppExit::error();
    }
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
                snapshot(app.world_mut(), tick)
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
