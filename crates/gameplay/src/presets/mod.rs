mod insane;
mod normal;

use bevy::prelude::*;

use crate::player::RUN_SPEED;

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PresetRules {
    pub monster: MonsterPreset,
    pub sprint: bool,
    pub flashbangs: bool,
    pub detector: bool,
    pub blackout: bool,
}

// NOTE: only Custom carries its own rules, so Normal and Insane cannot drift from their fixed rules.
#[derive(Resource, Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GamePreset {
    #[default]
    Normal,
    Custom(PresetRules),
    Insane,
}

impl GamePreset {
    pub const fn of(mode: GameMode) -> Self {
        match mode {
            GameMode::Normal => Self::Normal,
            GameMode::Custom => Self::Custom(PresetRules::NORMAL),
            GameMode::Insane => Self::Insane,
        }
    }

    pub const fn mode(self) -> GameMode {
        match self {
            Self::Normal => GameMode::Normal,
            Self::Custom(_) => GameMode::Custom,
            Self::Insane => GameMode::Insane,
        }
    }

    pub const fn rules(self) -> PresetRules {
        match self {
            Self::Normal => PresetRules::NORMAL,
            Self::Custom(rules) => rules,
            Self::Insane => PresetRules::INSANE,
        }
    }

    pub fn insane_equivalent(self) -> bool {
        self.rules() == PresetRules::INSANE
    }
}

pub struct PresetsPlugin;

impl Plugin for PresetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GamePreset>();
    }
}
