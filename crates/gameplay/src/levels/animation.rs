use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::{
    builder::{Door, DoorState},
    sounds::{GameplaySound, GameplaySoundKind},
};

const SWING_SPEED: f32 = FRAC_PI_2 / 0.45;

#[derive(Component, Default)]
pub struct DoorSwing(pub f32);

pub(crate) fn animate_doors(
    time: Res<Time>,
    mut doors: Query<(&Door, &mut DoorSwing)>,
    mut sounds: MessageWriter<GameplaySound>,
) {
    for (door, mut swing) in &mut doors {
        let target = match door.state {
            DoorState::Closed => 0.0,
            DoorState::Open => FRAC_PI_2,
        };
        let step = SWING_SPEED * time.delta_secs();
        let next = swing.0 + (target - swing.0).clamp(-step, step);
        if swing.0 != next {
            swing.0 = next;
            if door.state == DoorState::Closed && next == 0.0 {
                sounds.write(GameplaySound {
                    kind: GameplaySoundKind::DoorShut,
                    position: Vec3::new(door.position.x, 1.0, door.position.y),
                });
            }
        }
    }
}
