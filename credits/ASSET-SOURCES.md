# Shipped asset sources

## UI input prompts

The 97 PNGs in `assets/ui/input-prompts/` are unchanged copies of every
individual keyboard and mouse glyph in the Alt style of JulioCacko's FREE Input
Prompts Pack v1.4. They were copied on 2026-10-08 from the Nova Protocol
checkout (`assets/input-prompts/keyboard/Alt/`, checkout commit `b0152d27`;
Nova commit `0521b4af1` added the set). The pack's combined sprite sheet
(`T_Keyboard_Mouse_Key_Alt_Sprite.png` and `.svg`), its other styles, and its
gamepad sets were not copied. Copying the full keyboard and mouse Alt set was
an Automode scope decision, not a direct personal selection.

Original source: https://juliocacko.itch.io/free-input-prompts . Nova
Protocol's `credits/CREDITS.md` records the pack as CC0 1.0 Universal; the
license text is in `credits/licenses/FREE-Input-Prompts_CC0-1.0.md`. Source
and license are taken from Nova's records and were not independently verified
against the upstream page for this copy. `UiAssets::key_glyphs` in
`crates/assets/src/lib.rs` preloads the 76 keyboard glyphs that its
`KEY_GLYPHS` table maps. The mouse glyphs and spare alternates ship unused.

The UI font source and its required license notice are recorded in
`credits/CREDITS.md`. Recorded footstep and door source credits are there too;
see `art/sounds/README.md` for source hashes and edits.

## Human Deer monster

`assets/monster/human_deer_animated.glb` is a byte-identical copy of the
project-animated output at `art/visuals/generated/monster/human_deer_animated.glb`.
Its original rig and artwork came from ceeleste's Sketchfab "The Human Deer"
(https://sketchfab.com/3d-models/the-human-deer-7644694337404bb18eb68e6b637740a1).
The original glTF and bundled license are retained at
`art/visuals/sources/sketchfab/the_human_deer/`; the bundled credit notice is
also copied to `credits/licenses/The-Human-Deer-source-license.txt`. The bundled
files state CC-BY-4.0; these rights/provenance claims are not independently
verified. Changes add the project-authored WALK, CHASE and ATTACK animations
alongside the retained source animation. The exact required credit is in
`credits/CREDITS.md`; generation and checksums are documented at
`art/visuals/generated/monster/README.md`.

## Facility modular kit

The 51 GLBs and manifest under `assets/facility/modules/` are copies of the
project-authored generator output under `art/visuals/generated/modules/`.
`scripts/promote-facility-modules.sh` checks and copies these files. The manifest
records each GLB hash. Regenerate source files with
`scripts/generate-facility.sh` before promoting a revised kit.

Five GLBs (`floor_tile`, `floor_tile_marked`, `wall`, `wall_conduit`,
`wall_doorway`) embed the following third-party textures:

- Item: Poly Haven `concrete_floor_worn_001` (1K JPEG diffuse, OpenGL normal,
  and ARM maps).
- Authors: Dimitrios Savva (photography), Rico Cilliers (processing).
- Source: https://polyhaven.com/a/concrete_floor_worn_001
- License: CC0 1.0 Universal, https://polyhaven.com/license ; text at
  `credits/licenses/CC0-1.0.txt`. Attribution is not required.
- Changes: no changes to the JPEG bytes; embedded in project-authored GLBs.
- Per-file source URLs and SHA-256: `art/visuals/sources/polyhaven/concrete_floor_worn_001/SOURCE.md`.

The Quaternius Modular Sci-Fi MegaKit was a visual reference only. No
Quaternius geometry, textures or UVs were shipped. See
`art/visuals/PROVENANCE.md` for the reference history.
