# Shipped asset sources

## UI input prompt

`assets/ui/input-prompts/T_F_Key_Alt.png` is an unchanged copy of the Alt
keyboard F key from JulioCacko's FREE Input Prompts Pack v1.4, sourced from
the Nova Protocol asset bundle. Original source:
https://juliocacko.itch.io/free-input-prompts . Licensed under CC0 1.0
Universal (`credits/licenses/FREE-Input-Prompts_CC0-1.0.md`).

The UI font source and its required license notice are recorded in
`credits/CREDITS.md`.

## Facility modular kit

The 47 GLBs and manifest under `assets/facility/modules/` are copies of the
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
