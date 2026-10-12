mod logic;
mod sounds;

pub use logic::{
    Hidden, HidingCue, HidingCueKind, HidingMotion, HidingPhase, HidingPlugin, HidingSpot,
    UseHidingSpot, HIDING_TRANSITION,
};
pub use sounds::{HidingSounds, HidingSoundsPlugin, LOCKER_CUE_OFFSET, TABLE_CUE_OFFSET};
