use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[require(Transform, Visibility)]
pub enum DevicePlaceholder {
    Flashbang,
    Detector,
}
