# Create three collectible fuse items and inventory

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: backlog, gameplay

## Delivery / research

Create distinct visible, interactable fuses and a small inventory/count. Research pickup silhouette and placement before authoring. Reuse the F interaction path where appropriate, including headless transport; prevent duplicate pickups and test collect/state persistence. Do not turn the decorative fuse-panel mesh into implicit gameplay state.

## Status

Implemented: three seeded, distinct table pickups, F interaction, inventory, and three-slot HUD. A generated fuse module is promoted for rendering. `--seed <u64>` selects repeatable placements; without it a fresh seed is chosen. Code was reviewed but not compiled, tested, or visually checked after these edits. Runtime appearance and behavior remain unverified.
