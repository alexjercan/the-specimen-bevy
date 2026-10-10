# Persist achievements locally, sync Steam optionally, and add in-game viewer

- STATUS: OPEN
- PRIORITY: 0
- TAGS: backlog

## User facts

- Players on itch and in the browser need game achievements without Steam.
- Steam builds should also sync achievements through Steam when available.
- Show an Achievements viewer in the game, reached from a button below Credits in the main menu.
- This is follow-up work; do not implement it as part of the current flashbang fix.

## Agent findings

- Gameplay now tracks seven achievement IDs in memory in `crates/gameplay/src/achievements.rs`; Steamworks integration and persistent storage do not exist yet.
- The existing main menu and credits scene are in `crates/core/src/menu/`.

## Delivery

- Persist unlocked IDs in an application-specific user data directory for native non-Steam play and browser storage for WASM; choose exact storage formats and migration behavior during implementation.
- Provide an optional Steamworks adapter on supported native Steam launches. Detect availability through successful Steamworks initialization rather than requiring users to set an environment variable; allow safe fallback when Steam is absent.
- Reconcile local unlocks and Steam unlocks without discarding offline progress, and keep the viewer backed by the unified in-game progress.
- Add the Achievements button below Credits and an in-game viewer that works without Steam.

## Verification

- Native non-Steam and browser builds retain unlocked IDs across restarts; missing or corrupt storage does not prevent play.
- Steam unavailable does not block launch. Steam available syncs unlocks both ways, including unlocks earned offline.
- Main menu viewer displays the same unlocked state as gameplay on all platforms.

## Done when

- Persisted progress, optional Steam synchronization, and viewer are implemented and tested on supported targets. Steamworks configuration and achievement IDs are documented. No Steamworks publication is implied by completing local code.
