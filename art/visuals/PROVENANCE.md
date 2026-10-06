# Visual provenance and licenses

Records for every third-party input to `art/visuals/`. The modular GLBs are
also shipped under `assets/facility/modules/`. Their distribution record is
`credits/ASSET-SOURCES.md`; follow `credits/CREDITS.md` for assets that require
attribution.

| Item | Role | Files in repo | License | Record |
| --- | --- | --- | --- | --- |
| Poly Haven `concrete_floor_worn_001` (Dimitrios Savva, Rico Cilliers) | Direct use: concrete maps embedded in `generated/facility.glb` and in the concrete modules in `generated/modules/` | `sources/polyhaven/concrete_floor_worn_001/*.jpg` (3 x 1K JPEG) | CC0 1.0 | `sources/polyhaven/concrete_floor_worn_001/SOURCE.md`, `sources/CC0-1.0.txt` |
| Quaternius Modular Sci-Fi MegaKit, Standard (free) edition | Visual reference only: module scale, panel rhythm, trim/emissive accents | None. No mesh, texture, or UV data was copied, imported, or traced. | CC0 1.0, from `License_Standard.txt` in the local download | This file |
| `generated/facility.glb` | Original geometry, made by `scripts/generate_facility.py` from box and cylinder primitives | `generated/facility.glb`, `generated/facility.manifest.json` | Project-authored; embeds the CC0 Poly Haven maps above | `generated/README.md` |
| `generated/modules/*.glb` | Original module kit and props, made by `scripts/generate_facility.py --target kit` from box and cylinder primitives | 47 GLB files, `generated/modules/modules.manifest.json`; shipped copies at `assets/facility/modules/`. Sign text is a 5x7 block font built from boxes in the generator; no font file is used. | Project-authored; `floor_tile`, `floor_tile_marked`, `wall`, `wall_conduit`, and `wall_doorway` embed the CC0 Poly Haven maps above | `generated/README.md`, `credits/ASSET-SOURCES.md` |

## Quaternius Modular Sci-Fi MegaKit (reference only)

- Pack page: https://quaternius.com/packs/modularscifimegakit.html
- Author: Quaternius (https://quaternius.com)
- License: CC0 1.0 Universal,
  https://creativecommons.org/publicdomain/zero/1.0/ . The local copy at
  `~/Downloads/Modular SciFi MegaKit[Standard]/License_Standard.txt` states it.
- Use: we studied `Preview_1.png` to `Preview_3.png` and measured glTF bounds
  in headless Blender on 2026-10-06. The generator does not read the pack.
- Measured facts (Blender Z-up, kit units):
  - Floor plates (`Platform_Simple`, `Platform_Metal`): 4 x 4, 8 triangles.
  - Straight walls sit on the tile edge at x = -2 (`WallBand_Straight` is 4
    long and 3 high). The `Top*` trims continue from 3 to 5.
  - Door frames (`Door_Frame_Square`): 4.86 wide, 5 high, 0.5 deep, 280
    triangles. Door leaves: 2.11 x 4.05, pivot at one edge.
  - Lights (`Prop_Light_Small`/`Wide`): 0.86 or 1.32 long, with a separate
    emissive `M_Light` material.
  - Detail walls (`WallAstra_Straight`): 355 triangles, 5 material slots.
- Takeaways used in our own design: one square snap grid; walls and trims as
  stacked horizontal bands; frames and posts that hide module joins; a small
  material family; restrained emissive strips. The kit's 4-unit grid and 5 m
  doors read as oversized for human scale. Our grid is 2.5 m, with 3.0 m walls
  and a 1.2 x 2.2 m door opening.
