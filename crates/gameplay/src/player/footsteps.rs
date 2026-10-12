use bevy::prelude::*;
use bevy_rand::prelude::{ChaCha8Rng, GlobalRng};
use rand_core::Rng;

use super::{
    logic::{PlayerController, PlayerControlsEnabled, PlayerCue, PlayerInput},
    stamina::Stamina,
};

const WALK_STRIDE: f32 = 1.25;
const SPRINT_STRIDE: f32 = 1.8;
const MAX_FRAME_DISTANCE: f32 = 0.5;
const STEP_VARIANTS: u32 = 3;

#[derive(Default)]
pub(super) struct StepTracker {
    last: Option<Vec3>,
    distance: f32,
}

pub(super) fn footsteps(
    enabled: Res<PlayerControlsEnabled>,
    players: Query<(&Transform, &PlayerInput, &Stamina), With<PlayerController>>,
    mut tracker: Local<StepTracker>,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut cues: MessageWriter<PlayerCue>,
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
    tracker.distance += (position.xz() - previous.xz())
        .length()
        .min(MAX_FRAME_DISTANCE);
    let stride = if stamina.sprinting() {
        SPRINT_STRIDE
    } else {
        WALK_STRIDE
    };
    if tracker.distance >= stride {
        tracker.distance -= stride;
        cues.write(PlayerCue::Step((rng.next_u32() % STEP_VARIANTS) as usize));
    }
}
