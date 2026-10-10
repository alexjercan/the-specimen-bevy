use std::{
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use game_achievements::{install_native_for_test, AchievementStorePlugin};
use gameplay::achievements::{Achievement, AchievementProgress, AchievementUnlocked};

fn unique_path(label: &str) -> std::path::PathBuf {
    env::temp_dir()
        .join(format!(
            "game-achievements-plugin-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
        .join("achievements.json")
}

fn unlock(app: &mut App, achievement: Achievement) {
    app.world_mut()
        .get_resource_or_init::<AchievementProgress>()
        .unlocked
        .insert(achievement);
    app.world_mut().trigger(AchievementUnlocked(achievement));
}

#[test]
fn persisted_store_merges_saves_and_reloads_across_a_simulated_restart() {
    let path = unique_path("round-trip");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        br#"{"version":1,"unlocked":["RESTORE_BOILER","FUTURE_ACHIEVEMENT"]}"#,
    )
    .unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    install_native_for_test(&mut app, path.clone());

    assert!(app
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .contains(&Achievement::RestoreBoiler));

    unlock(&mut app, Achievement::EscapeUndetected);
    app.update();

    let saved = fs::read_to_string(&path).unwrap();
    assert!(saved.contains("RESTORE_BOILER"));
    assert!(saved.contains("FUTURE_ACHIEVEMENT"));
    assert!(saved.contains("ESCAPE_UNDETECTED"));

    let mut restarted = App::new();
    restarted.add_plugins(MinimalPlugins);
    install_native_for_test(&mut restarted, path.clone());
    let progress = restarted.world().resource::<AchievementProgress>();
    assert!(progress.unlocked.contains(&Achievement::RestoreBoiler));
    assert!(progress.unlocked.contains(&Achievement::EscapeUndetected));

    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn persist_false_adds_nothing_and_never_touches_the_default_path() {
    let dir = env::temp_dir().join(format!(
        "game-achievements-plugin-no-persist-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let previous = env::var_os("XDG_DATA_HOME");
    env::set_var("XDG_DATA_HOME", &dir);

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AchievementStorePlugin { persist: false }));
    unlock(&mut app, Achievement::RestoreBoiler);
    app.update();

    let expected_path = dir.join("the-specimen-bevy/achievements.json");
    assert!(!expected_path.exists());

    match previous {
        Some(value) => env::set_var("XDG_DATA_HOME", value),
        None => env::remove_var("XDG_DATA_HOME"),
    }
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
}
