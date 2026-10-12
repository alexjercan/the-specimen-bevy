mod assets;
mod logic;
mod sounds;
mod view;

pub use assets::MonsterAssets;
pub(crate) use logic::{catch_player, sees_player, WitnessedHiding};
pub use logic::{
    Caught, Monster, MonsterCue, MonsterCueKind, MonsterFigure, MonsterNavigation, MonsterPlugin,
    PatrolRng, MONSTER_HEAD_JOINT,
};
pub use sounds::{MonsterSounds, MonsterSoundsPlugin, MONSTER_CUE_OFFSET};
pub use view::MonsterViewPlugin;
