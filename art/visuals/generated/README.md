# Generated facility PoC

`facility.glb` is an original modular corridor and room, made by
`scripts/generate_facility.py` in headless Blender. `facility.manifest.json`
records module sizes, bounds, light anchors, the camera, and the GLB hash. It
is a review artifact, not a shipped asset.

## Regenerate and check

```sh
nix develop -c ./scripts/generate-facility.sh
nix develop -c python3 scripts/check_facility_glb.py
```

`generate-facility.sh` downloads and verifies the Poly Haven maps with
`fetch-facility-textures.sh`. Then it runs Blender with
`--background --factory-startup` and `PYTHONHASHSEED=0`. Blender comes from
the dev shell (5.2.2 LTS at the time of writing). With that Blender, repeated
runs gave byte-identical GLBs. The generator has no random input. The checker
reads the GLB with only the Python standard library. It checks the world
bounds, the slab height, the named nodes, the camera and light anchor
positions, and that the images are embedded JPEGs.

## Units and axes

- 1 Blender unit = 1 m. The script sets `unit_settings.scale_length = 1.0`
  and metric meters.
- The generator builds the scene Z-up, with the corridor along Blender +Y. The
  exporter writes glTF Y-up, so in Bevy: `bevy = (x, z, -y)`. The corridor
  runs along Bevy **-Z**, which is the default camera forward direction.
- Origin: the floor top is at Bevy y = 0. Bevy x = 0 is the corridor center
  line. The closed back wall of the corridor is on the z = 0 line.
- Spawn the scene at the identity transform (`facility.glb#Scene0`).

## Layout (Bevy coordinates, meters)

| Part | Extent |
| --- | --- |
| Grid | 2.5 m tiles. Wall center lines are on tile edges. |
| Corridor | x -1.25..1.25, z 0..-12.5 (5 tiles). Clear width 2.3 m between wall faces (2.25 m at the skirting and waist trim), 2.14 m between posts. |
| Side door | Left wall at x = -1.25, wall tile z -5.0..-7.5, opening z -5.65..-6.85 (1.2 x 2.2 m), closed. |
| End doorway | z = -12.5, opening x -0.6..0.6, height 2.2 m. The door leaf is open 62 degrees into the room. |
| Room | x -3.75..3.75, z -12.5..-20.0 (3 x 3 tiles, 7.5 m square) |
| Floor slab | y -0.1..0. Yellow guide stripes at x = +/-0.95 in the corridor. |
| Ceiling | Underside at y = 3.0 (ribs drop to 2.88). Slab top at 3.1. |
| Whole GLB | min (-3.94, -0.1, -20.19), max (3.94, 3.1, 0.19) |

## Modules

All modules are box and cylinder primitives with small bevels. The UVs use a
world-scale box projection (one texture tile = 3 m, the real size of the Poly
Haven scan). Instances share mesh data.

| Module | Size (m) | Triangles | Instances |
| --- | --- | --- | --- |
| `floor_tile` / `floor_tile_marked` | 2.5 x 2.5 x 0.1 | 44 / 68 | 9 / 5 |
| `ceiling_tile` | 2.5 x 2.5, slab 0.1 plus 0.12 ribs | 72 | 14 |
| `wall` | 2.5 long, 3.0 high, 0.2 core (0.25 at the skirting) | 156 | 14 |
| `wall_conduit` | `wall` plus two pipes at 2.43 and 2.55 m height | 268 | 7 |
| `wall_doorway` | `wall` with a 1.2 x 2.2 opening and a steel frame | 524 | 2 |
| `door_panel` | 1.18 x 2.18 x 0.06, pivot at the hinge edge | 92 | 2 |
| `wall_post` | 0.36 x 0.36 x 3.0, at wall joins | 56 | 22 |
| `ceiling_light_cool` / `_dead` / `_amber` | 1.4 x 0.32, hangs to y 2.78 | 96 | 3 / 2 / 1 |
| `wall_lamp_red` | 0.24 x 0.32, 0.16 deep | 184 | 1 |

Total: 82 mesh nodes, 9,028 triangles, about 0.6 MB with three 1K JPEG maps.

Materials: `concrete_floor`, `concrete_wall`, `paint_lower` (Poly Haven maps
tinted by `baseColorFactor`); `steel_dark`, `steel_door`, `steel_ceiling`,
`conduit`, `paint_hazard`, `glass_dark`, `lamp_dead`; and the emissive
`lamp_cool`, `lamp_amber`, `lamp_red` (`KHR_materials_emissive_strength`
6-8). The materials are single-sided. The export also contains tangents.

## Camera and lights for the Bevy example

The GLB has no cameras or lights. It has empty nodes with glTF extras so that
the example can find them by `Name`:

- `camera_main`: extras `{"anchor": "camera", "target": [0, 1.3, -16.5]}`
- `light_corridor_0`, `light_corridor_1`, `light_corridor_3`: cool ceiling
  lights. Cells 2 and 4 have dead fixtures and get no light.
- `light_room_amber`: the amber ceiling light in the room.
- `light_room_fault_red`: the red fault lamp on the far room wall.

Recommended main camera (eye height 1.6 m, at the closed end of the corridor,
looking through the open door into the amber room):

```rust
Camera3d::default(),
Transform::from_xyz(0.0, 1.6, -0.6).looking_at(Vec3::new(0.0, 1.3, -16.5), Vec3::Y),
```

Use the default `PerspectiveProjection` (45 degree vertical FOV) at 16:9. A
Cycles preview at these values shows both corridor walls, the side door, the
guide stripes, and the lit doorway in the center.

| Anchor | Bevy position | Linear color |
| --- | --- | --- |
| `light_corridor_0` | (0.0, 2.7, -1.25) | (0.78, 0.88, 1.0) |
| `light_corridor_1` | (0.0, 2.7, -3.75) | (0.78, 0.88, 1.0) |
| `light_corridor_3` | (0.0, 2.7, -8.75) | (0.78, 0.88, 1.0) |
| `light_room_amber` | (0.0, 2.7, -16.25) | (1.0, 0.58, 0.2) |
| `light_room_fault_red` | (2.2, 2.2, -19.65) | (1.0, 0.06, 0.03) |

Put `PointLight`s at these anchors. A directional light does not reach the
interior, because the ceiling closes the scene. The intensities are not
calibrated: tune them from the Bevy screenshot. The Blender preview used about
4:1 power between a ceiling light and the red lamp.
