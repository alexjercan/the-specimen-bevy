use bevy::{audio::SpatialListener, prelude::*};
use bevy_rand::prelude::{ChaCha8Rng, GlobalRng};
use game_audio::{
    AmbienceActive, AmbientEmitter, AmbientEmitters, AmbientSound, AudioPaused, ConduitAmbience,
    PlaySound, Sound,
};
use gameplay::{
    controller::{PlayerController, PlayerControlsEnabled, PlayerInput, SprintExhausted, Stamina},
    levels::{GameplaySound, GameplaySoundKind, Room, Walls},
};
use rand_core::Rng;

use crate::menu::PauseState;

pub(crate) struct SoundGluePlugin;

impl Plugin for SoundGluePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientEmitters(vec![
            AmbientEmitter {
                sound: AmbientSound::Roomtone,
                position: None,
                volume: 0.12,
            },
            AmbientEmitter {
                sound: AmbientSound::LowPressure,
                position: None,
                volume: 0.04,
            },
            AmbientEmitter {
                sound: AmbientSound::Conduit,
                position: None,
                volume: 0.055,
            },
            AmbientEmitter {
                sound: AmbientSound::Boiler,
                position: Some(Vec3::new(-10.0, 1.0, 0.0)),
                volume: 0.17,
            },
            AmbientEmitter {
                sound: AmbientSound::Tank,
                position: Some(Vec3::new(0.0, 1.3, 0.0)),
                volume: 0.15,
            },
            AmbientEmitter {
                sound: AmbientSound::Vent,
                position: Some(Vec3::new(-13.65, 0.45, -1.1)),
                volume: 0.09,
            },
            AmbientEmitter {
                sound: AmbientSound::Vent,
                position: Some(Vec3::new(-13.65, 1.9, -20.0)),
                volume: 0.09,
            },
            AmbientEmitter {
                sound: AmbientSound::Vent,
                position: Some(Vec3::new(13.65, 1.9, -21.25)),
                volume: 0.09,
            },
            AmbientEmitter {
                sound: AmbientSound::VentWind,
                position: Some(Vec3::new(-13.65, 0.45, -1.1)),
                volume: 0.035,
            },
            AmbientEmitter {
                sound: AmbientSound::VentWind,
                position: Some(Vec3::new(-13.65, 1.9, -20.0)),
                volume: 0.035,
            },
            AmbientEmitter {
                sound: AmbientSound::VentWind,
                position: Some(Vec3::new(13.65, 1.9, -21.25)),
                volume: 0.035,
            },
            AmbientEmitter {
                sound: AmbientSound::CoolBuzz,
                position: Some(Vec3::new(0.0, 2.7, -27.5)),
                volume: 0.045,
            },
            AmbientEmitter {
                sound: AmbientSound::CoolBuzz,
                position: Some(Vec3::new(10.0, 2.7, -27.5)),
                volume: 0.045,
            },
            AmbientEmitter {
                sound: AmbientSound::CoolBuzz,
                position: Some(Vec3::new(5.0, 2.7, -20.0)),
                volume: 0.045,
            },
        ]));
        app.add_observer(attach_listener).add_systems(
            Update,
            (
                forward_gameplay_sounds,
                forward_sprint_exhaustion,
                footsteps,
                sync_pause,
                sync_ambience,
                boiler_ticks,
                faucet_bursts,
            ),
        );
    }
}

fn attach_listener(added: On<Add, PlayerController>, mut commands: Commands) {
    commands
        .entity(added.entity)
        .insert(SpatialListener::new(0.18));
}

fn forward_gameplay_sounds(
    mut events: MessageReader<GameplaySound>,
    mut sounds: MessageWriter<PlaySound>,
) {
    for event in events.read() {
        let sound = match event.kind {
            GameplaySoundKind::DoorUnlatch => Sound::DoorUnlatch,
            GameplaySoundKind::DoorSwing => Sound::DoorSwing,
            GameplaySoundKind::DoorShut => Sound::DoorShut,
            GameplaySoundKind::DoorLocked => Sound::DoorLocked,
            GameplaySoundKind::FuseSlot(slot) => Sound::FuseSlot(slot),
            GameplaySoundKind::FuseComplete => Sound::FuseComplete,
            GameplaySoundKind::FlashlightClick => Sound::FlashlightClick,
            GameplaySoundKind::LockerOpen => Sound::LockerOpen,
            GameplaySoundKind::LockerClose => Sound::LockerClose,
            GameplaySoundKind::TableEnter => Sound::TableEnter,
            GameplaySoundKind::TableLeave => Sound::TableLeave,
        };
        sounds.write(PlaySound {
            sound,
            position: match event.kind {
                GameplaySoundKind::FuseSlot(_)
                | GameplaySoundKind::FuseComplete
                | GameplaySoundKind::FlashlightClick => None,
                _ => Some(event.position),
            },
        });
    }
}

fn forward_sprint_exhaustion(
    mut exhausted: MessageReader<SprintExhausted>,
    mut sounds: MessageWriter<PlaySound>,
) {
    for _ in exhausted.read() {
        sounds.write(PlaySound {
            sound: Sound::SprintExhausted,
            position: None,
        });
    }
}

#[derive(Default)]
struct StepTracker {
    last: Option<Vec3>,
    distance: f32,
}

fn footsteps(
    enabled: Res<PlayerControlsEnabled>,
    players: Query<(&Transform, &PlayerInput, &Stamina), With<PlayerController>>,
    mut tracker: Local<StepTracker>,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut sounds: MessageWriter<PlaySound>,
) {
    let Ok((transform, input, stamina)) = players.single() else {
        tracker.last = None;
        tracker.distance = 0.0;
        return;
    };
    let position = transform.translation;
    let previous = tracker.last.replace(position);
    if !enabled.0 || input.movement == Vec2::ZERO {
        return;
    }
    let Some(previous) = previous else { return };
    tracker.distance += (position.xz() - previous.xz()).length().min(0.5);
    let stride = if stamina.sprinting { 1.8 } else { 1.25 };
    if tracker.distance >= stride {
        tracker.distance -= stride;
        sounds.write(PlaySound {
            sound: Sound::Step((rng.next_u32() % 3) as usize),
            position: None,
        });
    }
}

fn sync_ambience(
    players: Query<&Transform, With<PlayerController>>,
    rooms: Query<(&Room, &Walls)>,
    mut active: ResMut<AmbienceActive>,
    mut conduit: ResMut<ConduitAmbience>,
) {
    let player = players.iter().next();
    active.0 = player.is_some();
    conduit.0 = player.is_some_and(|player| {
        let position = player.translation.xz();
        rooms
            .iter()
            .any(|(room, walls)| walls.0 == "wall_conduit" && room.0.contains(position))
    });
}

fn boiler_ticks(
    time: Res<Time>,
    paused: Res<AudioPaused>,
    players: Query<&Transform, With<PlayerController>>,
    mut remaining: Local<f32>,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut sounds: MessageWriter<PlaySound>,
) {
    let Ok(player) = players.single() else {
        *remaining = 8.0;
        return;
    };
    if paused.0 {
        return;
    }
    *remaining -= time.delta_secs();
    if *remaining > 0.0 {
        return;
    }
    *remaining = 6.0 + 14.0 * (rng.next_u32() as f64 / (u32::MAX as f64 + 1.0)) as f32;
    let position = Vec3::new(-10.0, 1.2, 0.0);
    if player.translation.distance(position) <= 18.0 {
        sounds.write(PlaySound {
            sound: Sound::BoilerTick,
            position: Some(position),
        });
    }
}

fn faucet_bursts(
    time: Res<Time>,
    paused: Res<AudioPaused>,
    players: Query<&Transform, With<PlayerController>>,
    mut remaining: Local<f32>,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut sounds: MessageWriter<PlaySound>,
) {
    let Ok(player) = players.single() else {
        *remaining = 10.0;
        return;
    };
    if paused.0 {
        return;
    }
    *remaining -= time.delta_secs();
    if *remaining > 0.0 {
        return;
    }
    *remaining = 10.0 + 8.0 * (rng.next_u32() as f64 / (u32::MAX as f64 + 1.0)) as f32;
    let position = Vec3::new(-13.4, 1.5, 0.0);
    if player.translation.distance(position) <= 12.0 {
        sounds.write(PlaySound {
            sound: Sound::FaucetBurst,
            position: Some(position),
        });
    }
}

fn sync_pause(state: Option<Res<State<PauseState>>>, mut paused: ResMut<AudioPaused>) {
    let value = state.is_some_and(|state| *state.get() == PauseState::Paused);
    if paused.0 != value {
        paused.0 = value;
    }
}

#[cfg(test)]
#[path = "../../tests/unit/sound.rs"]
mod tests;
