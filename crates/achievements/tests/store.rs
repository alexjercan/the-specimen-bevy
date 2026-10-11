use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use game_achievements::{AchievementStore, AchievementStorePlugin};
use gameplay::achievements::{Achievement, AchievementProgress, AchievementUnlocked};

fn unique_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "game-achievements-store-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn file_app(path: &std::path::Path) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AchievementStorePlugin {
            store: AchievementStore::File(path.to_path_buf()),
        },
    ));
    app
}

fn unlock(app: &mut App, achievement: Achievement) {
    app.world_mut()
        .resource_mut::<AchievementProgress>()
        .unlocked
        .insert(achievement);
    app.world_mut().trigger(AchievementUnlocked(achievement));
}

fn unlocked_ids(path: &std::path::Path) -> Vec<String> {
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(saved["version"], 1);
    saved["unlocked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn missing_file_loads_empty_and_is_created_on_first_unlock() {
    let dir = unique_dir("missing");
    let path = dir.join("achievements.json");

    let mut app = file_app(&path);
    assert!(app
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .is_empty());
    assert!(!path.exists());

    unlock(&mut app, Achievement::RestoreBoiler);
    app.update();
    assert_eq!(unlocked_ids(&path), ["RESTORE_BOILER"]);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn corrupt_file_is_backed_up_without_overwriting_an_existing_backup() {
    let dir = unique_dir("corrupt");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("achievements.json");
    fs::write(&path, b"not json").unwrap();
    let existing_backup = dir.join("achievements.json.corrupt");
    fs::write(&existing_backup, b"previous backup").unwrap();

    let mut app = file_app(&path);
    assert!(app
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .is_empty());
    assert!(!path.exists());
    assert_eq!(fs::read(&existing_backup).unwrap(), b"previous backup");
    assert_eq!(
        fs::read(dir.join("achievements.json.corrupt.1")).unwrap(),
        b"not json"
    );

    unlock(&mut app, Achievement::RestoreBoiler);
    app.update();
    assert_eq!(unlocked_ids(&path), ["RESTORE_BOILER"]);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn directory_at_path_disables_saving_without_touching_data() {
    let dir = unique_dir("unreadable");
    let path = dir.join("achievements.json");
    fs::create_dir_all(&path).unwrap();

    let mut app = file_app(&path);
    assert!(app
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .is_empty());

    unlock(&mut app, Achievement::RestoreBoiler);
    app.update();
    assert!(path.is_dir());
    assert!(fs::read_dir(&path).unwrap().next().is_none());
    assert!(!dir.join("achievements.json.tmp").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn store_accepts_any_version_keeps_unknown_ids_and_saves_sorted() {
    let dir = unique_dir("format");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("achievements.json");
    fs::write(
        &path,
        br#"{"version":99,"unlocked":["ZZZ_FUTURE","RESTORE_BOILER","AAA_FUTURE"]}"#,
    )
    .unwrap();

    let mut app = file_app(&path);
    assert_eq!(
        app.world().resource::<AchievementProgress>().unlocked,
        [Achievement::RestoreBoiler].into_iter().collect()
    );

    unlock(&mut app, Achievement::EscapeUndetected);
    app.update();
    assert_eq!(
        unlocked_ids(&path),
        [
            "AAA_FUTURE",
            "ESCAPE_UNDETECTED",
            "RESTORE_BOILER",
            "ZZZ_FUTURE"
        ]
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn hidden_insane_escape_persists_and_restores_the_insane_gate() {
    let dir = unique_dir("insane");
    let path = dir.join("achievements.json");

    let mut app = file_app(&path);
    for achievement in Achievement::ALL {
        unlock(&mut app, achievement);
    }
    app.update();
    assert!(unlocked_ids(&path).contains(&"ESCAPE_INSANE".to_string()));

    let restarted = file_app(&path);
    let progress = restarted.world().resource::<AchievementProgress>();
    assert!(progress.unlocked.contains(&Achievement::EscapeInsane));
    assert!(progress.insane_unlocked());

    fs::remove_dir_all(dir).unwrap();
}
