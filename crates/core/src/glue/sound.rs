use bevy::prelude::*;
use bevy_rand::prelude::{ChaCha8Rng, GlobalRng};
use game_audio::{
    AmbienceActive, AmbientEmitter, AmbientEmitters, AudioPaused, ConduitAmbience, PlaySound,
    PlaySoundFrom, Sound,
};
use gameplay::{
    controller::{PlayerController, PlayerControlsEnabled, PlayerInput, Stamina},
    levels::{AmbientSource, IntermittentSound, Room, Walls},
};
use rand_core::Rng;

use crate::menu::PauseState;

pub(crate) struct SoundGluePlugin;

impl Plugin for SoundGluePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientEmitters(Vec::new()));
        app.add_systems(
            Update,
            (
                footsteps,
                sync_pause,
                sync_ambience,
                sync_emitters,
                play_intermittent_sounds,
            ),
        );
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

fn sync_emitters(sources: Query<(Entity, &AmbientSource)>, mut emitters: ResMut<AmbientEmitters>) {
    let mut current: Vec<_> = sources
        .iter()
        .map(|(entity, source)| {
            (
                entity,
                AmbientEmitter {
                    source: entity,
                    sound: source.kind,
                    spatial: source.kind.spatial(),
                    volume: source.volume,
                },
            )
        })
        .collect();
    current.sort_by_key(|(entity, _)| entity.to_bits());
    emitters.0 = current.into_iter().map(|(_, emitter)| emitter).collect();
}

fn play_intermittent_sounds(
    time: Res<Time>,
    paused: Res<AudioPaused>,
    players: Query<&Transform, With<PlayerController>>,
    mut sources: Query<
        (Entity, &GlobalTransform, &mut IntermittentSound),
        Without<PlayerController>,
    >,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut sounds: MessageWriter<PlaySoundFrom>,
) {
    let Ok(player) = players.single() else { return };
    if paused.0 {
        return;
    }
    for (entity, transform, mut source) in &mut sources {
        source.remaining -= time.delta_secs();
        if source.remaining > 0.0 {
            continue;
        }
        source.remaining = source.interval
            + source.variation * (rng.next_u32() as f64 / (u32::MAX as f64 + 1.0)) as f32;
        let position = transform.transform_point(source.offset);
        if player.translation.distance(position) > source.range {
            continue;
        }
        sounds.write(PlaySoundFrom {
            sound: source.kind,
            source: entity,
            offset: source.offset,
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
