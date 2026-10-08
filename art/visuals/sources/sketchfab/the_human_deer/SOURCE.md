# Sketchfab: The Human Deer

| Field | Value |
| --- | --- |
| Model page | https://sketchfab.com/3d-models/the-human-deer-7644694337404bb18eb68e6b637740a1 |
| Author | ceeleste (https://sketchfab.com/ceeleste) |
| License | CC-BY-4.0 (http://creativecommons.org/licenses/by/4.0/), as stated in the bundled `license.txt` and in the glTF `asset.extras` fields |
| Local archive | `/home/alex/Downloads/the_human_deer.zip` |
| Archive SHA-256 | `3e5d58e4c9be7fa61721eae9800c73265cdf32b66dd83ef4b53b55d8ac6d2b28` |
| Archive entry dates | 2026-08-05 (per the zip directory) |
| Retrieved | 2026-10-09, extracted by hand from the local archive above |
| Changes | None. All 15 files are stored byte for byte as extracted from the archive. |
| Role | Review only. Used by the `human_deer_viewer` example to inspect the model. Not shipped in a game build, not under `assets/`. |

The license and rights above are as stated in the source archive. They were
not independently verified against the Sketchfab listing.

## Required credit

Quoted exactly from the bundled `license.txt`:

> This work is based on "The Human Deer" (https://sketchfab.com/3d-models/the-human-deer-7644694337404bb18eb68e6b637740a1) by ceeleste (https://sketchfab.com/ceeleste) licensed under CC-BY-4.0 (http://creativecommons.org/licenses/by/4.0/)

## Files

| File | SHA-256 |
| --- | --- |
| `license.txt` | `93a7eb87a804ee39efcb10dcb37460fd677cc7f597f01338becb46fa8b7867de` |
| `scene.gltf` | `2acd3bba4d8a3c040213edb2a3f3f7199ebf32e28d3c93278953669f3dea3e3f` |
| `scene.bin` | `855bbb210b127eb2bde2c4474cb7fef20dcf6e13ec2c9c9ba29435f10d9ab724` |
| `textures/Deer_body1_baseColor.png` | `9ff03ef50c6347072d862a1d85b6f3b806f0819acc1cafd5bdc9940925c9b250` |
| `textures/Deer_body1_emissive.png` | `bbabd0e13a9c34dc8d87bb76225979367dc64f08ba968a1007868a00081efba3` |
| `textures/Deer_body1_metallicRoughness.png` | `1a11f92b2111fef2d8dfb10fd1da23d5304a9fc09d9a35fb64c1de4eee1fa7cb` |
| `textures/Deer_body1_normal.png` | `2ee499825fd87117e5762f2172981491ab6cc4b360e465fb5da056a7098adea8` |
| `textures/Deer_head1_baseColor.png` | `85a4f752c64f4ba213241b05f529a5c92882cc9458d0cdfade7e29956bf106ba` |
| `textures/Deer_head1_emissive.png` | `0fcecdbd0a0343185f4546e3f3ac81dcb0ed4d78e4f67e7047046a7d863c46ac` |
| `textures/Deer_head1_metallicRoughness.png` | `f9a6bf943d60cdb9ec43ce2fda14e8015fba7ab31f57efeee76193fc39cfdd76` |
| `textures/Deer_head1_normal.png` | `58960dcf10403e67227f0f9473c40943d13e59d2c2846e367ea03f3e27400a43` |
| `textures/Hair_cards_baseColor.png` | `9046c55895727ba9b380f0b5ec833af3c85f78cec3ea574a272f3027d2b55bf4` |
| `textures/hair_cards_v2_baseColor.png` | `d4978858cf5038418ab4d919c0241b5293766b4d495d33ef4cb6b38eb0233c49` |
| `textures/hair_cards_v2_metallicRoughness.png` | `9ec538ba954132ce8388442c5488b54d945d8c2dd8625b889a9e2147b672ed31` |
| `textures/hair_cards_v2_normal.png` | `92ad4c6dd5277fcda2192d27631e561090cb2d929314645ed8515329f29b4589` |

## glTF facts

- Exported by Sketchfab-0.3.0. One scene, 328 nodes, 72 meshes, 1 skin, 4
  materials.
- One animation, `Take 001`, with 126 channels.
- `extensionsUsed`: `KHR_materials_emissive_strength`.
- Root node `Sketchfab_model` carries a Z-up to Y-up matrix with scale
  0.054. Its child, the `.fbx` node, carries a further scale of 0.01.
- Measured raw height, at identity root scale/offset, from the joint
  `GlobalTransform`s in Bevy (`human_deer_viewer`, one sample early in
  `Take 001`): min y = -4.596 m, max y = 0.139 m, height = 4.734 m. The
  combination of the two glTF scale factors above does not by itself
  predict this; the loaded and animated model was measured directly in
  Bevy, not computed from the raw matrices. Joint positions do not include
  the mesh surface, so the true mesh extent is slightly larger.
- The example logs joint bounds once, 30 frames after the joints appear.
  It logs the bounds after the fix-up below. To measure the raw bounds
  again, set `MODEL_SCALE` to 1.0 and `MODEL_Y_OFFSET` to 0.0.
- Applied fix-up in `human_deer_viewer.rs`: a root `Transform` with
  `scale = 0.42` and `translation.y = 1.93`, chosen from the raw
  measurement above to put the feet on `y = 0` and bring the creature to
  about 1.99 m tall (within the 1.5-3 m target). No rotation correction
  was needed; the model is already upright in Bevy's Y-up space.

## Run commands

```sh
nix develop -c rustfmt --check --edition 2021 examples/human_deer_viewer.rs
nix develop -c cargo check --example human_deer_viewer
nix develop -c timeout 300 xvfb-run -a cargo run --example human_deer_viewer -- --capture
nix develop -c cargo run --example human_deer_viewer
```
