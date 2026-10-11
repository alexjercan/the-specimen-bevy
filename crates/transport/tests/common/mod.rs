#![allow(dead_code)]

use bevy::prelude::*;
use gameplay::controller::{ControllerSettings, KeyBindings};

pub fn controller_settings() -> ControllerSettings {
    ControllerSettings {
        keys: KeyBindings {
            forward: KeyCode::KeyW,
            left: KeyCode::KeyA,
            backward: KeyCode::KeyS,
            right: KeyCode::KeyD,
            sprint: KeyCode::ShiftLeft,
            interact: KeyCode::KeyF,
            flashlight: MouseButton::Left.into(),
            flashbang: MouseButton::Right.into(),
        },
        mouse_sensitivity: 0.0015,
    }
}
