# Define the playable facility loop and objective pacing

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: backlog, design

## Delivery / research

Research + spike: document the start, three-fuse search, panel activation, locked EXIT, failure/retry and win flow. Check route lengths and optional paths in the existing floor. Decide fuse placements, gating feedback, and what the player knows before implementing. Done when a playtestable walkthrough and acceptance cases are recorded.

## User facts (2026-10-07)

- Fuse pickups go on tables; outline each fuse itself, not its table.
- Author 5-6 eligible table locations. Each run uses a seed to select exactly 3 distinct locations and spawns one fuse at each. The same seed must reproduce the same selection.
- F picks up an aimed fuse using the existing on-screen interaction call-to-action hint (for example, `F PICK UP FUSE`). Preserve unambiguous targeting when a door is also in range.
- Show three fuse slots in a bottom corner of the UI: empty slots are dark fuse outlines; each collected fuse fills one slot with a fuse image. The exact corner is not decided.
- These are design requirements, not implemented behavior. Research first; do not finalize the candidate table set without checking routes, clearance, and visibility.

## Agent findings (source inspection, not visual verification)

- The immediate-mode first floor starts in intake at about (0, -5) and has 18 rooms. The EXIT objective room is (-1.25..8.75, -31.25..-21.25), with the outside door at (0, -31.25); the existing `fuse_panel` is a decorative wall-mounted GLB at (5, 1.85, -31.15). `DoorState` is only Closed/Open and the current F interaction toggles an aimed panel. Locking requires an explicit new game rule, not just an art change. Sources: `crates/gameplay/src/levels/first_floor_builder/{rooms,lights}.rs`, `crates/gameplay/src/levels/{builder,doors}.rs`.
- Inspected table-like placements include the maintenance concept table (-7.5, -22.9), east-side hiding-room concept table (9, 0), office concept table (3.75, -18.75), and reception table (-2.5, -10). These are candidates only, NOT an approved pool of 5-6. Reception is close to intake and busy with papers; the maintenance work island at (-7.5, -20) and workbenches carry built-in clutter. Find enough suitable tables, or explicitly author more, then inspect access, clearance, and visibility before selecting the 5-6 eligible locations. Source: `crates/gameplay/src/levels/first_floor_builder/props.rs`.
- The generated `concept_table` has a top surface at 0.8 m and the `work_island` at 0.92 m; the `workbench` top is 0.9 m with a 1.5 m back panel and small clutter. These measurements come from `scripts/generate_facility.py`, not a rendered visual review. The existing manifest lists table assets but has no fuse pickup model. A small original fuse/holder mesh and its dimensions, provenance, generated/source and shipped manifests would be needed before visual implementation.
- Existing `LevelRenderPlugin` spawns a `WorldAssetRoot` child for `Prop` after `FacilityAssets` are ready. Gameplay must keep the fuse identity and pickup state on its own entity independent of GLB descendants, so headless transport still works. The F action, short-range analytic door targeting, and fixed screen-position hint are starting points, but current targeting only knows doors; fuse interaction needs a common nearest-visible choice, distance/line-of-sight checks, and unambiguous hint priority when a fuse and door overlap. Source: `crates/gameplay/src/levels/render.rs`, `crates/gameplay/src/levels/doors.rs`, `crates/core/src/glue/door_hint.rs`.
- No outline dependency or outline implementation was found in the current Cargo files or crates. Candidate visual spike A: a thin slightly enlarged dark/bright silhouette shell for the fuse mesh, depth tested (not x-ray), visible at useful distance; candidate B: emissive edge/rim shader on a dedicated pickup mesh. These are alternatives to test under dim/red light, not confirmed Bevy 0.19-compatible solutions. Post-process object outlines need another pass and may cost more; do not select a plugin without a compatibility/performance test. Never outline the whole table when only the fuse is collectible.

## Proposed pacing spike (unapproved)

1. Start at intake; show the need for three fuses through the panel/EXIT and restrained clues, not a global quest marker.
2. Build a vetted pool of 5-6 table locations spread across the level. Select 3 distinct locations from the run seed, not 3 fixed locations. Check that every possible selection is reachable and makes a reasonable route. Make each fuse's outline visible in low light at a useful distance, but test whether it should remain occluded by walls and closed doors. F pickup should only work when aimed at a reachable fuse, not at its table through a wall. Reuse the center-screen interaction hint, and show collection progress in three bottom-corner fuse slots.
3. Return to the fuse panel and insert all three. Until then, the outside EXIT remains physically closed and provides clear feedback. When powered, unlock it; traversing the outside EXIT completes the run. Do not treat this as an implemented state machine.
4. Spike two silhouette styles and table placements in a dark/red-lit and flashlight-off view. Measure recognition, accidental table/door interaction conflicts, route detours, clearance around the table and whether objectives are missed from the doorway.

## Decisions to review before implementation

- Finalize the 5-6 eligible tables and verify every possible three-location combination for reachability, route quality, and prop collision. Seed source and how players enter/share a seed still need a decision; the selection itself must be reproducible.
- Always-on outline versus proximity/line-of-sight reveal; whether the pickup is visible through occlusion (recommend not). Color/width/flicker must remain readable against planned dimmer lighting and accessible without color alone. Pick the bottom-left or bottom-right corner for the three-slot HUD, and check legibility at the game's target resolution.
- Should the panel require manual insertion after collecting three fuses, and what is the missing-fuse/locked-door feedback? The three-fuse panel interaction remains a proposal.

## Verification / done when

- For the research phase: report the candidate pool, technical trade-offs and open decisions for owner review here. Later acceptance cases should cover same-seed repeatability, exactly 3 distinct pickups from 5-6 eligible tables, fuse-only outline, F hint/target arbitration with doors, and HUD progress from 0/3 through 3/3 in rendered and headless modes as applicable. No code implementation or visual playtest is claimed. Full task stays OPEN until a playable walkthrough and acceptance cases are recorded.
