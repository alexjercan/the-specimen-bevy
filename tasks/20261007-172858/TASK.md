# Hide under tables and in lockers

- STATUS: CLOSED
- PRIORITY: 60
- TAGS: backlog, gameplay

## Delivery / research

Research visual and interaction affordances and choose specific authored hiding props. Implement enter/leave state, camera/controls and interaction behavior; test no accidental collision trapping and no interaction through walls. AI detection effects are deferred until monster work.

## Status

Hiding and the storage-locker fix are implemented in source and uncommitted. The worker's 19 headless hiding tests passed after the storage fix. The user checked the earlier version in-game and said it "looks good". The animation, 1.45 m locker eye, and later hiding-audio integration have not had in-game review.

## Notes

- Spots (`HidingSpot` on existing props in `crates/gameplay/src/levels/first_floor_builder/props.rs`): lockers at hiding (13.35, +/-2.2), storage (13.35, -12.5) and (9.5, -9.15), security (10.5, -24.15); tables at hiding (9, 0), office (3.75, -18.75), maintenance (-7.5, -22.9). Rejected: reception table (exit touches shelf bins), maintenance locker (-10, -16.65) (exit overlaps tipped chair).
- Open side of both GLBs is local -Z (Blender +Y). Locker eye is local (0, 1.45, 0) and faces the door grille. Table eye is (0, 0.45, 0). Exits are local (0, -0.95) for lockers and (0, -1.1) for tables, at the stored standing height.
- F targets: `InteractTarget::Hide` joins the nearest-hit and wall-sight arbitration with doors, fuses and the panel. While `Hidden` is present, `aimed` returns only `Leave`, so F cannot open doors or pick up fuses. Spot changes go through the `UseHidingSpot` message, so one F press does one action.
- `Hidden { spot, height, phase }` on the player (phase is `Entering`, `Hidden` or `Leaving`) holds the state for a later snapshot. Transport is unchanged.
- Transition: 0.4 s smoothstep along current pose -> exit point at hidden eye height -> target. Rotation slerps to the spot facing. F during a transition reverses it from the current pose. Look and movement are locked during a transition. When settled, look works and walk/run stay locked. A pause stops the transition (virtual time).
- HUD: "HIDE" over the spot. "LEAVE" at bottom center while hidden.
- Known limit: the enter glide from the player's position to the front exit point is not collision-checked, so the camera can briefly clip a nearby prop. Walls and final poses are safe.
- Deferred: monster detection and per-spot authored exits if a layout change needs them. Hiding audio is tracked separately in the sound task.
- Tests: `crates/gameplay/tests/hiding.rs` includes a regression that walks from the storage hall to both lockers, enters each with F, and leaves at its authored exit. All 19 hiding tests passed (2026-10-07); removing the (9.5, -9.15) tag made the regression fail at the F step. A later hiding-audio event regression was added but has not yet been run.
