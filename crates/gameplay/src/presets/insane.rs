use super::{
    GameMode, GamePreset, MonsterChaseSpeed, MonsterPreset, MonsterRunHearing, MonsterWalkHearing,
    MonsterWalkSpeed,
};

pub(super) const PRESET: GamePreset = GamePreset {
    mode: GameMode::Insane,
    monster: MonsterPreset {
        walk_speed: MonsterWalkSpeed::Fast,
        chase_speed: MonsterChaseSpeed::Fast,
        walk_hearing: MonsterWalkHearing::Long,
        run_hearing: MonsterRunHearing::Long,
    },
    sprint: false,
    flashbangs: false,
    detector: false,
    blackout: true,
};
