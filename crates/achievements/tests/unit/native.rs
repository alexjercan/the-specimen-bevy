use std::{collections::HashSet, fs, time::{SystemTime, UNIX_EPOCH}};

use gameplay::achievements::Achievement;

use super::{load_from_path, save_to_path};

fn unique_dir(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "game-achievements-native-{label}-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ))
}

#[test]
fn round_trip_preserves_unknown_ids() {
    let dir = unique_dir("round-trip");
    let path = dir.join("achievements.json");
    let known = HashSet::from([Achievement::RestoreBoiler]);
    let unknown = HashSet::from(["FUTURE_ACHIEVEMENT".to_string()]);
    save_to_path(&path, &known, &unknown).unwrap();

    let loaded = load_from_path(&path);
    assert_eq!(loaded.known, known);
    assert_eq!(loaded.unknown, unknown);
    assert_eq!(loaded.path, Some(path));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_file_loads_empty_and_stays_writable() {
    let dir = unique_dir("missing");
    let path = dir.join("achievements.json");

    let loaded = load_from_path(&path);
    assert!(loaded.known.is_empty());
    assert!(loaded.unknown.is_empty());
    assert_eq!(loaded.path, Some(path));
}

#[test]
fn corrupt_file_is_backed_up_without_overwriting_an_existing_backup() {
    let dir = unique_dir("corrupt");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("achievements.json");
    fs::write(&path, b"not json").unwrap();
    let existing_backup = dir.join("achievements.json.corrupt");
    fs::write(&existing_backup, b"previous backup").unwrap();

    let loaded = load_from_path(&path);
    assert!(loaded.known.is_empty());
    assert!(loaded.unknown.is_empty());
    assert_eq!(loaded.path, Some(path.clone()));
    assert!(!path.exists());
    assert_eq!(fs::read(&existing_backup).unwrap(), b"previous backup");
    assert_eq!(
        fs::read(dir.join("achievements.json.corrupt.1")).unwrap(),
        b"not json"
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn directory_at_path_disables_saving_without_touching_data() {
    let dir = unique_dir("unreadable");
    let path = dir.join("achievements.json");
    fs::create_dir_all(&path).unwrap();

    let loaded = load_from_path(&path);
    assert!(loaded.known.is_empty());
    assert!(loaded.unknown.is_empty());
    assert_eq!(loaded.path, None);
    assert!(path.is_dir());

    fs::remove_dir_all(dir).unwrap();
}
