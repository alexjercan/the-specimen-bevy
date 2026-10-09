use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use game_settings::{parse_key, GameSettings, GraphicsQuality, MovementKeys, DEFAULT_SENSITIVITY};

#[test]
fn saves_and_loads_player_settings() {
    let folder = std::env::temp_dir().join(format!(
        "horror-settings-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = folder.join("profile/settings.json");
    let mut settings = GameSettings::default();
    settings.master = 0.3;
    settings.sfx = 0.6;
    settings.music = 0.2;
    settings.mouse_sensitivity = 0.004;
    settings.graphics = GraphicsQuality::Low;
    settings.keys.forward = "ArrowUp".into();
    settings.keys.flashlight = "MouseMiddle".into();
    settings.keys.flashbang = "KeyQ".into();
    settings.save(&path).unwrap();
    assert_eq!(GameSettings::load(&path).unwrap(), settings);
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn corrupt_values_cannot_break_input_or_volume() {
    let mut settings: GameSettings = serde_json::from_str(r#"{"master":-3,"sfx":2,"music":0.4,"mouse_sensitivity":-1,"keys":{"forward":"KeyA","left":"KeyA","backward":"KeyS","right":"KeyD","interact":"KeyF"}}"#).unwrap();
    settings.sanitize();
    assert_eq!(settings.master, 0.0);
    assert_eq!(settings.sfx, 1.0);
    assert_eq!(settings.keys, MovementKeys::default());
    settings.keys.forward = "MouseLeft".into();
    settings.sanitize();
    assert_eq!(settings.keys, MovementKeys::default());
    settings.keys.flashbang = "MouseLeft".into();
    settings.sanitize();
    assert_eq!(settings.keys, MovementKeys::default());
    assert_eq!(
        game_settings::parse_binding("MouseRight"),
        Some(game_settings::InputBinding::Mouse(
            bevy::prelude::MouseButton::Right
        ))
    );
    assert_eq!(settings.mouse_sensitivity, game_settings::MIN_SENSITIVITY);
    assert_eq!(parse_key("Escape"), None);
    assert!(DEFAULT_SENSITIVITY > 0.0);
}

#[test]
fn missing_fields_keep_defaults() {
    let mut settings: GameSettings = serde_json::from_str("{\"master\":0.5}").unwrap();
    settings.sanitize();
    assert_eq!(settings.master, 0.5);
    assert_eq!(settings.sfx, 1.0);
    assert_eq!(settings.music, 1.0);
    assert_eq!(settings.keys, MovementKeys::default());
}
