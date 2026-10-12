use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::{KeyCode, MouseButton};
use game_settings::{
    parse_key, CustomLevel, DisplayMode, GameSettings, GraphicsQuality, InputBinding, MovementKeys,
};

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
    let mut settings = GameSettings {
        master: 0.3,
        sfx: 0.6,
        music: 0.2,
        mouse_sensitivity: 0.004,
        graphics: GraphicsQuality::Low,
        display_mode: DisplayMode::Windowed,
        fps_overlay: false,
        preferred_mode: game_settings::PreferredMode::Custom,
        ..Default::default()
    };
    settings.custom_difficulty.walk_speed = CustomLevel::Medium;
    settings.custom_difficulty.run_hearing = CustomLevel::High;
    settings.custom_difficulty.sprint = false;
    settings.keys.forward = KeyCode::ArrowUp;
    settings.keys.flashlight = MouseButton::Middle.into();
    settings.keys.flashbang = KeyCode::KeyQ.into();
    settings.save(&path).unwrap();
    assert_eq!(GameSettings::load(&path).unwrap(), settings);
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn old_settings_are_copied_only_when_new_settings_are_missing() {
    let folder = std::env::temp_dir().join(format!(
        "specimen-settings-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let old_path = folder.join("horror-game-bevy/settings.json");
    let new_path = folder.join("the-specimen-bevy/settings.json");
    let old = GameSettings {
        master: 0.3,
        ..Default::default()
    };
    old.save(&old_path).unwrap();
    let loaded = game_settings::load_or_migrate(&new_path).unwrap();
    assert_eq!(loaded, old);
    assert_eq!(GameSettings::load(&old_path).unwrap(), old);
    assert_eq!(GameSettings::load(&new_path).unwrap(), old);
    let mut updated = old.clone();
    updated.master = 0.8;
    updated.save(&new_path).unwrap();
    assert_eq!(game_settings::load_or_migrate(&new_path).unwrap(), updated);
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn corrupt_values_cannot_break_input_or_volume() {
    let mut settings: GameSettings = serde_json::from_str(r#"{"master":-3,"sfx":2,"music":0.4,"mouse_sensitivity":-1,"keys":{"forward":"KeyA","left":"KeyA","backward":"KeyS","right":"KeyD","interact":"KeyF"}}"#).unwrap();
    settings.sanitize();
    assert_eq!(settings.master, 0.0);
    assert_eq!(settings.sfx, 1.0);
    assert_eq!(settings.keys, MovementKeys::default());
    settings.keys.flashbang = MouseButton::Left.into();
    settings.sanitize();
    assert_eq!(settings.keys, MovementKeys::default());
    assert_eq!(
        InputBinding::parse("MouseRight"),
        Some(InputBinding::Mouse(MouseButton::Right))
    );
    assert_eq!(InputBinding::parse("Escape"), None);
    assert_eq!(
        InputBinding::parse("KeyQ"),
        Some(InputBinding::Key(KeyCode::KeyQ))
    );
    assert_eq!(settings.mouse_sensitivity, game_settings::MIN_SENSITIVITY);
    assert_eq!(parse_key("Escape"), None);
}

#[test]
fn missing_fields_keep_defaults() {
    let mut settings: GameSettings = serde_json::from_str("{\"master\":0.5}").unwrap();
    settings.sanitize();
    assert_eq!(settings.master, 0.5);
    assert_eq!(settings.sfx, 1.0);
    assert_eq!(settings.music, 1.0);
    assert_eq!(settings.keys, MovementKeys::default());
    assert_eq!(settings.display_mode, DisplayMode::Fullscreen);
    assert!(!settings.fps_overlay);
    assert_eq!(
        settings.preferred_mode,
        game_settings::PreferredMode::Normal
    );
    assert_eq!(
        settings.custom_difficulty,
        game_settings::CustomDifficulty::default()
    );
}

#[test]
fn invalid_custom_options_reset_to_normal_choices() {
    let mut settings: GameSettings = serde_json::from_str(
        r#"{"preferred_mode":"custom","custom_difficulty":{"walk_speed":255,"chase_speed":2,"walk_hearing":3,"run_hearing":1,"sprint":false}}"#,
    ).unwrap();
    settings.sanitize();
    assert_eq!(
        settings.preferred_mode,
        game_settings::PreferredMode::Custom
    );
    assert_eq!(settings.custom_difficulty.walk_speed, CustomLevel::Low);
    assert_eq!(settings.custom_difficulty.chase_speed, CustomLevel::High);
    assert_eq!(settings.custom_difficulty.walk_hearing, CustomLevel::Low);
    assert_eq!(settings.custom_difficulty.run_hearing, CustomLevel::Medium);
    assert!(!settings.custom_difficulty.sprint);
    assert!(settings.custom_difficulty.flashbangs);
}

// NOTE: written by the release before custom levels became an enum; the numbers must keep loading.
const SAVED_BEFORE_CUSTOM_LEVELS: &str = r#"{
  "master": 0.8,
  "sfx": 0.5,
  "music": 0.25,
  "mouse_sensitivity": 0.002,
  "keys": {
    "forward": "KeyW",
    "left": "KeyA",
    "backward": "KeyS",
    "right": "KeyD",
    "interact": "KeyE",
    "sprint": "ShiftLeft",
    "flashlight": "MouseLeft",
    "flashbang": "KeyQ"
  },
  "graphics": "medium",
  "display_mode": "windowed",
  "fps_overlay": true,
  "preferred_mode": "custom",
  "custom_difficulty": {
    "walk_speed": 2,
    "chase_speed": 1,
    "walk_hearing": 0,
    "run_hearing": 2,
    "sprint": false,
    "flashbangs": true,
    "detector": false,
    "blackout": true
  }
}"#;

#[test]
fn settings_saved_before_custom_levels_still_load_and_save_the_same_numbers() {
    let folder = std::env::temp_dir().join(format!(
        "horror-settings-old-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = folder.join("settings.json");
    fs::create_dir_all(&folder).unwrap();
    fs::write(&path, SAVED_BEFORE_CUSTOM_LEVELS).unwrap();

    let settings = GameSettings::load(&path).unwrap();
    let custom = settings.custom_difficulty;
    assert_eq!(
        (
            custom.walk_speed,
            custom.chase_speed,
            custom.walk_hearing,
            custom.run_hearing
        ),
        (
            CustomLevel::High,
            CustomLevel::Medium,
            CustomLevel::Low,
            CustomLevel::High
        )
    );
    assert!(!custom.sprint && custom.flashbangs && !custom.detector && custom.blackout);
    assert_eq!(settings.keys.interact, KeyCode::KeyE);
    assert_eq!(settings.graphics, GraphicsQuality::Medium);

    settings.save(&path).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let old: serde_json::Value = serde_json::from_str(SAVED_BEFORE_CUSTOM_LEVELS).unwrap();
    assert_eq!(saved, old);
    fs::remove_dir_all(folder).unwrap();
}
