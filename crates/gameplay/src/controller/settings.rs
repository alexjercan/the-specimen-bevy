use bevy::prelude::*;
use bevy_enhanced_input::prelude::Binding;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyBindings {
    pub forward: KeyCode,
    pub left: KeyCode,
    pub backward: KeyCode,
    pub right: KeyCode,
    pub sprint: KeyCode,
    pub interact: KeyCode,
    pub flashlight: Binding,
    pub flashbang: Binding,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct ControllerSettings {
    pub keys: KeyBindings,
    pub mouse_sensitivity: f32,
}
