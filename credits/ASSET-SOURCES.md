# Shipped asset sources

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
