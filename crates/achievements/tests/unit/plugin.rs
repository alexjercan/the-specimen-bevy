use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use gameplay::achievements::{Achievement, AchievementProgress, AchievementUnlocked};

use super::install_native_for_test;

#[test]
fn unlock_event_is_saved_and_loaded_on_next_launch() {
    let dir = std::env::temp_dir().join(format!(
        "game-achievements-plugin-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = dir.join("achievements.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    install_native_for_test(&mut app, path.clone());
    app.world_mut()
        .resource_mut::<AchievementProgress>()
        .unlocked
        .insert(Achievement::RestoreBoiler);
    app.world_mut()
        .trigger(AchievementUnlocked(Achievement::RestoreBoiler));
    app.update();
    assert!(path.is_file());

    let mut second = App::new();
    second.add_plugins(MinimalPlugins);
    install_native_for_test(&mut second, path);
    assert!(second
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .contains(&Achievement::RestoreBoiler));
    std::fs::remove_dir_all(dir).unwrap();
}
