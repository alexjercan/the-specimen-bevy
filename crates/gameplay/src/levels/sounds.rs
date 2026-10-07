use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplaySoundKind {
    DoorUnlatch,
    DoorSwing,
    DoorShut,
    FusePickup,
    PanelInstall,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct GameplaySound {
    pub kind: GameplaySoundKind,
    pub position: Vec3,
}
