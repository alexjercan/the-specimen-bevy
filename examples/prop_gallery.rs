use bevy::{prelude::*, render::render_resource::Face};

#[path = "gallery/mod.rs"]
mod gallery;

use gallery::{part, wall_part, Exhibit, Gallery, GalleryPlugin};

const EXHIBITS: &[Exhibit] = &[
    Exhibit {
        group: "Tables",
        title: "Workbench + fuse",
        parts: &[
            part("workbench", 0.0, 0.0, 0.0, 0.0, 1.0),
            part("fuse_pickup", 0.25, 0.9, -0.16, 0.0, 1.0),
        ],
        radius: 3.2,
    },
    Exhibit {
        group: "Tables",
        title: "Hiding table + papers + toolbox",
        parts: &[
            part("concept_table", 0.0, 0.0, 0.0, 0.0, 1.0),
            part("clutter_papers", -0.25, 0.8, 0.0, 0.2, 0.6),
            part("toolbox", 0.46, 0.8, -0.14, 0.0, 1.0),
        ],
        radius: 3.2,
    },
    Exhibit {
        group: "Tables",
        title: "Work island",
        parts: &[part("work_island", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.5,
    },
    Exhibit {
        group: "Storage",
        title: "Shelf",
        parts: &[part("shelf_unit", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.8,
    },
    Exhibit {
        group: "Storage",
        title: "Shelf, alternate layout",
        parts: &[part("shelf_unit_alt", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.8,
    },
    Exhibit {
        group: "Storage",
        title: "Shelf with filled bins",
        parts: &[
            part("shelf_unit_bins", 0.0, 0.0, 0.0, 0.0, 1.0),
            part("shelf_bin_contents", 0.0, 0.0, 0.0, 0.0, 1.0),
        ],
        radius: 3.8,
    },
    Exhibit {
        group: "Storage",
        title: "Low shelf",
        parts: &[part("shelf_unit_low", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.0,
    },
    Exhibit {
        group: "Storage",
        title: "Locker",
        parts: &[part("concept_locker", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.5,
    },
    Exhibit {
        group: "Storage",
        title: "File cabinet",
        parts: &[part("file_cabinet", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.0,
    },
    Exhibit {
        group: "Storage",
        title: "File cabinet, open drawers",
        parts: &[part("file_cabinet_open", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.0,
    },
    Exhibit {
        group: "Storage",
        title: "Storage crate",
        parts: &[part("storage_crate", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 2.7,
    },
    Exhibit {
        group: "Storage",
        title: "Open storage crate",
        parts: &[part("storage_crate_open", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 2.7,
    },
    Exhibit {
        group: "Fixtures",
        title: "Lab console",
        parts: &[part("lab_console", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.3,
    },
    Exhibit {
        group: "Boiler room",
        title: "Boiler unit",
        parts: &[part("boiler_unit", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 5.0,
    },
    Exhibit {
        group: "Boiler room",
        title: "Pipe manifold",
        parts: &[part("pipe_manifold", 0.0, 1.6, 0.0, 0.0, 1.0)],
        radius: 3.0,
    },
    Exhibit {
        group: "Boiler room",
        title: "Tool panel",
        parts: &[part("tool_pegboard", 0.0, 1.6, 0.0, 0.0, 1.0)],
        radius: 3.0,
    },
    Exhibit {
        group: "Boiler room",
        title: "Wall with pipes",
        parts: &[
            part("wall", 0.0, 0.0, 0.0, 0.0, 1.0),
            part("pipe_manifold", 0.0, 1.6, -0.14, 0.0, 1.0),
        ],
        radius: 5.0,
    },
    Exhibit {
        group: "Boiler room",
        title: "Wall with tool panel",
        parts: &[
            part("wall", 0.0, 0.0, 0.0, 0.0, 1.0),
            part("tool_pegboard", 0.0, 1.6, -0.14, 0.0, 1.0),
        ],
        radius: 5.0,
    },
    Exhibit {
        group: "Fixtures",
        title: "Crawl vent, bolted grille",
        parts: &[
            part("concept_crawl_vent", 0.0, 0.9, 0.0, 0.0, 1.0),
            wall_part("vent_grille", 0.0, 0.9, -0.085),
        ],
        radius: 2.8,
    },
    Exhibit {
        group: "Fixtures",
        title: "Crawl vent, grille on floor",
        parts: &[
            part("concept_crawl_vent", 0.0, 0.9, 0.0, 0.0, 1.0),
            part("vent_grille", 0.0, 0.0, -0.72, 0.0, 1.0),
        ],
        radius: 2.8,
    },
    Exhibit {
        group: "Fixtures",
        title: "Removed vent grille",
        parts: &[part("vent_grille", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 2.8,
    },
    Exhibit {
        group: "Fixtures",
        title: "Containment tank",
        parts: &[part("concept_containment_tank", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 4.2,
    },
    Exhibit {
        group: "Fixtures",
        title: "Fuse panel",
        parts: &[part("fuse_panel", 0.0, 0.8, 0.0, 0.0, 1.0)],
        radius: 2.8,
    },
    Exhibit {
        group: "Walls",
        title: "Plain wall",
        parts: &[part("wall", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 5.0,
    },
    Exhibit {
        group: "Walls",
        title: "Conduit wall",
        parts: &[part("wall_conduit", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 5.0,
    },
    Exhibit {
        group: "Walls",
        title: "Wall with vent",
        parts: &[
            part("wall", 0.0, 0.0, 0.0, 0.0, 1.0),
            part("concept_crawl_vent", 0.0, 0.9, -0.14, 0.0, 1.0),
            wall_part("vent_grille", 0.0, 0.9, -0.22),
        ],
        radius: 5.0,
    },
    Exhibit {
        group: "Doors and passages",
        title: "Open passage",
        parts: &[part("wall_doorway", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 5.0,
    },
    Exhibit {
        group: "Doors and passages",
        title: "Closed door",
        parts: &[
            part("wall_doorway", 0.0, 0.0, 0.0, 0.0, 1.0),
            part("door_panel", -0.59, 0.0, 0.0, 0.0, 1.0),
        ],
        radius: 5.0,
    },
    Exhibit {
        group: "Doors and passages",
        title: "Wall post",
        parts: &[part("wall_post", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 4.0,
    },
    Exhibit {
        group: "Walls",
        title: "Mounted vent",
        parts: &[part("wall_vent", 0.0, 1.65, 0.0, 0.0, 1.0)],
        radius: 2.6,
    },
    Exhibit {
        group: "Surfaces",
        title: "Floor tile",
        parts: &[part("floor_tile", 0.0, 0.08, 0.0, 0.0, 1.0)],
        radius: 4.0,
    },
    Exhibit {
        group: "Surfaces",
        title: "Marked floor tile",
        parts: &[part("floor_tile_marked", 0.0, 0.08, 0.0, 0.0, 1.0)],
        radius: 4.0,
    },
    Exhibit {
        group: "Surfaces",
        title: "Ceiling tile",
        parts: &[part("ceiling_tile", 0.0, -1.6, 0.0, 0.0, 1.0)],
        radius: 4.0,
    },
    Exhibit {
        group: "Props",
        title: "Tipped chair",
        parts: &[part("chair_tipped", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 2.5,
    },
    Exhibit {
        group: "Decorations",
        title: "Potted plant",
        parts: &[part("potted_plant", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 2.8,
    },
    Exhibit {
        group: "Props",
        title: "Steel drum",
        parts: &[part("steel_drum", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 2.8,
    },
    Exhibit {
        group: "Props",
        title: "Spilled drum",
        parts: &[part("drum_spilled", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 3.2,
    },
    Exhibit {
        group: "Decorations",
        title: "Paper debris",
        parts: &[part("clutter_papers", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 2.5,
    },
    Exhibit {
        group: "Decorations",
        title: "Tools",
        parts: &[part("clutter_tools", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 1.8,
    },
    Exhibit {
        group: "Decorations",
        title: "Toolbox",
        parts: &[part("toolbox", 0.0, 0.0, 0.0, 0.0, 1.0)],
        radius: 2.5,
    },
    Exhibit {
        group: "Devices",
        title: "Flashbang",
        parts: &[],
        radius: 1.4,
    },
    Exhibit {
        group: "Devices",
        title: "Detector",
        parts: &[],
        radius: 1.4,
    },
    Exhibit {
        group: "Props",
        title: "Fuse pickup",
        parts: &[part("fuse_pickup", 0.0, 0.5, 0.0, 0.0, 1.0)],
        radius: 2.5,
    },
];

const GALLERY: Gallery = Gallery {
    exhibits: EXHIBITS,
    env_var: "PROP_GALLERY_EXHIBIT",
    focus_height,
    spawn_extra: Some(spawn_device),
    min_radius: 1.4,
};

fn main() {
    App::new().add_plugins(GalleryPlugin(GALLERY)).run();
}

fn focus_height(exhibit: &Exhibit) -> f32 {
    match exhibit.title {
        "Boiler unit" | "Containment tank" => 1.5,
        "Pipe manifold" | "Tool panel" => 1.6,
        "Plain wall"
        | "Conduit wall"
        | "Wall with vent"
        | "Open passage"
        | "Closed door"
        | "Wall post"
        | "Wall with pipes"
        | "Wall with tool panel" => 1.45,
        "Shelf" | "Shelf, alternate layout" | "Shelf with filled bins" => 1.1,
        "Tools" => 0.25,
        _ => 0.8,
    }
}

fn spawn_device(
    commands: &mut Commands,
    root: Entity,
    entry: &Exhibit,
    _assets: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    if entry.group != "Devices" {
        return;
    }
    let outline = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.85, 1.0),
        unlit: true,
        cull_mode: Some(Face::Front),
        ..default()
    });
    commands
        .entity(root)
        .with_children(|children| match entry.title {
            "Flashbang" => {
                let body = materials.add(Color::srgb(0.16, 0.24, 0.14));
                children.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.035, 0.12))),
                    MeshMaterial3d(body),
                    Transform::from_xyz(0.0, 0.8, 0.0),
                ));
                children.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.045, 0.14))),
                    MeshMaterial3d(outline),
                    Transform::from_xyz(0.0, 0.8, 0.0),
                ));
            }
            "Detector" => {
                let body = materials.add(Color::srgb(0.32, 0.33, 0.34));
                let face = Color::srgb(1.0, 0.62, 0.18);
                let face = materials.add(StandardMaterial {
                    base_color: face,
                    emissive: LinearRgba::from(face) * 2.0,
                    ..default()
                });
                children.spawn((
                    Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(0.16, 0.07, 0.1)))),
                    MeshMaterial3d(body),
                    Transform::from_xyz(0.0, 0.8, 0.0),
                ));
                children.spawn((
                    Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(0.1, 0.004, 0.06)))),
                    MeshMaterial3d(face),
                    Transform::from_xyz(0.0, 0.835, 0.0),
                ));
                children.spawn((
                    Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(0.18, 0.09, 0.12)))),
                    MeshMaterial3d(outline),
                    Transform::from_xyz(0.0, 0.8, 0.0),
                ));
            }
            _ => unreachable!(),
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn gallery_covers_available_props_and_browses_both_ways() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/facility/modules");
        for entry in EXHIBITS {
            assert!(!entry.parts.is_empty() || entry.group == "Devices");
            for part in entry.parts {
                assert!(
                    root.join(format!("{}.glb", part.module)).is_file(),
                    "{}",
                    part.module
                );
            }
        }
        for title in [
            "Workbench + fuse",
            "File cabinet, open drawers",
            "Shelf with filled bins",
            "Open storage crate",
            "Crawl vent, bolted grille",
            "Crawl vent, grille on floor",
            "Wall with vent",
            "Pipe manifold",
            "Tool panel",
            "Wall with pipes",
            "Wall with tool panel",
            "Open passage",
            "Closed door",
            "Flashbang",
            "Detector",
        ] {
            assert!(EXHIBITS.iter().any(|entry| entry.title == title), "{title}");
        }
        assert_eq!(
            game_settings::cycle(&(0..EXHIBITS.len()).collect::<Vec<_>>(), 0, -1),
            EXHIBITS.len() - 1
        );
        assert_eq!(
            game_settings::cycle(
                &(0..EXHIBITS.len()).collect::<Vec<_>>(),
                EXHIBITS.len() - 1,
                1
            ),
            0
        );
    }
}
