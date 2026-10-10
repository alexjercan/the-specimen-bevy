use super::{
    GameMode, GamePreset, MonsterChaseSpeed, MonsterPreset, MonsterRunHearing, MonsterWalkHearing,
    MonsterWalkSpeed,
};

pub(super) const PRESET: GamePreset = GamePreset {
    mode: GameMode::Normal,
    monster: MonsterPreset {
        walk_speed: MonsterWalkSpeed::Slow,
        chase_speed: MonsterChaseSpeed::Normal,
        walk_hearing: MonsterWalkHearing::Short,
        run_hearing: MonsterRunHearing::Short,
    },
    sprint: true,
    flashbangs: true,
    detector: true,
    blackout: false,
};
