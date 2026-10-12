mod perception;

use bevy::{
    input::mouse::MouseMotion,
    prelude::{App, ButtonInput, EulerRot, Transform, With, World},
};
use gameplay::{
    devices::{Detector, DetectorReading, Flashbangs, Flashed},
    facility::Escaped,
    monster::Caught,
    player::{ControllerSettings, Flashlight, Input, PlayerController, PlayerInput, Stamina},
};
use serde::{Deserialize, Serialize};
use stdio::Playable;

use perception::{Heard, Map, Seen};

pub struct Specimen;

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Controls {
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
pub struct Snapshot {
    player: Option<PlayerSnapshot>,
    won: bool,
    game_over: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    map: Option<Map>,
    power_on: bool,
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

impl Playable for Specimen {
    type Input = Controls;
    type View = Snapshot;

    fn build(app: &mut App) {
        perception::build(app);
    }

    fn ready(world: &World) -> bool {
        world
            .components()
            .component_id::<PlayerController>()
            .is_some_and(|id| {
                world
                    .archetypes()
                    .iter()
                    .any(|a| a.contains(id) && !a.is_empty())
            })
    }

    fn apply(input: &Controls, world: &mut World) -> Result<(), String> {
        if !input.look.iter().all(|value| value.is_finite()) {
            return Err("look values must be finite".into());
        }
        let keys = world.resource::<ControllerSettings>().keys;
        for (key, held) in [
            (keys.forward, input.w),
            (keys.left, input.a),
            (keys.backward, input.s),
            (keys.right, input.d),
            (keys.sprint, input.shift),
            (keys.interact, input.f),
        ] {
            if let Some(held) = held {
                press(world, key, held);
            }
        }
        for (binding, held) in [
            (keys.flashlight, input.flashlight),
            (keys.flashbang, input.flashbang),
        ] {
            let Some(held) = held else {
                continue;
            };
            match binding {
                Input::Key(key) => press(world, key, held),
                Input::Mouse(button) => press(world, button, held),
            }
        }
        world.write_message(MouseMotion {
            delta: input.look.into(),
        });
        Ok(())
    }

    fn observe(world: &mut World, _tick: u64) -> Snapshot {
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
                    running: stamina.sprinting(),
                    flashlight_on: flashlight.on,
                    flashlight_charge: flashlight.charge,
                    flashbangs: flashbangs.map_or(0, |flashbangs| flashbangs.0),
                    flash_remaining: flashed.map_or(0.0, |flashed| flashed.remaining),
                    has_detector: detector.is_some(),
                    detector: detector
                        .and_then(|detector| detector.reading)
                        .map(detector_snapshot),
                    stamina_charge: stamina.charge,
                    stamina_exhausted: stamina.exhausted(),
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
        Snapshot {
            player,
            won,
            game_over,
            map: view.map,
            power_on: view.power_on,
            visible: view.visible,
            heard: view.heard,
        }
    }
}
