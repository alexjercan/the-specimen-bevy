# Gate the EXIT door behind the fuse panel and finish the run

- STATUS: OPEN
- PRIORITY: 85
- TAGS: backlog, gameplay

## Delivery / research

Make the EXIT door remain locked until the player installs all three fuses at the panel. Provide feedback for missing/accepted fuses, preserve ordinary door animation/collision and make repeated activation safe. Define an explicit win condition when exiting and verify it in rendered and headless tests. Depends on fuse inventory and objective-flow design.

## Approved flow

Collect all three fuses. The wall fuse panel becomes interactable through F only when all three are held; one activation installs them and unlocks only the outside EXIT door. Crossing that door completes the run. Normal windowed play shows a simple completion screen with Main Menu and Quit; headless/transport preserve a queryable win state in the snapshot rather than exiting before it can be read. Keep ordinary door animation and collision unchanged. Tests must cover the locked door, panel activation once, and crossing the threshold. Do not couple objective targeting to player collision.

## Status

In progress. Implementation and tests landed uncommitted in the shared tree: ExitDoor/DoorLock on the outside door, logical FusePanel with InstallFuses, Escaped via ObjectivePlugin, GameState::Complete screen, and `won` in the transport snapshot. A read-only code review found no defects. Build, Clippy, fmt, tests, and a windowed/headless run are not yet verified.
