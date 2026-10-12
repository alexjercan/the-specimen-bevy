use bevy::prelude::*;

pub const DRAIN_SECONDS: f32 = 5.0;
pub const RECHARGE_SECONDS: f32 = 8.0;
pub const RESTART_CHARGE: f32 = 0.25;

#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StaminaState {
    #[default]
    Rested,
    Sprinting,
    Exhausted,
    Locked,
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component)]
pub struct Stamina {
    pub charge: f32,
    pub state: StaminaState,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            charge: 1.0,
            state: StaminaState::Rested,
        }
    }
}

impl Stamina {
    pub fn spent() -> Self {
        Self {
            charge: 0.0,
            state: StaminaState::Locked,
        }
    }

    pub fn sprinting(&self) -> bool {
        self.state == StaminaState::Sprinting
    }

    pub fn exhausted(&self) -> bool {
        matches!(self.state, StaminaState::Exhausted | StaminaState::Locked)
    }

    pub fn stop_sprinting(&mut self) {
        if self.sprinting() {
            self.state = StaminaState::Rested;
        }
    }

    pub fn advance(&mut self, wants_run: bool, moving: bool, seconds: f32) -> bool {
        if self.state == StaminaState::Locked {
            *self = Self::spent();
            return false;
        }
        if !wants_run && self.charge >= RESTART_CHARGE && self.state == StaminaState::Exhausted {
            self.state = StaminaState::Rested;
        }
        let sprinting = wants_run && moving && !self.exhausted() && self.charge > 0.0;
        if !self.exhausted() {
            self.state = if sprinting {
                StaminaState::Sprinting
            } else {
                StaminaState::Rested
            };
        }
        if sprinting {
            self.charge = (self.charge - seconds.max(0.0) / DRAIN_SECONDS).max(0.0);
            if self.charge == 0.0 {
                self.state = StaminaState::Exhausted;
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
