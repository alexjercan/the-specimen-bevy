# Poly Haven: Concrete Floor Worn 001

| Field | Value |
| --- | --- |
| Asset page | https://polyhaven.com/a/concrete_floor_worn_001 |
| API record | https://api.polyhaven.com/info/concrete_floor_worn_001 |
| Authors | Dimitrios Savva (photography), Rico Cilliers (processing) |
| License | CC0 1.0 Universal, per https://polyhaven.com/license |
| License text | `../../CC0-1.0.txt` (from https://creativecommons.org/publicdomain/zero/1.0/legalcode.txt) |
| Real-world size | 3.0 m x 3.0 m per texture tile (API `dimensions`: 3000 mm) |
| Resolution used | 1K JPEG |
| Retrieved | 2026-10-06, with `scripts/fetch-facility-textures.sh` |
| Changes | None. The files are stored and embedded byte for byte in `generated/facility.glb` and in the five concrete modules in `generated/modules/` (`floor_tile`, `floor_tile_marked`, `wall`, `wall_conduit`, `wall_doorway`). |

CC0 does not require attribution. We record the authors for provenance.

## Files

| File | Source URL | MD5 (matches Poly Haven API) | SHA-256 |
| --- | --- | --- | --- |
| `concrete_floor_worn_001_diff_1k.jpg` | https://dl.polyhaven.org/file/ph-assets/Textures/jpg/1k/concrete_floor_worn_001/concrete_floor_worn_001_diff_1k.jpg | `e35597cca586150b1ab2aa9a331a39c5` | `6e40c0fc908f4d66431836f5abe203f3a4eb064e88824378a91a46a26e2f464c` |
| `concrete_floor_worn_001_nor_gl_1k.jpg` | https://dl.polyhaven.org/file/ph-assets/Textures/jpg/1k/concrete_floor_worn_001/concrete_floor_worn_001_nor_gl_1k.jpg | `9de6626758f8793b71182b892c110827` | `523bd94a1be5dd2f11c2ae2b7d7c77445cf5bd9e41604db515a7c8364dcba067` |
| `concrete_floor_worn_001_arm_1k.jpg` | https://dl.polyhaven.org/file/ph-assets/Textures/jpg/1k/concrete_floor_worn_001/concrete_floor_worn_001_arm_1k.jpg | `8174fe8a5c982edef079c3c3775e5377` | `b21e63637d04b969894bd40a45df185d77671ed74b8955cd6e403c34ed465043` |

## Map use

- `diff`: sRGB base color. Each material multiplies it by its own
  `baseColorFactor` (floor, wall, and lower-wall paint tints).
- `nor_gl`: OpenGL (+Y) tangent-space normal map, which matches glTF and Bevy.
- `arm`: R = AO, G = roughness, B = metallic. This is the glTF
  `metallicRoughnessTexture` channel layout. The GLB does not use the R channel
  as `occlusionTexture`.
