use super::{
    MonsterChaseSpeed, MonsterPreset, MonsterRunHearing, MonsterWalkHearing, MonsterWalkSpeed,
    PresetRules,
};

impl PresetRules {
    pub const INSANE: Self = Self {
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
}
