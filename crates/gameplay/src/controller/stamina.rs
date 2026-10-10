use bevy::prelude::*;

pub const DRAIN_SECONDS: f32 = 5.0;
pub const RECHARGE_SECONDS: f32 = 8.0;
pub const RESTART_CHARGE: f32 = 0.25;

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component)]
pub struct Stamina {
    pub charge: f32,
    pub sprinting: bool,
    pub exhausted: bool,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            charge: 1.0,
            sprinting: false,
            exhausted: false,
        }
    }
}

impl Stamina {
    pub fn advance(&mut self, wants_run: bool, moving: bool, seconds: f32) -> bool {
        if !wants_run && self.charge >= RESTART_CHARGE {
            self.exhausted = false;
        }
        self.sprinting = wants_run && moving && !self.exhausted && self.charge > 0.0;
        if self.sprinting {
            self.charge = (self.charge - seconds.max(0.0) / DRAIN_SECONDS).max(0.0);
            if self.charge == 0.0 {
                self.sprinting = false;
                self.exhausted = true;
                return true;
            }
        } else {
            self.charge = (self.charge + seconds.max(0.0) / RECHARGE_SECONDS).min(1.0);
        }
        false
    }
}

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct SprintExhausted {
    pub position: Vec3,
}
