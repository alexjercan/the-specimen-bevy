use bevy::{audio::SpatialListener, prelude::*};
use bevy_rand::prelude::{ChaCha8Rng, GlobalRng};
use game_audio::{AmbienceActive, AudioPaused, PlaySound, Sound};
use gameplay::{
    controller::{PlayerController, PlayerControlsEnabled, PlayerInput},
    levels::{GameplaySound, GameplaySoundKind},
};
use rand_core::Rng;

use crate::menu::PauseState;

pub(crate) struct SoundGluePlugin;

impl Plugin for SoundGluePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(attach_listener).add_systems(
            Update,
            (forward_gameplay_sounds, footsteps, sync_pause, sync_ambience),
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
            GameplaySoundKind::FusePickup => Sound::FusePickup,
            GameplaySoundKind::PanelInstall => Sound::PanelInstall,
            GameplaySoundKind::LockerOpen => Sound::LockerOpen,
            GameplaySoundKind::LockerClose => Sound::LockerClose,
            GameplaySoundKind::TableEnter => Sound::TableEnter,
            GameplaySoundKind::TableLeave => Sound::TableLeave,
        };
        sounds.write(PlaySound {
            sound,
            position: Some(event.position),
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
    players: Query<(&Transform, &PlayerInput), With<PlayerController>>,
    mut tracker: Local<StepTracker>,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut sounds: MessageWriter<PlaySound>,
) {
    let Ok((transform, input)) = players.single() else {
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
    let stride = if input.running { 1.8 } else { 1.25 };
    if tracker.distance >= stride {
        tracker.distance -= stride;
        sounds.write(PlaySound {
            sound: Sound::Step((rng.next_u32() % 3) as usize),
            position: None,
        });
    }
}

fn sync_ambience(players: Query<(), With<PlayerController>>, mut active: ResMut<AmbienceActive>) {
    let value = !players.is_empty();
    if active.0 != value {
        active.0 = value;
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
