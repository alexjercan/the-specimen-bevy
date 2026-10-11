use std::{fs, path::Path};

use game_halloween::{CANDY_FUSE, HALLOWEEN_MODULES};

fn manifest(dir: &str) -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .join(dir)
        .join("modules.manifest.json");
    serde_json::from_slice(&fs::read(&path).expect("promoted manifest")).expect("valid manifest")
}

fn bounds(entry: &serde_json::Value, key: &str) -> Vec<f64> {
    entry["bounds_bevy"][key]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap())
        .collect()
}

#[test]
fn declared_halloween_scenes_match_promoted_manifest() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/halloween");
    let manifest = manifest("halloween");
    let mut names: Vec<_> = manifest["modules"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    let mut declared = HALLOWEEN_MODULES.to_vec();
    declared.sort_unstable();
    assert_eq!(names, declared);
    let source = include_str!("../src/lib.rs");
    for name in HALLOWEEN_MODULES {
        assert!(root.join(format!("{name}.glb")).is_file(), "{name}");
        assert!(
            source.contains(&format!("\"halloween/{name}.glb#Scene0\"")),
            "{name}"
        );
    }
}

#[test]
fn candy_fuse_fits_inside_the_base_fuse_outline() {
    let facility = manifest("facility/modules");
    let halloween = manifest("halloween");
    let fuse = &facility["modules"]["fuse_pickup"];
    let candy = &halloween["modules"][CANDY_FUSE];
    for (axis, (lo, hi)) in bounds(candy, "min")
        .into_iter()
        .zip(bounds(candy, "max"))
        .enumerate()
    {
        assert!(lo >= bounds(fuse, "min")[axis] - 1e-4, "axis {axis}");
        assert!(hi <= bounds(fuse, "max")[axis] + 1e-4, "axis {axis}");
    }
}

#[test]
fn decorations_rest_on_their_prop_surfaces() {
    let facility = manifest("facility/modules");
    let halloween = manifest("halloween");
    for decor in game_halloween::DECOR {
        let prop = &facility["modules"][decor.prop];
        let top = bounds(prop, "max")[1] as f32;
        let module = &halloween["modules"][decor.module];
        let base = bounds(module, "min")[1] as f32;
        assert!(base.abs() < 1e-3, "{} base", decor.module);
        assert!(
            (decor.offset.y - top).abs() < 0.03,
            "{} on {}",
            decor.module,
            decor.prop
        );
        let half_width = (bounds(module, "max")[0] - bounds(module, "min")[0]) as f32 / 2.0;
        let prop_min = bounds(prop, "min");
        let prop_max = bounds(prop, "max");
        assert!(decor.offset.x - half_width >= prop_min[0] as f32 - 0.02);
        assert!(decor.offset.x + half_width <= prop_max[0] as f32 + 0.02);
        assert!(decor.offset.z.abs() + half_width <= prop_max[2] as f32 + 0.02);
    }
}
