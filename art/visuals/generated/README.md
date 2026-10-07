# Generated facility PoC

`facility.glb` is an original modular corridor and room, made by
`scripts/generate_facility.py` in headless Blender. `facility.manifest.json`
records module sizes, bounds, light anchors, the camera, and the GLB hash. It
is a review artifact, not a shipped asset. It is the frozen baseline for the
`facility_gallery` example.

`modules/` holds the source generator output as separate GLB files, plus
original props and decorations. `scripts/promote-facility-modules.sh` verifies
and copies the 51 GLBs and their manifest to `assets/facility/modules/` for
distribution. The `facility_layout` example and the game load the promoted
copy through `game_assets`. See "Module kit" below.

## Regenerate and check

```sh
nix develop -c ./scripts/generate-facility.sh
nix develop -c python3 scripts/check_facility_modules.py
nix develop -c python3 scripts/check_facility_glb.py
./scripts/promote-facility-modules.sh
nix develop -c python3 scripts/check_facility_modules.py assets/facility/modules
```

`generate-facility.sh` downloads and verifies the Poly Haven maps with
`fetch-facility-textures.sh`. Then it runs Blender with
`--background --factory-startup` and `PYTHONHASHSEED=0`. Blender comes from
the dev shell (5.2.2 LTS at the time of writing). The generator has no random
input.

With no argument, the script writes only the module kit
(`--target kit`). The kit is byte-stable: runs with `PYTHONHASHSEED` 0 to 4
gave the same 18 GLB hashes. For the kit, the generator triangulates quads
with a fixed diagonal, caps cylinders with triangle fans, and writes vertices
and faces in a sorted order.

`./scripts/generate-facility.sh facility` rewrites `facility.glb`. Do not do
this unless you want a new baseline. The legacy path keeps Blender's "beauty"
triangulation. That triangulation breaks ties by memory address, so a
different script text or hash seed can give a different triangle index order
for the same vertices. The committed `facility.glb` is the original file
(SHA-256 `7367a56d...`).

`check_facility_glb.py` reads `facility.glb` with only the Python standard
library. It checks the world bounds, the slab height, the named nodes, the
camera and light anchor positions, and that the images are embedded JPEGs.
`check_facility_modules.py` checks the kit; see "Module kit".

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

## Module kit

`modules/<name>.glb` holds one module per file. Each file has one root node
with the module name, an identity transform, and one mesh. The front of each
module faces Bevy **-Z**. `modules/modules.manifest.json` records, per module:
the category, the snap type, the Bevy-space bounds, the triangle count, the
materials, the light anchor and color for light fixtures, and the GLB hash.

| Snap | Pivot |
| --- | --- |
| `cell_center` | Cell center on the floor top. Place at (2.5 i, 0, 2.5 k). |
| `edge_center` | Middle of a cell edge on the floor top. Length along X. Rotate about Y so that -Z points into the area. |
| `vertex` | Grid vertex on the floor top. |
| `door_hinge` | Hinge edge on the floor. The leaf extends along +X. |
| `wall_mount` | Back center on the wall face. The fixture extends to -Z. The height is chosen at placement. |
| `floor` | Bottom center on the floor top. |

| Module | Category | Snap | Size x/y/z (m) | Triangles | KiB |
| --- | --- | --- | --- | --- | --- |
| `floor_tile` | floor | `cell_center` | 2.5 x 0.1 x 2.5 | 44 | 380 |
| `floor_tile_marked` | floor | `cell_center` | 2.5 x 0.103 x 2.5 | 68 | 384 |
| `ceiling_tile` | ceiling | `cell_center` | 2.5 x 0.22 x 2.5 | 72 | 9 |
| `wall` | wall | `edge_center` | 2.5 x 3.0 x 0.25 | 156 | 394 |
| `wall_conduit` | wall | `edge_center` | 2.5 x 3.0 x 0.375 | 276 | 404 |
| `wall_doorway` | wall | `edge_center` | 2.5 x 3.0 x 0.278 | 524 | 436 |
| `door_panel` | door | `door_hinge` | 1.18 x 2.18 x 0.13 | 92 | 14 |
| `wall_post` | structure | `vertex` | 0.38 x 3.0 x 0.38 | 56 | 8 |
| `ceiling_light_cool` / `_dead` / `_amber` | fixture | `cell_center` | 1.4 x 0.22 x 0.32 | 104 | 12 |
| `wall_lamp_red` | fixture | `wall_mount` | 0.24 x 0.32 x 0.16 | 204 | 18 |
| `fuse_panel` | decoration | `wall_mount` | 1.1 x 1.3 x 0.09 | 1564 | 167 |
| `fuse_pickup` | prop | `floor` | 0.2 x 0.06 x 0.06 | 472 | 37 |
| `exit_sign` | decoration | `wall_mount` | 0.5 x 0.2 x 0.07 | 92 | 13 |
| `wall_vent` | decoration | `wall_mount` | 0.6 x 0.4 x 0.04 | 248 | 30 |
| `storage_crate` | prop | `floor` | 1.02 x 0.7 x 0.825 | 104 | 14 |
| `steel_drum` | prop | `floor` | 0.6 x 0.91 x 0.6 | 352 | 24 |
| `shelf_unit` | prop | `floor` | 1.8 x 2.0 x 0.508 | 672 | 72 |
| `workbench` | prop | `floor` | 1.6 x 1.5 x 0.7 | 216 | 27 |

The original visual fuse panel is a wall-mounted steel cabinet with a MAIN
label, a breaker grid, two yellow tripped switches, and a small latch. It is
not a working electrical system. Its GLB bounds and SHA-256 are recorded in
`modules.manifest.json` and checked with the other modules.

The original `fuse_pickup` is a knife-blade cartridge fuse: a ceramic body
with a yellow band, steel end caps, and flat contact blades. It lies along X
with its origin at the bottom center, so it rests on a table top (for example
`concept_table` at 0.8 m) at the placement height. The -Z view gives a flat
fuse silhouette for a possible UI icon. It is a visual mesh only; the
pickup, outline, and collision are gameplay work.

The initial 18-module kit was about 2.3 MB; later modules, including the
fuse panel, extend that kit. The five concrete modules (floors and walls)
each embed their own copy of the three 1K Poly Haven JPEGs (about 380 KB), so
Bevy loads one image set per module file. A shared external-image `.gltf`
would avoid this. It is a known cost of separate GLB files.

Differences from the modules inside `facility.glb`:

- UV period 2.5 m instead of 3.0 m: one texture repeat per tile. Floors and
  walls tile without a texture jump at module joins. The Poly Haven scan is
  drawn about 17% smaller than its real size.
- Fixed triangulation and fan caps (see above). The shapes are the same, but
  `wall_conduit`, `wall_lamp_red`, and the ceiling lights have a few more
  triangles (cylinder caps).
- New materials: `lamp_green` (emissive), `paint_crate`, `paint_drum`.
- New modules: `exit_sign`, `wall_vent`, `storage_crate`, `steel_drum`,
  `shelf_unit`, `workbench`. All are box and cylinder primitives from the
  generator. No third-party mesh is used.

`check_facility_modules.py` (standard library only) checks the kit against
the manifest: the file set, hashes and sizes; one identity root node per
file; embedded JPEG images and repeat samplers; the bounds, which it computes
from the vertex data; the snap contract for each snap type; that no triangle
of `wall_doorway` enters the 1.2 x 2.2 m opening and that `door_panel` fits
it; and texture continuity. For continuity it checks that the UV period
divides the tile and that each textured vertex has the UV of the box
projection, modulo 1. Thus the UVs at the two sides of a joint agree.

## Bevy composition (`facility_layout`)

`examples/facility_layout/` composes the kit in Bevy from the authored first
floor in `crates/gameplay/src/levels/first_floor.rs`. The level is a
declaration made with the small builder in
`crates/gameplay/src/facility/level.rs`: rooms as cell rectangles, openings
(passage, open door with a swing angle, or the locked exit), ceiling lamps,
wall fixtures, wall signs, exit hangers, props, lights, glow, objectives, and
the player start. `FacilityPlugin` builds the level in one pass. Each logical
entity is spawned together with its scene children and point lights. In a
windowed app the pass runs when `GameAssetsState` is `Ready`. Without the
asset state (headless), or when loading fails, the same declaration is built
without visuals. From the rooms and openings, the build makes:

- one floor and one ceiling tile per cell;
- one wall on each cell edge between different areas, or between an area and
  empty space. A shared wall uses the style of the first area in the list and
  faces it;
- a `wall_doorway` and a `door_panel` on each door edge, and nothing on a
  passage edge;
- posts at corners, wall ends, T-joints, and changes of wall style or facing,
  and on every second vertex of a straight run. The other joints show a plain
  wall-to-wall seam, to show the texture continuity.

Facility point lights do not cast shadows. The production code does not
validate or route. The tests in `crates/gameplay/tests/level.rs` check the
authored level: clearances, door zones, wall slots, sign arrows against a
test-only route search, and readability. Set `FACILITY_SHOT_DIR` to write the
example captures to another directory.

The current first floor uses sixteen areas on a 2.5 m grid:

| North to south | West | Center | East |
| --- | --- | --- | --- |
| Far north | maintenance | EXIT room, service below | smaller security, office below |
| Upper middle | utility, west hall | open reception with desk | east hall, red-lit storage |
| Lower middle | west hall | intake | east hall |
| Bottom | boiler (one door), west hall | damaged lab | east hall, solo locker hiding room |

The EXIT is a 4 x 4 tile objective room. Its north edge has the exterior
EXIT door; its internal doors join service, the relocated 3 x 2 tile office,
and the smaller 2 x 3 tile security room. The fuse panel on its north wall is
an original Blender-generated visual prototype: breakers, indicators, and
labeling communicate the objective, but it has no input, power state, or
gameplay logic. Service, intake, and both halls remain one cell wide.
Reception replaces the former narrow cross and its four isolated wall/ceiling
blocks; its desk and papers sit away from its open center path and hall
entrances. Short lower lateral links connect intake to the halls and
three-cell-wide lab. The halls join the lab at its west and east doors; the
center route has independent bypasses. Utility opens from the west hall;
the hiding room has only one east-hall entrance and lockers are visual
concepts without hiding mechanics. Boiler has only a west-hall entrance and
is not a bypass. Red-lit storage opens from the east hall, not office or
security. EXIT arrows on both faces of ceiling hangers follow the route for
each reader. Authored colored floor route lines are hidden pending a separate
design pass; generic route modules and path tests remain.

The earlier `facility_floor1_*.png` revision had a narrow cross with four
isolated blocks, an EXIT door on service, and a larger top-right office.
Those images are retained as history; `facility_floor1_v2_*.png` shows this
current plan.

```sh
nix develop -c cargo test -p gameplay
nix develop -c xvfb-run -a cargo run --example facility_layout
```

The current run writes `art/visuals/screenshots/facility_floor1_v2_*.png` and
exits. The earlier `facility_floor1_*.png`, `facility_layout_*.png` baseline,
and `facility_wayfinding_*.png` study are preserved. The plan view is
orthographic from above, with ceilings
and ceiling lights hidden. Other views are at 1.6 m eye height with distance
fog. Light intensities are not calibrated. There is no player controller or
collision.

## Archived wayfinding study (`facility_wayfinding_*`)

The area coordinates, fixture locations, line routes, and capture list below
describe the earlier study, not the current `facility_layout` example.

This was an earlier visual prototype in the `facility_layout` example. It
included signs, colored floor route lines, a boiler room, a lab, clutter, and
hiding-space silhouettes. It has no gameplay: no hiding, collision, interaction, monster,
or encounter. The first encounter in the lab is a concept for review. It is
not implemented.

New areas: `boiler` (-3, -4)..(-2, -3), off the west arm of the cross, and
`lab` (-3, -2)..(-1, -1), behind the open side door of the intake.

### Signs

- Directional signs are on walls. Each row is a `sign_label_*` plate and a
  `sign_arrow`. A `WallSign` fixture names its reader cell. The reader faces
  the wall. `derive` computes the arrow from the shortest walkable path from
  the reader cell: the first move that turns or leaves a junction gives Left,
  Right, or Ahead. A wrong or backward arrow is an error. An arrow is on the
  side that it points to.
- Signs: `<- BOILER ROOM` and `<- MAINTENANCE` on the cross wall to the left
  of the intake; `STORAGE ->` and `OFFICE ^` on the wall to the right; and
  `<- MAINTENANCE`, `STORAGE ->`, `OFFICE ->` across from the boiler-room door.
- Only EXIT hangs from the ceiling (`sign_hanger`), as in a hospital. Both
  faces carry `sign_label_exit` and a downward `sign_arrow`, and each face
  reads correctly (not mirrored) for the person in front of it. The down arrow
  means "exit ahead": the shortest exit path from the hanger cell starts
  straight ahead for the reader of the front face (the face that points along
  `facing`). `derive` rejects a hanger whose front reader must turn or turn
  back. The back face carries the same graphic as an exit-route marker. It
  makes no direction claim. Four hangers: intake, both cross arms, service.
  The bottom is at 2.2 m.
- Door plaques above the doors name the room behind them.
- The tests check, per wall sign and viewpoint: the arrow direction against
  the path, line of sight (walls, door openings, hangers), a view angle of 45
  degrees or less, a look-up angle under 25 degrees, and a distance of at most
  40 times the 0.112 m cap height.

### Route colors

| Color | Destination | Why |
| --- | --- | --- |
| orange | boiler room | heat and hazard |
| blue | storage | neutral service color, clear against orange |
| green | exit | the usual egress color, same as the EXIT signs |

Maintenance and office have signs only. More line colors would make the dark
cross too busy. The chip on each label has its route color.

The route lines are painted on the floor (`route_line_*`, snap `floor_line`,
0.1 m wide, 3 mm high, walkable, scaled along X). There are no wall bands. A
`RouteLine` runs along `heading` for `cells` cells, 0.55 m from the cell
center toward `side`. This is inside the yellow floor stripes at 0.95 m, and
0.5 m clear of the wall faces. At the intake end, an orange line on the left
and a blue line on the right mark the split. Each line runs into the
junction and turns there on the inside of its turn. The next line starts at
the outer edge of the first line, so the corner has no gap and no overlap.
Green runs the service corridor and stops 0.25 m short of the exit door. A
line also stops 0.25 m short of any wall or door frame.

`derive` rejects a line against the route, on the outside of a turn, that
turns without a line on the new leg, or that crosses a prop, a walkable
decal, a floor stripe, or another line. The tests also check that each line
runs through every junction on its path, that the corners join, that the
line midpoints are on free floor, and that no cell carries more than two
colors.

### Boiler room, lab, and clutter

- Boiler room: `boiler_unit` with a flickering fire light, `pipe_manifold`,
  `tool_pegboard` (one missing-tool outline), `concept_crawl_vent`, and a
  `vent_grille` on the floor.
- Lab (first-encounter concept): `concept_containment_tank`, a cage and glass
  tank with the front broken out, shards and fluid residue on the floor, and a
  weak flickering teal light inside. The breach faces the lab door. Light
  `glass_edge` rims mark the jagged hole. The checker fails if any tank
  triangle closes the breach box (`breach_bevy` in the manifest).
  `lab_console`, a tipped chair, papers with a three-toe print smudge, drag
  marks toward the door, and claw marks inside the lab and in the intake.
- Maintenance (lived-in, disturbed): a freestanding `work_island` with tools
  and papers on top, under the amber light, plus a tipped chair, a spilled
  drum, a tipped toolbox, papers, claw marks, and the hiding table.
- Hiding silhouettes (visual concepts only): `concept_locker`,
  `concept_table` (clear space under the top), `concept_crawl_vent`.
- Floor papers and drag marks are `walkable` decals (3 cm or less). Other props
  may not overlap. A test fills the floor on a 0.1 m grid with a 0.3 m body
  radius: every doorway and every free cell center must connect. The work
  island has a clear aisle of at least 0.5 m on all sides.

### Views and commands

```sh
scripts/generate-facility.sh
python3 scripts/check_facility_modules.py
nix develop -c cargo test -p gameplay
nix develop -c xvfb-run -a cargo run --example facility_layout
```

The earlier run wrote `art/visuals/screenshots/facility_wayfinding_*.png`: `plan`,
`intake`, `cross`, `maintenance`, `storage`, `service` (the baseline cameras
on the new layout), `signage` (intake wall signs and EXIT hanger),
`signage_boiler` (the sign across from the boiler door), `boiler`, `lab` (the
broken tank), `workshop` (the work island and clutter), and `hiding_locker`,
`hiding_table`, `hiding_vent`. The committed `facility_layout_*.png` images
are the baseline. This run does not write them.

Limitations: the glyphs are a 5x7 block font made from boxes (no font file).
Light intensities are not calibrated. The glass is alpha-blended and casts no
shadow. The arrow rule uses cell paths, not a navigation mesh. Line of sight
checks walls, door openings, posts, and EXIT hangers, but ignores props.
There is no player, collision, or interaction.
