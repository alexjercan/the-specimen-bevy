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
fn small_pumpkin_is_unlit_and_light_sources_are_explicit() {
    let halloween = manifest("halloween");
    let small = &halloween["modules"]["pumpkin_small"];
    let jack = &halloween["modules"]["jack_o_lantern"];
    for edge in ["min", "max"] {
        let small_bounds = bounds(small, edge);
        let jack_bounds = bounds(jack, edge);
        for axis in [0, 1] {
            assert!(
                (small_bounds[axis] - jack_bounds[axis]).abs() < 1e-4,
                "pumpkins differ in {edge} axis {axis}"
            );
        }
        assert!(
            (small_bounds[2] - jack_bounds[2]).abs() < 0.03,
            "only the jack-o'-lantern face may protrude beyond the body"
        );
    }
    assert!(small.get("light_anchor_bevy").is_none());
    let materials = small["materials"].as_array().unwrap();
    assert!(!materials.iter().any(|material| material == "pumpkin_ember"));
    for name in ["jack_o_lantern", "lantern", "candle_cluster"] {
        assert!(
            halloween["modules"][name]["light_anchor_bevy"].is_array(),
            "{name}"
        );
    }
}

#[test]
fn decorations_rest_on_their_prop_surfaces() {
    let facility = manifest("facility/modules");
    let halloween = manifest("halloween");
    for decor in game_halloween::DECOR {
        if decor.prop == "ceiling_light_amber" {
            assert_eq!(decor.anchor, bevy::prelude::Vec2::new(0.0, -2.5));
            assert!(decor.offset.y >= 1.8 && decor.offset.y <= 2.2);
            continue;
        }
        if decor.prop == "potted_plant" {
            assert_eq!(decor.offset, bevy::prelude::Vec3::ZERO);
            continue;
        }
        if decor.prop == "clutter_tools" {
            assert_eq!(decor.offset.y, 0.0);
            assert!(decor.offset.x.abs() >= 0.8);
            continue;
        }
        let prop = &facility["modules"][decor.prop];
        if prop["snap"] == "wall_mount" {
            assert!(decor.offset.y.abs() <= 0.17);
            assert!(decor.offset.z <= 0.0);
            continue;
        }
        let top = if decor.prop == "workbench" {
            0.9
        } else {
            bounds(prop, "max")[1] as f32
        };
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
        assert!(
            decor.offset.x - half_width >= prop_min[0] as f32 - 0.02,
            "{} on {} at {}",
            decor.module,
            decor.prop,
            decor.anchor
        );
        assert!(
            decor.offset.x + half_width <= prop_max[0] as f32 + 0.02,
            "{} on {} at {}",
            decor.module,
            decor.prop,
            decor.anchor
        );
        assert!(
            decor.offset.z.abs() + half_width <= prop_max[2] as f32 + 0.02,
            "{} on {} at {}",
            decor.module,
            decor.prop,
            decor.anchor
        );
    }
}
