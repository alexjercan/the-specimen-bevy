#![allow(dead_code)]

use std::time::Duration;

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::{
    fuses::FuseSeed,
    player::{ControllerSettings, Input, KeyBindings, PlayerPlugin},
    presets::GamePreset,
};

pub const SEED: u64 = 7;

pub fn controller_settings() -> ControllerSettings {
    ControllerSettings {
        keys: KeyBindings {
            forward: KeyCode::KeyW,
            left: KeyCode::KeyA,
            backward: KeyCode::KeyS,
            right: KeyCode::KeyD,
            sprint: KeyCode::ShiftLeft,
            interact: KeyCode::KeyF,
            flashlight: Input::Mouse(MouseButton::Left),
            flashbang: Input::Mouse(MouseButton::Right),
        },
        mouse_sensitivity: 0.0015,
    }
}

pub fn level_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(FuseSeed(SEED))
        .insert_resource(GamePreset::default());
    app
}

pub fn player_app(frame: Duration) -> App {
    let mut app = level_app();
    app.add_plugins((InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(frame))
        .insert_resource(controller_settings())
        .add_plugins(PlayerPlugin);
    app
}
