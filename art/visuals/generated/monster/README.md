# Human Deer animation prototype (task #227)

Status: prototype for review. Not shipped. Not in `credits/`. Used only by the review example
`examples/human_deer_viewer.rs`, not gameplay. The example starts on IDLE;
Left/Right arrows (or < and >) cycle clips and show the selected name on screen.

## Attribution

This work is based on "The Human Deer"
(https://sketchfab.com/3d-models/the-human-deer-7644694337404bb18eb68e6b637740a1)
by ceeleste (https://sketchfab.com/ceeleste), licensed under CC-BY-4.0
(http://creativecommons.org/licenses/by/4.0/).

Changes from the original: new WALK, CHASE and ATTACK clips on the original
101-joint skeleton. The original clip "Take 001" is renamed IDLE and
resampled to 24 Hz with every joint keyed. Mesh, skin weights, materials and
textures are not changed.

The license statement comes from the `license.txt` file in the downloaded
archive. Nobody has verified it independently against the Sketchfab page.
The source copy and its provenance record are in
`art/visuals/sources/sketchfab/the_human_deer/` (task #226).

## Files

- `human_deer_animated.glb`: skinned model with embedded textures and four
  named clips. The 12 embedded PNG files are byte-identical to the source.
- `human_deer_animated.manifest.json`: source and output SHA-256, Blender
  version, clip facts and validation report.
- `previews/<clip>.mp4`: side view and three-quarter front view, rendered
  from the exported GLB after a fresh import.
- `previews/<clip>_sheet.png`: 8 frames per view.

## Clips

| Clip | Frames at 24 fps | Length | Loop | Notes |
| --- | --- | --- | --- | --- |
| IDLE | 0-150 | 6.25 s | yes | Original "Take 001". Feet planted. Head, jaw, arms and hip sway. |
| WALK | 0-36 | 1.5 s | yes | Hunched walk on the hind legs. Arms swing opposite to the legs. Tail and tongue sway. |
| CHASE | 0-16 | 0.667 s | yes | Four-limb transverse gallop. Body crouches, hands plant on the ground, jaw open. |
| ATTACK | 0-40 | 1.667 s | no | Rear up with arms raised in a V and jaw open, then lunge-strike, then recover to the base pose. |

All clips are in place. The root joint does not move. The base pose of WALK,
CHASE and ATTACK is IDLE frame 0. Looping clips have identical first and last
keys.

To stop foot sliding, gameplay must move the entity at the in-place speed
multiplied by the entity scale:

| Clip | Raw model units | At viewer scale 0.42 |
| --- | --- | --- |
| WALK | 1.222 m/s | 0.51 m/s |
| CHASE | 7.5 m/s | 3.15 m/s |

ATTACK: the right arm reaches the strike pose at frame 19 (0.79 s). The left
arm is 2 frames later. A possible hit window is 0.75 s to 0.9 s. This is a
suggestion, not a gameplay decision.

## Method

`scripts/animate_human_deer.py` runs in Blender. It imports the archive,
reads the evaluated IDLE frame-0 pose, and computes every frame as follows:

- Pelvis, spine, neck, head, jaw, tongue and all 16 tail joints get offsets
  about the body axes.
- Hind legs use analytic two-bone IK (thigh and shin) on the real joint
  positions. The bend plane comes from the rest geometry of each leg. The
  metatarsal and the foot are aimed after the IK. Stance feet stay fixed on
  the ground. A toes-down foot pivots on the toe.
- CHASE hands use the same IK on the arms, with a ground clamp. Finger joints
  stay at least 5 cm above the ground.
- WALK and ATTACK arms use rotations about the body axes. Finger curl uses
  the palm plane of each hand.
- Two unskinned source meshes get node keys in the new clips. The exposed
  right-leg bone follows `R_upper_leg_J_078` (the source IDLE keys it the
  same way). Hair card `pPlane19` follows `C_Hip_J_00`.

Regenerate from the repository root:

```sh
nix develop -c blender -b --python scripts/animate_human_deer.py -- \
  --source /path/to/the_human_deer.zip --previews
```

`--source` accepts the archive or an extracted directory. `--out` defaults
to this directory. Two runs gave byte-identical GLB output (SHA-256
`a9f271b6157a85b6c4b9c9da500975e5f8ce4ecc85a799ab51aa970b87c252ce`).

## Validation (Blender 5.2.2)

The script stops with an error if a check fails.

- 101 joints. Every clip keys all 101 joints. All values are finite. All
  samplers use LINEAR interpolation.
- The clip set is exactly IDLE, WALK, CHASE and ATTACK. Durations are 6.25 s,
  1.5 s, 0.667 s and 1.667 s. In WALK, CHASE and ATTACK the first and last
  keys are identical.
- IDLE compared with the source clip on every exported channel, both ways.
  Joints that the source does not key are compared with the source node
  default. Max error: 0.15 deg rotation, 2.5 mm translation (raw units).
- Node, mesh, material, image and skin counts match the source (328, 72, 4,
  12, 1). Material alpha modes match. Embedded textures are byte-identical.
- Animated non-joint nodes are exactly the expected ones: IDLE keys
  `C_Hip_ctrl` and `R_leg_exposed_bone_low`. The other clips key
  `R_leg_exposed_bone_low` and `pPlane19`.
- No IK target is out of reach by more than 0.1 mm. No joint goes below the
  ground. Result: shortfall 0, lowest joint 3.2 cm above the ground
  (`R_toe_J_083`).
- The armature scale is uniform. Every action slot binds to an object.

Manual checks made during development, not in the script:

- The deformed IDLE vertices of the GLB match the source import within
  0.05 mm per matched mesh at frames 0, 37, 75 and 112.
- Close-up renders show no broken or twisted limbs. The grey quads near the
  antlers and the dark plate on the thigh also show in renders of the
  unmodified source.

## Limitations

- These are procedural prototype clips, not hand-keyed animation or motion
  capture. They were reviewed in Blender Eevee renders. The Bevy review
  example loads this GLB, but animation quality in Bevy still needs visual review.
- The GLB rest pose (node defaults) is the source bind pose, not the source
  scene default pose. For some unweighted helper joints the difference is
  large (up to 180 deg, 6 m). Skinning is exact. Every clip keys all joints,
  so a playing clip always sets the full pose. With no clip playing, the
  model shows its bind (modeling) pose.
- IDLE is resampled at 24 Hz. The source keys are on an irregular 120 Hz
  grid. The deviation is in the validation numbers above.
- IDLE does not key `pPlane19` (the source does not). After another clip,
  this 4 cm hair card keeps its last offset until something resets it.
  `C_Hip_ctrl` (an empty control node) is keyed only in IDLE.
- The ground checks use joint positions, not the mesh. Sole and claw tips
  can sink a few centimetres.
- There is no collision check between limbs and the body. In ATTACK the
  raised arms pass near the antlers.
- The GLB is about 30 MB, because the textures are embedded.
