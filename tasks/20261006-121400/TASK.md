# Research and create PoC visuals

- STATUS: OPEN
- PRIORITY: 0
- TAGS: visuals,research

## User facts

- The Quaternius Modular Sci-Fi MegaKit is a promising visual reference.
- Research how Python scripts could generate original modular facility art
  inspired by the pack's approach. Inspect OBJ files; identify required
  libraries before adding them.
- Poly Haven concrete textures are a candidate for direct CC0 use. Record
  source and license information even when assets are only used as inspiration.
- The initial plan was for review. The owner then approved a Blender PoC, a
  1 unit = 1 meter convention, a Bevy example, and screenshots. Generated art
  and textures stay under `art/`, not `assets/`.

## Agent findings

- The local Standard download includes `License_Standard.txt`, which states
  **CC0 1.0 Universal** and identifies the models as by Quaternius. Pack page:
  https://quaternius.com/packs/modularscifimegakit.html ; license:
  https://creativecommons.org/publicdomain/zero/1.0/ . The archive contains 191
  OBJ files and 190 glTF files, plus FBX, textures, and preview images. It
  includes `glTF` exports with separate `.bin` and image files, which are more
  useful for direct Bevy loading than OBJ. No pack files have been moved into
  the repo.
- Example geometry, from local OBJ headers and vertex/face counts:
  `Platforms/Platform_Simple.obj` is a 4-face floor spanning about 4 x 4 units;
  `Walls/BottomSimple_Straight.obj` is a narrow straight wall base spanning 4
  units; `Platforms/Door_Frame_Square.obj` is a 140-face frame spanning about
  4.86 units wide and 5 units high; `Walls/WallAstra_Straight.obj` is a
  177-face wall with 5 material assignments. These are samples, not a validated
  universal module grid. Some meshes have offsets or rotations in their
  coordinates; infer snap points and pivots from assembled examples rather than
  assuming each file is centered.
- The look uses simple repeatable floor/wall/frame silhouettes, geometric panel
  cuts and bevels, a small material family, reused UV-mapped trim textures,
  normal maps, ORM maps, and restrained emissive accents. This is a *working
  style analysis* from the sample meshes, MTL files, and texture names. The OBJ
  MTLs contain creator-machine absolute Windows image paths; do not use those
  paths as an import pipeline. The glTF files reference local images by
  relative URI.
- Blender 5.2.0 LTS was installed locally but was not initially in the Nix dev
  shell. The PoC added Nix Blender 5.2.2 LTS to the dev shell. Blender's Python API supports mesh
  generation with `Mesh.from_pydata` and `bmesh`, and background execution with
  `blender --background --python`; its glTF exporter can produce game-ready
  glTF/GLB. Sources:
  https://docs.blender.org/api/current/bpy.types.Mesh.html#bpy.types.Mesh.from_pydata
  , https://docs.blender.org/api/current/bmesh.html ,
  https://docs.blender.org/manual/en/latest/advanced/command_line/arguments.html
  ,
  https://docs.blender.org/manual/en/latest/addons/import_export/scene_gltf2.html
  . Check the exporter API on the installed version before writing commands.
- `trimesh` (https://trimesh.org/) can read, inspect, and export meshes but
  would be an extra Python dependency and is less suitable for bevels, UV
  editing, and art iteration. A standard-library-only OBJ parser is enough for
  measurements. Start with Blender only for the PoC; decide whether and how to
  pin Blender in Nix before implementation. No new library has been added.
- Poly Haven marks assets CC0: https://polyhaven.com/license . Candidate worn
  concrete: https://polyhaven.com/a/concrete_floor_worn_001 . CC0 does not
  require attribution, but record provenance and license anyway. The repo's
  `credits/CREDITS.md` says not to add attribution entries unless required;
  keep research references in this task and `art/visuals/README.md`, then store
  source/license metadata alongside any imported asset and copy applicable
  license texts to `credits/licenses/` when packaging calls for them. Do not
  treat inspiration references as shipped assets.

## Proposed plan (initial research; PoC results below)

1. **Style and interface study:** inspect a small set of local floor, straight
   wall, inner/outer corner, door frame, and light meshes in Blender; measure
   dimensions, origins, orientation, UV ranges, triangle counts, material
   slots, and how adjacent pieces join. Make a visual contact sheet or
   comparison renders from our own scene. Do not try to reproduce Quaternius
   mesh coordinates or UV layout exactly; derive a simple facility design
   language.
2. **Define generator contract:** choose one snap grid from measured joins, a
   fixed up axis, named piece variants, a common pivot/door clearance,
   dimensions, a deterministic seed, output directory, and manifest format.
   Target a minimal set: floor, straight wall, corner, door opening, ceiling,
   and a lamp or vent. Keep collision proxies/door clearance explicit. Treat
   grid size and aesthetic palette as reviewable choices, not facts settled by
   the downloaded pack.
3. **Build a small headless Blender Python PoC:** create mesh primitives with
   `bpy`/`bmesh`, beveled hard edges, a repeated panel/trim system, and a few
   material slots. Export glTF/GLB for Bevy; keep scripts and generated outputs
   separate. First validate untextured geometry and modular joins. Only then
   add UVs and a small shared material/trim atlas if needed. Compare
   Blender-only generation against a plain Python OBJ export only if Blender
   becomes a blocker; avoid `meshio`/`trimesh` until a need is proven.
4. **Texture trial:** select 1-2 *specific* Poly Haven CC0 textures at 1K/2K.
   Download only the maps needed for the prototype (base color, OpenGL normal,
   roughness or packed maps); test texture scale and normal/metallic-roughness
   channels in Blender and Bevy rather than assuming drop-in compatibility.
   Keep Quaternius' trim textures as a visual reference unless direct reuse is
   approved. Record exact Poly Haven item URLs, authors if listed, CC0 URL,
   files, transformations, and checksums with promoted files.
5. **Scene test:** assemble one corridor turn, one room, and one door from
   generated pieces. Check joins, visible seams, collision, wayfinding under
   low light, image scale, glTF dependencies, and packaging through the Nix
   build. Iterate style away from clean sci-fi toward a worn, functional
   facility.
6. **Document and gate:** save generation steps, asset manifest,
   license/provenance records, example renders, limits, and measured results.
   Ask for review of generated vs. reference screenshots before expanding the
   module set or committing to a permanent asset pipeline.

## Verification and done when

- Initial research phase: plan, observed local pack facts, tool choices,
  source/license links, and open decisions were recorded before implementation.
- Future PoC: a fresh machine can run the documented script with declared
  tools; same input/seed yields the same named module set; pieces join without
  gaps at the chosen snap points; doors have usable clearance; a Bevy test
  scene loads the exported glTF/GLB and selected textures without missing
  files; source and CC0 provenance are recorded for each shipped third-party
  file.

## PoC outcome (2026-10-06)

The owner approved implementation with these terms: original geometry only,
Blender in the dev shell, 1 Blender unit = 1 m, one Poly Haven CC0 concrete,
no files under `assets/`. The Bevy example now captures a screenshot and a
short optional silent video through the autopilot/capture crates. This task
stays OPEN for owner review of the visual direction.

- `flake.nix`: the dev shell has `blender` (nixpkgs 5.2.2 LTS, from the binary
  cache).
- `scripts/fetch-facility-textures.sh` downloads 3 x 1K JPEG maps of
  `concrete_floor_worn_001` and checks them against pinned SHA-256 values.
  The MD5 values also match the Poly Haven API.
- `scripts/generate_facility.py` (Blender Python, `bmesh`) builds 12 original
  modules: floor, marked floor, ceiling, wall, conduit wall, doorway, door
  leaf, post, 3 ceiling-light states, and a red wall lamp. It then builds a
  5-tile corridor, a closed side door, an open end door, and a 3 x 3 room. It
  writes `art/visuals/generated/facility.glb` and `facility.manifest.json`.
  `scripts/generate-facility.sh` runs it headless with `PYTHONHASHSEED=0`.
- `scripts/check_facility_glb.py` (standard library only) checks the GLB
  bounds, slab height, node names, anchor positions, and embedded images
  against the manifest.
- Measured result: 2.5 m grid, 3.0 m walls, 0.2 m wall core, 1.2 x 2.2 m door
  opening, 2.3 m corridor clear width. Bevy bounds are (-3.94, -0.1, -20.19)
  to (3.94, 3.1, 0.19). The GLB has 82 mesh nodes, 9,028 triangles, and is
  598 KB. The floor top is at y = 0 and the corridor runs along -Z. The
  manifest module bounds confirm the physical sizes (floor tile
  2.5 x 2.5 x 0.1, wall 2.5 x 3.0).
- Verification: the headless export passed in the Nix dev shell. The checker
  passes. In the dev shell, 7 consecutive runs gave the same SHA-256. One
  earlier first run gave a different accessor layout with the same geometry;
  the cause is not known. A Cycles preview from the recommended camera showed
  correct joins, the guide stripes, and the lit doorway. That preview is not
  in the repo.
- Records: `art/visuals/PROVENANCE.md`,
  `art/visuals/sources/polyhaven/concrete_floor_worn_001/SOURCE.md`,
  `art/visuals/sources/CC0-1.0.txt`, and `art/visuals/generated/README.md`
  (axes, dimensions, camera, and light anchors).
- Recommended Bevy camera:
  `Transform::from_xyz(0.0, 1.6, -0.6).looking_at(Vec3::new(0.0, 1.3, -16.5), Vec3::Y)`
  with the default 45 degree FOV. Put point lights at the `light_*` anchor
  nodes. The light intensities are not calibrated yet.
- Bevy screenshot verification: `art/visuals/screenshots/facility_gallery.png`
  is a nonblank 1280x720 view of the corridor, side door, concrete walls and
  floor stripes with cool overhead lighting and an amber doorway. The video
  `facility_gallery.webm` is a 1280x720 VP9 capture at 30 fps, 62 frames,
  2.066 seconds. It has no audio because no game-audio PCM adapter exists yet.
- Remaining: owner review of the image, lighting intensity, and generated look
  against the reference before the module set grows.

## Module kit and Bevy layout (2026-10-06)

The owner approved steps 1-4: split the kit into separate GLB modules under
`art/visuals/generated/`, then compose an authored multi-room layout in Bevy.
`facility.glb` and `facility_gallery` stay as the baseline.

- `scripts/generate_facility.py --target kit` writes 18 modules to
  `art/visuals/generated/modules/` and `modules.manifest.json` (snap type,
  Bevy bounds, light anchors, hashes). New original modules: `exit_sign`,
  `wall_vent`, `storage_crate`, `steel_drum`, `shelf_unit`, `workbench`.
  The kit uses a 2.5 m UV period, so floor and wall textures continue across
  module joins.
- `scripts/generate-facility.sh` now writes only the kit by default. The
  argument `facility` rewrites `facility.glb`. Finding: Blender's "beauty"
  triangulation breaks ties by memory address. A different script text or
  hash seed changed the triangle index order of `facility.glb` (same
  vertices). The kit uses fixed triangulation, fan caps, and a sorted vertex
  and face order; seeds 0 to 4 gave identical files.
- `scripts/check_facility_modules.py` checks hashes, root nodes, bounds, snap
  pivots, the door opening clearance, and UV continuity.
- `examples/facility_layout/` derives walls, doorways, door leaves, and posts
  from areas and openings in `layout.rs`, with unit tests in
  `tests/layout.rs`. Three corridors and three rooms; cool, amber, flicker,
  dead, red fault, and exit-sign lighting; four prop types. The autopilot run
  writes `art/visuals/screenshots/facility_layout_*.png` (plan and five eye
  views).
- Details and commands: `art/visuals/generated/README.md`.

## Wayfinding and story prototype (2026-10-06)

Owner-approved visual prototype in `facility_layout`. Visual only: no hiding,
collision, interaction, monster, or encounter mechanics.

- Wall-mounted direction signs (label plate + arrow). Each sign names its
  reader cell; `derive` computes each arrow from the walkable path and rejects
  wrong-way and backward arrows. Only EXIT hangs overhead.
- Route colors on one wall band per corridor segment: orange boiler room,
  blue storage, green exit. Short branch bands at the intake end. No floor
  lines.
- New `boiler` and `lab` rooms. The lab has a broken containment tank as a
  first-encounter landmark (concept, not implemented). The maintenance room
  has a central work island and floor clutter; the lab and intake have
  monster traces.
- Kit: 46 modules (13 new, 4 obsolete removed); the other 33 module hashes
  are unchanged. `facility.glb`, the gallery, `assets/`, and the
  `facility_layout_*.png` baseline images are unchanged.
- Screenshots: `art/visuals/screenshots/facility_wayfinding_*.png`.
- Details, color rationale, tests, and commands:
  `art/visuals/generated/README.md`.

## Open decisions for owner review

- Generate original assets only, or also allow direct use of selected CC0
  MegaKit modules for early blockout?
- Keep a clean sci-fi trim-sheet look, or shift to painted concrete and
  practical industrial hardware? The plan assumes the latter as a test, not a
  decision.
- Use physical keys, keycards, or fuses for pickup silhouettes? This affects
  the first prop list but not the modular geometry study.
