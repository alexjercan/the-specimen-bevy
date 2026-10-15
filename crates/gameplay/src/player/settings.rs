use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl From<KeyCode> for Input {
    fn from(key: KeyCode) -> Self {
        Self::Key(key)
    }
}

impl From<MouseButton> for Input {
    fn from(button: MouseButton) -> Self {
        Self::Mouse(button)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBindings {
    pub forward: KeyCode,
    pub left: KeyCode,
    pub backward: KeyCode,
    pub right: KeyCode,
    pub sprint: KeyCode,
    pub interact: KeyCode,
    pub flashlight: Input,
    pub flashbang: Input,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct ControllerSettings {
    pub keys: KeyBindings,
    pub mouse_sensitivity: f32,
    pub invert_y: bool,
}
