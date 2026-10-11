use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::{KeyCode, MouseButton};
use game_settings::{
    parse_key, DisplayMode, GameSettings, GraphicsQuality, InputBinding, MovementKeys,
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
    settings.custom_difficulty.walk_speed = 1;
    settings.custom_difficulty.run_hearing = 2;
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
    assert_eq!(settings.custom_difficulty.walk_speed, 0);
    assert_eq!(settings.custom_difficulty.chase_speed, 2);
    assert_eq!(settings.custom_difficulty.walk_hearing, 0);
    assert_eq!(settings.custom_difficulty.run_hearing, 1);
    assert!(!settings.custom_difficulty.sprint);
    assert!(settings.custom_difficulty.flashbangs);
}
