# Visual direction: the facility

Research and candidate references, not approved production assets. The only
third-party files in this repository are three CC0 Poly Haven concrete maps
under `sources/`; see `PROVENANCE.md`. Check the license on each download
before adding it to `assets/`; keep the source URL, author, license version,
changes, and required credits with the asset.

## Procedural facility PoC

`generated/facility.glb` is an original modular corridor and room, made in
headless Blender from the Nix dev shell. `generated/modules/` has the same
module family as separate GLB files, with original props and decorations.
`scripts/generate-facility.sh` writes the kit; the `facility_layout` example
composes a multi-room layout from it in Bevy. The committed
`screenshots/facility_layout_*.png` images are the baseline layout. The
earlier wayfinding study is in `screenshots/facility_wayfinding_*.png`.
The current example captures `screenshots/facility_floor1_v2_*.png`. Earlier
`screenshots/facility_floor1_*.png` captures preserve the previous revision:
it had a narrow cross and four isolated ceiling/wall blocks at its sides, an
EXIT door directly on service, and a larger top-right office. The current
plan has an open reception with a desk, from which narrow west and east halls
loop through intake and the damaged lab. The north EXIT is a distinct
objective room with a nonfunctional visual fuse-panel prototype. It opens
internally to service, the relocated office, and the smaller top-right
security room. Red-lit middle-right storage connects via the east hall, not
directly to office or security. A utility room sits left-middle and a solo
locker hiding room sits lower-right. Boiler remains bottom-left with one
entrance. Wall signs and double-sided EXIT hangers follow actual routes.
Authored colored floor-route fixtures remain hidden pending a separate design
pass; their generic modules remain available.
The boiler, disturbed maintenance island, broken containment tank (a
first-encounter concept, not implemented), and hiding-space silhouettes
(visual concepts only, no mechanics) remain. The Quaternius MegaKit is a
reference only. See `generated/README.md` for the grid, the axes, the module
sizes, the snap rules, and the Bevy cameras and lights.

## A look worth testing

A closed industrial facility, not a black maze: painted concrete, steel doors,
conduit, worn floors, and a few functional lights. Give each area a readable
identity: a cool, orderly intake corridor; an amber maintenance room; a nearly
unlit service wing with one red fault lamp. Reuse the same architecture, but
change signs, damage, props, and sound so the player can navigate by memory.
Make the escape door visible early. Hide what is behind a doorway, not the
doorway itself.

### Images, surfaces, and reference boards

| Reference | Use | Rights / caveat |
| --- | --- | --- |
| [Poly Haven: worn concrete floor](https://polyhaven.com/a/concrete_floor_worn_001) | Color, stains, scale and PBR material for traffic routes | [Poly Haven license](https://polyhaven.com/license): CC0. Start at 1K or 2K, not 16K. |
| [Poly Haven: concrete textures](https://polyhaven.com/textures/concrete) | Compare walls and floor finishes; select one clean and one damaged variant | CC0 per site license; confirm the selected asset page. |
| [Poly Haven: HDRIs](https://polyhaven.com/hdris) | Reference for indirect illumination and material tests, not an excuse to light the final indoor scene evenly | CC0 per site license; indoor look may need custom lights. |
| [What Happened Here? - Smith and Worch](https://www.worch.com/files/gdc/What_Happened_Here_Web_Notes.pdf) | Environmental-storytelling reference: let a room's objects imply an event instead of spelling it out | Reading reference, not reusable art. |

### Models to inspect, not yet imported

| Candidate | Best fit | Rights / adaptation |
| --- | --- | --- |
| [Quaternius: Modular Sci-Fi MegaKit](https://quaternius.com/packs/modularscifimegakit.html) | Corridor, junction, door and room blockout | Creator lists CC0; check the free download contents. Sci-fi styling may need restrained colors and industrial retexturing. |
| [Kenney: Factory Kit](https://kenney.nl/assets/factory-kit) | Machinery and storage-room dressing | [Kenney license](https://kenney.nl/docs/license): CC0. Check export formats and scale inside the downloaded kit. |
| [Kenney: Mini Dungeon](https://kenney.nl/assets/mini-dungeon) | Placeholder collectible silhouette | CC0; medieval shapes are a poor final match for a facility. |
| [Quaternius: Key](https://poly.pizza/m/h5nke04hRD) | Temporary physical key pickup | Listing says CC0 and offers glTF; this is a third-party listing, so confirm origin/license when downloading. A facility keycard or fuse might communicate setting better. |
| [Quaternius: Ultimate Monsters](https://quaternius.com/packs/ultimatemonsters.html) | Rigged placeholder for pursuit tests | Creator lists CC0; stylized design may undermine a grounded visual tone. |
| [Purple.Point: Horror Humanoid Creature](https://sketchfab.com/3d-models/horror-humanoid-creature-b5874b20f8c34b919a52eb3cb7dad94c) | Possible final pursuer silhouette | Sketchfab lists CC Attribution. Confirm the exact license version, download availability, rig/animations and author before use; credit required. Reported high-poly model (~96K triangles) may need reduction. |

Do not adopt a model because its thumbnail looks good. Check its silhouette from the player's typical viewing distance, animation quality, polycount, materials, and author/license trail. Avoid relying on unclear license labels or AI-generated models without a separate provenance review.

## Lighting experiment

1. Block out a T-junction, a room with two exits, and one long corridor in plain materials. Give the player a reliable way to see the next decision point (signage, floor marking, emergency lamp). Keep peripheral space uncertain.
2. Establish a hierarchy: dim base illumination for navigation, isolated practical lights for points of interest, and a movable flashlight for inspection. Let darkness conceal detail, not required interaction prompts or the only route.
3. Place a dim light *behind* a half-open door or around a turn; this implies occupied space before the player can confirm it. Repeat the motif once with no threat, then alter it later. Use silhouettes and partial occlusion before a full monster reveal.
4. Compare still frames at player eye height with and without the flashlight. If signage, collectible outlines, or the escape route disappear completely, add bounce/fill or contrast rather than only increasing flashlight brightness.
5. Test limited fog for depth and short sightlines. Avoid uniform fog and constant flicker; both flatten clues and can harm accessibility. Offer reduced-flash settings if using flashes or strobes.

These are art-direction hypotheses, not fixed Bevy light or performance settings. Measure shadow-light cost on the target hardware. Prefer glTF/GLB exports when available, inspect material/normal-map conventions in-engine, and make a small playable lighting study before committing to a kit. Lighting is also a gameplay promise: an illuminated exit should be reachable, even if it is locked.
