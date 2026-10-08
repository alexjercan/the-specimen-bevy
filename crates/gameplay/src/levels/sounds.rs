use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplaySoundKind {
    DoorUnlatch,
    DoorSwing,
    DoorShut,
    DoorLocked,
    FuseSlot(usize),
    FuseComplete,
    FlashlightClick,
    LockerOpen,
    LockerClose,
    TableEnter,
    TableLeave,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct GameplaySound {
    pub kind: GameplaySoundKind,
    pub position: Vec3,
}
