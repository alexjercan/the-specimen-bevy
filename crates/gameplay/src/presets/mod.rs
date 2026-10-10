mod insane;
mod normal;

use bevy::prelude::*;

use crate::controller::RUN_SPEED;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MonsterWalkSpeed {
    Slow,
    Fast,
}

impl MonsterWalkSpeed {
    pub const fn value(self) -> f32 {
        match self {
            Self::Slow => 0.5,
            Self::Fast => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MonsterChaseSpeed {
    Normal,
    Fast,
}

impl MonsterChaseSpeed {
    pub const fn value(self) -> f32 {
        match self {
            Self::Normal => RUN_SPEED + 1.0,
            Self::Fast => RUN_SPEED * 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MonsterWalkHearing {
    Short,
    Long,
}

impl MonsterWalkHearing {
    pub const fn value(self) -> f32 {
        match self {
            Self::Short => 3.0,
            Self::Long => 6.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MonsterRunHearing {
    Short,
    Long,
}

impl MonsterRunHearing {
    pub const fn value(self) -> f32 {
        match self {
            Self::Short => 9.0,
            Self::Long => 18.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GameMode {
    #[default]
    Normal,
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
            GameMode::Insane => insane::PRESET,
        }
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
