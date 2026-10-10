mod insane;
mod normal;

use bevy::prelude::*;

use crate::controller::RUN_SPEED;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MonsterWalkSpeed {
    Slow,
    Medium,
    Fast,
}

impl MonsterWalkSpeed {
    pub const fn value(self) -> f32 {
        match self {
            Self::Slow => 0.5,
            Self::Medium => 0.75,
            Self::Fast => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MonsterChaseSpeed {
    Normal,
    Medium,
    Fast,
}

impl MonsterChaseSpeed {
    pub const fn value(self) -> f32 {
        match self {
            Self::Normal => RUN_SPEED + 1.0,
            Self::Medium => (RUN_SPEED + 1.0 + RUN_SPEED * 2.0) / 2.0,
            Self::Fast => RUN_SPEED * 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MonsterWalkHearing {
    Short,
    Medium,
    Long,
}

impl MonsterWalkHearing {
    pub const fn value(self) -> f32 {
        match self {
            Self::Short => 3.0,
            Self::Medium => 4.5,
            Self::Long => 6.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MonsterRunHearing {
    Short,
    Medium,
    Long,
}

impl MonsterRunHearing {
    pub const fn value(self) -> f32 {
        match self {
            Self::Short => 9.0,
            Self::Medium => 13.5,
            Self::Long => 18.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GameMode {
    #[default]
    Normal,
    Custom,
    Insane,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MonsterPreset {
    pub walk_speed: MonsterWalkSpeed,
    pub chase_speed: MonsterChaseSpeed,
    pub walk_hearing: MonsterWalkHearing,
    pub run_hearing: MonsterRunHearing,
}

#[derive(Resource, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GamePreset {
    pub mode: GameMode,
    pub monster: MonsterPreset,
    pub sprint: bool,
    pub flashbangs: bool,
    pub detector: bool,
    pub blackout: bool,
}

impl GamePreset {
    pub const fn of(mode: GameMode) -> Self {
        match mode {
            GameMode::Normal => normal::PRESET,
            GameMode::Custom => Self {
                mode,
                ..normal::PRESET
            },
            GameMode::Insane => insane::PRESET,
        }
    }
}

impl GamePreset {
    pub fn insane_equivalent(self) -> bool {
        self.monster == insane::PRESET.monster
            && self.sprint == insane::PRESET.sprint
            && self.flashbangs == insane::PRESET.flashbangs
            && self.detector == insane::PRESET.detector
            && self.blackout == insane::PRESET.blackout
    }
}

impl Default for GamePreset {
    fn default() -> Self {
        Self::of(GameMode::Normal)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/presets.rs"]
mod tests;
