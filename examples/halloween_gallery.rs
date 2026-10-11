use bevy::prelude::*;
use game_halloween::{
    candle_light, spawn_decor, Decor, MonsterEyesPlugin, CANDLE_CLUSTER, CANDY_FUSE, DECOR,
    JACK_O_LANTERN, LANTERN, PUMPKIN_SMALL, SIGN_TRIM, SPECIMEN_JAR, SPIDER, SPIDER_WEB,
};
use gameplay::levels::FUSE_MODULE;

#[path = "gallery/mod.rs"]
mod gallery;

use gallery::{part, Exhibit, Gallery, GalleryPlugin};

const MONSTER_SCALE: f32 = 0.42;
const MONSTER_LIFT: f32 = 1.93;
const MONSTER_EYES: Vec3 = Vec3::new(-0.018, 0.073, -0.111);

const EXHIBITS: &[Exhibit] = &[
    Exhibit {
        group: "Fuses",
        title: "Candy fuse",
        parts: &[part(CANDY_FUSE, 0.0, 0.5, 0.0, 0.0, 1.0).in_dir("halloween")],
        radius: 0.6,
    },
    Exhibit {
        group: "Fuses",
        title: "Fuse and candy fuse",
        parts: &[
            part(FUSE_MODULE, 0.0, 0.5, -0.06, 0.0, 1.0),
            part(CANDY_FUSE, 0.0, 0.5, 0.06, 0.0, 1.0).in_dir("halloween"),
        ],
        radius: 0.7,
    },
    Exhibit {
        group: "Fuses",
        title: "Workbench + candy fuse",
        parts: &[
            part("workbench", 0.0, 0.0, 0.0, 0.0, 1.0),
            part(CANDY_FUSE, 0.25, 0.9, -0.16, 0.0, 1.0).in_dir("halloween"),
        ],
        radius: 3.2,
    },
    Exhibit {
        group: "Pumpkins",
        title: "Jack-o'-lantern",
        parts: &[part(JACK_O_LANTERN, 0.0, 0.0, 0.0, 0.0, 1.0).in_dir("halloween")],
        radius: 1.0,
    },
    Exhibit {
        group: "Pumpkins",
        title: "Jack-o'-lantern on drum",
        parts: &[part("steel_drum", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.0,
    },
    Exhibit {
        group: "Pumpkins",
        title: "Uncarved pumpkin",
        parts: &[part(PUMPKIN_SMALL, 0.0, 0.0, 0.0, 0.0, 1.0).in_dir("halloween")],
        radius: 1.2,
    },
    Exhibit {
        group: "Pumpkins",
        title: "Small pumpkin on crate",
        parts: &[part("storage_crate", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.0,
    },
    Exhibit {
        group: "Tabletop",
        title: "Specimen jar",
        parts: &[part(SPECIMEN_JAR, 0.0, 0.0, 0.0, 0.0, 1.0).in_dir("halloween")],
        radius: 0.9,
    },
    Exhibit {
        group: "Tabletop",
        title: "Lantern",
        parts: &[part(LANTERN, 0.0, 0.0, 0.0, 0.0, 1.0).in_dir("halloween")],
        radius: 0.9,
    },
    Exhibit {
        group: "Tabletop",
        title: "Candles",
        parts: &[part(CANDLE_CLUSTER, 0.0, 0.0, 0.0, 0.0, 1.0).in_dir("halloween")],
        radius: 1.0,
    },
    Exhibit {
        group: "Wall decor",
        title: "Spider web",
        parts: &[part(SPIDER_WEB, 0.0, 0.5, 0.0, 0.0, 1.0).in_dir("halloween")],
        radius: 0.9,
    },
    Exhibit {
        group: "Wall decor",
        title: "Spider",
        parts: &[part(SPIDER, 0.0, 0.5, 0.0, 0.0, 1.0).in_dir("halloween")],
        radius: 0.7,
    },
    Exhibit {
        group: "Wall decor",
        title: "Room sign trim",
        parts: &[part(SIGN_TRIM, 0.0, 0.5, 0.0, 0.0, 1.0).in_dir("halloween")],
        radius: 1.7,
    },
    Exhibit {
        group: "Compositions",
        title: "Lab sign with web and spider",
        parts: &[
            part("sign_label_lab", 0.0, 1.5, 0.0, 0.0, 1.0),
            part(SIGN_TRIM, 0.0, 1.5, 0.0, 0.0, 1.0).in_dir("halloween"),
            part(SPIDER_WEB, -0.55, 1.34, -0.045, 0.0, 1.0).in_dir("halloween"),
            part(SPIDER, -0.55, 1.33, -0.05, 0.0, 1.0).in_dir("halloween"),
        ],
        radius: 2.3,
    },
    Exhibit {
        group: "Compositions",
        title: "Table with jar and candles",
        parts: &[
            part("concept_table", 0.0, 0.0, 0.0, 0.0, 1.0),
            part(SPECIMEN_JAR, 0.4, 0.8, -0.1, 0.0, 1.0).in_dir("halloween"),
            part(CANDLE_CLUSTER, -0.35, 0.8, 0.0, 0.0, 1.0).in_dir("halloween"),
        ],
        radius: 3.0,
    },
    Exhibit {
        group: "Monster",
        title: "Monster eyes",
        parts: &[part(
            "human_deer_animated",
            -MONSTER_EYES.x * MONSTER_SCALE,
            MONSTER_LIFT,
            -MONSTER_EYES.z * MONSTER_SCALE,
            0.0,
            MONSTER_SCALE,
        )
        .in_dir("monster")],
        radius: 1.6,
    },
];

const GALLERY: Gallery = Gallery {
    exhibits: EXHIBITS,
    env_var: "HALLOWEEN_GALLERY_EXHIBIT",
    focus_height,
    spawn_extra: Some(spawn_extra),
    min_radius: 0.4,
};

fn main() {
    App::new()
        .add_plugins((GalleryPlugin(GALLERY), MonsterEyesPlugin))
        .run();
}

fn focus_height(exhibit: &Exhibit) -> f32 {
    match exhibit.title {
        "Monster eyes" => MONSTER_LIFT + MONSTER_EYES.y * MONSTER_SCALE,
        "Candy fuse" | "Fuse and candy fuse" => 0.53,
        "Jack-o'-lantern" | "Uncarved pumpkin" => 0.2,
        "Specimen jar" | "Lantern" | "Candles" => 0.18,
        "Spider web" | "Spider" | "Room sign trim" => 0.5,
        "Lab sign with web and spider" => 1.5,
        _ => 0.8,
    }
}

fn spawn_extra(
    commands: &mut Commands,
    root: Entity,
    entry: &Exhibit,
    assets: &AssetServer,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<StandardMaterial>,
) {
    match entry.title {
        "Jack-o'-lantern" => {
            commands.entity(root).with_child(candle_light());
        }
        "Lantern" | "Candles" | "Table with jar and candles" => {
            let position = match entry.title {
                "Lantern" => Vec3::new(0.0, 0.18, 0.0),
                "Table with jar and candles" => Vec3::new(-0.35, 1.01, 0.0),
                _ => Vec3::new(0.0, 0.21, 0.0),
            };
            commands.entity(root).with_child((
                PointLight {
                    color: Color::srgb(1.0, 0.45, 0.1),
                    intensity: 2_000.0,
                    range: 2.5,
                    ..default()
                },
                Transform::from_translation(position),
            ));
        }
        "Jack-o'-lantern on drum" => {
            place(commands, root, assets, &DECOR[0]);
        }
        "Small pumpkin on crate" => {
            let decor = DECOR
                .iter()
                .find(|decor| decor.prop == "storage_crate" && decor.module == PUMPKIN_SMALL)
                .expect("storage crate pumpkin decor entry");
            place(commands, root, assets, decor);
        }
        _ => {}
    }
}

fn place(commands: &mut Commands, root: Entity, assets: &AssetServer, decor: &Decor) {
    let module = assets.load(format!("halloween/{}.glb#Scene0", decor.module));
    spawn_decor(commands, root, Quat::IDENTITY, decor, module);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn gallery_covers_halloween_modules_and_browses_both_ways() {
        for entry in EXHIBITS {
            for part in entry.parts {
                let root =
                    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("assets/{}", part.dir));
                assert!(
                    root.join(format!("{}.glb", part.module)).is_file(),
                    "{}",
                    part.module
                );
            }
        }
        for module in game_halloween::HALLOWEEN_MODULES {
            assert!(
                EXHIBITS
                    .iter()
                    .any(|entry| entry.parts.iter().any(|part| part.module == module)),
                "{module}"
            );
        }
        assert_eq!(gallery::cycle(EXHIBITS.len(), 0, -1), EXHIBITS.len() - 1);
        assert_eq!(gallery::cycle(EXHIBITS.len(), EXHIBITS.len() - 1, 1), 0);
    }
}
