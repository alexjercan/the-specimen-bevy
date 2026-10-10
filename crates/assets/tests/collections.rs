use std::{collections::BTreeSet, fs, path::Path};

#[test]
fn declared_scenes_match_promoted_manifest() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let modules = root.join("assets/facility/modules");
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(modules.join("modules.manifest.json")).expect("promoted module manifest"),
    )
    .expect("valid module manifest");
    let declared: Vec<_> = include_str!("../src/lib.rs")
        .lines()
        .filter_map(|line| {
            line.trim()
                .trim_end_matches(',')
                .strip_prefix("\"facility/modules/")
                .and_then(|path| path.strip_suffix(".glb#Scene0\""))
        })
        .collect();
    let names: BTreeSet<_> = declared.iter().copied().collect();
    let manifest_names: BTreeSet<_> = manifest["modules"]
        .as_object()
        .expect("manifest modules")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(declared.len(), names.len(), "duplicate loader scene path");
    assert_eq!(names, manifest_names, "loader paths must match manifest");
    for name in names {
        assert!(modules.join(format!("{name}.glb")).is_file());
    }
}

#[test]
fn declared_achievement_icons_exist_in_locked_and_unlocked_pairs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let declared: Vec<_> = include_str!("../src/lib.rs")
        .lines()
        .filter_map(|line| {
            line.trim()
                .trim_end_matches(',')
                .strip_prefix("\"ui/achievements/")
                .and_then(|path| path.strip_suffix(".png\""))
        })
        .collect();
    let unique: BTreeSet<_> = declared.iter().copied().collect();
    assert_eq!(declared.len(), 14);
    assert_eq!(unique.len(), 14, "duplicate achievement icon path");
    for name in unique {
        assert!(root.join(format!("ui/achievements/{name}.png")).is_file());
        let stem = name
            .strip_suffix("-locked")
            .or_else(|| name.strip_suffix("-unlocked"))
            .expect("icon variant suffix");
        assert!(declared.contains(&format!("{stem}-locked").as_str()));
        assert!(declared.contains(&format!("{stem}-unlocked").as_str()));
    }
}

#[test]
fn declared_key_glyphs_match_mapping_and_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let declared: Vec<_> = include_str!("../src/lib.rs")
        .lines()
        .filter_map(|line| {
            line.trim()
                .trim_end_matches(',')
                .strip_prefix("\"ui/input-prompts/")
                .and_then(|path| path.strip_suffix(".png\""))
        })
        .collect();
    let names: BTreeSet<_> = declared.iter().copied().collect();
    let mapped: BTreeSet<_> = game_assets::KEY_GLYPHS
        .iter()
        .map(|(_, stem)| *stem)
        .collect();
    let keys: BTreeSet<_> = game_assets::KEY_GLYPHS
        .iter()
        .map(|(key, _)| *key)
        .collect();
    assert_eq!(declared.len(), names.len(), "duplicate key glyph path");
    assert_eq!(keys.len(), game_assets::KEY_GLYPHS.len(), "duplicate key");
    assert_eq!(names, mapped, "loader paths must match key glyph mapping");
    for name in names {
        assert!(root.join(format!("ui/input-prompts/{name}.png")).is_file());
    }
    assert_eq!(game_assets::key_glyph_stem("Digit4"), Some("T_3_Key_Alt-1"));
    assert_eq!(game_assets::key_glyph_stem("Equal"), None);
}
