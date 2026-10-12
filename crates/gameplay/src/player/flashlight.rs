use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::logic::{PlayerController, PlayerControlsEnabled, PlayerCue};
use crate::monster::Caught;

pub const DRAIN_SECONDS: f32 = 15.0;
pub const RECHARGE_SECONDS: f32 = 10.0;
pub const RESTART_CHARGE: f32 = 0.1;

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component)]
pub struct Flashlight {
    pub charge: f32,
    pub on: bool,
}

impl Default for Flashlight {
    fn default() -> Self {
        Self {
            charge: 1.0,
            on: false,
        }
    }
}

impl Flashlight {
    pub fn toggle(&mut self) {
        if self.on {
            self.on = false;
        } else if self.charge >= RESTART_CHARGE {
            self.on = true;
        }
    }

    pub fn advance(&mut self, seconds: f32) {
        if self.on {
            self.charge = (self.charge - seconds / DRAIN_SECONDS).max(0.0);
            if self.charge == 0.0 {
                self.on = false;
            }
        } else {
            self.charge = (self.charge + seconds / RECHARGE_SECONDS).min(1.0);
        }
    }
}

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct ToggleFlashlight;

pub(super) fn attach(added: On<Add, PlayerController>, mut commands: Commands) {
    commands.entity(added.entity).insert(Flashlight::default());
}

pub(super) fn toggle(
    _: On<Start<ToggleFlashlight>>,
    enabled: Res<PlayerControlsEnabled>,
    mut players: Query<&mut Flashlight, (With<PlayerController>, Without<Caught>)>,
    mut cues: MessageWriter<PlayerCue>,
) {
    if !enabled.0 {
        return;
    }
    for mut flashlight in &mut players {
        let was_on = flashlight.on;
        flashlight.toggle();
        if flashlight.on != was_on {
            cues.write(PlayerCue::FlashlightClick);
        }
    }
}

pub(super) fn advance(
    time: Res<Time>,
    enabled: Res<PlayerControlsEnabled>,
    mut players: Query<&mut Flashlight, With<PlayerController>>,
) {
    if !enabled.0 {
        return;
    }
    for mut flashlight in &mut players {
        flashlight.advance(time.delta_secs());
    }
}
