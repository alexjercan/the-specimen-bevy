# Review facility layout, landmarks and objective routes

- STATUS: OPEN
- PRIORITY: 95
- TAGS: backlog, level-design

## Delivery / research

Research + layout spike: walk every route for three fuse positions, EXIT, boiler and hiding places; test multiple approaches and whether single-entry boiler/storage exceptions make objectives unfair. Review signage, room identity, density, clearance, sightlines and alternate escape paths; capture findings before moving walls or props.

## Source review (2026-10-07)

Research only: no level edits, tests, game run, or measured playtest. Findings below use authored topology and placement, not verified walking times, sightlines, clearance, or lighting. Keep this task OPEN for a playable route review.

- `crates/gameplay/src/levels/first_floor_builder/rooms.rs` defines 18 rooms with multiple routes between north and south. Reception/intake provide alternative central crossings. Utility has a west_hall-maintenance-utility loop; hiding has an east_hall-prep-hiding loop. Boiler and red-lit storage are single-entry dead ends, but neither is an eligible fuse room. Avoid putting mandatory objectives or an unavoidable chaser encounter in these dead ends without an escape design.
- `crates/gameplay/src/levels/first_floor_builder/fuse_tables.rs` lists five eligible furniture sites: hiding, office, maintenance, utility, security. `crates/gameplay/src/levels/fuses.rs` selects three distinct sites per seed. Source placement matches nearby tables/workbenches, but pickup reachability and furniture clearance need an in-game check.
- From intake, approximate shortest door hops to office/maintenance/utility/hiding/security are 3/3/3/3/4. Return hops from those rooms to the EXIT panel room are 1/2/3/3/1. Three interior EXIT-room doors are not locked; only the outside EXIT door is gated. The panel can be reached before collecting fuses.
- Additive out-and-back hop sums for the ten combinations (not actual multi-stop tour lengths): office+maintenance+security 14; office+maintenance+hiding 15; office+utility+security 15; office+hiding+security 15; office+maintenance+utility 15; maintenance+utility+security 16; maintenance+hiding+security 16; office+utility+hiding 16; maintenance+utility+hiding 17; utility+hiding+security 17. The 14-17 spread is a rough difficulty indicator; shared paths and search order may change the real ranking. Without hints identifying active rooms, a blind player may search all five sites regardless of seed.
- Door plaques identify maintenance, office and security, but no utility or hiding plaque asset exists. Utility is the clearest signage candidate; hiding may intentionally remain unmarked. Existing color accents distinguish boiler, storage and lab better than the five eligible fuse rooms. The HUD shows collected count, not active room locations.
- `art/visuals/generated/README.md` cites removed gameplay/example paths and claims no player collision. Historical captures do not establish the current rendered layout. Refresh its stale sections separately before using it as a route reference.

## Next review

1. Compare fixed seeds producing the easiest and hardest combinations, then sample all ten. Time the full search and return, not just shortest graph paths.
2. Check that interior EXIT-room approaches are open early, and confirm both utility and hiding loops are traversable at walking and running speeds.
3. Inspect all five fuse tables for sightline, outline visibility, interaction reach, clearance and distinct room identity. In particular, decide in play whether utility needs a plaque.
4. Inspect the intake starting view and boiler/storage dead ends for orientation and future chase fairness. Do not move geometry or add signs until these observations are reviewed.
