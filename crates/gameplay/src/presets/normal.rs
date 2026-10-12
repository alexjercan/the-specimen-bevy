use super::{
    MonsterChaseSpeed, MonsterPreset, MonsterRunHearing, MonsterWalkHearing, MonsterWalkSpeed,
    PresetRules,
};

impl PresetRules {
    pub const NORMAL: Self = Self {
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
}
