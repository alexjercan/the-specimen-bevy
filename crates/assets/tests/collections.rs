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
