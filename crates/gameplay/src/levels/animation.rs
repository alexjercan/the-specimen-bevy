use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use game_audio::{PlaySourceSound, Sound};

use super::builder::{Door, DoorState};

const SWING_SPEED: f32 = FRAC_PI_2 / 0.45;

#[derive(Component, Default)]
pub struct DoorSwing(pub f32);

pub(crate) fn animate_doors(
    time: Res<Time>,
    mut doors: Query<(Entity, &Door, &mut DoorSwing)>,
    mut sounds: MessageWriter<PlaySourceSound>,
) {
    for (entity, door, mut swing) in &mut doors {
        let target = match door.state {
            DoorState::Closed => 0.0,
            DoorState::Open => FRAC_PI_2,
        };
        let step = SWING_SPEED * time.delta_secs();
        let next = swing.0 + (target - swing.0).clamp(-step, step);
        if swing.0 != next {
            swing.0 = next;
            if door.state == DoorState::Closed && next == 0.0 {
                sounds.write(PlaySourceSound {
                    source: entity,
                    sound: Sound::DoorShut,
                });
            }
        }
    }
}
