# Implement a rechargeable flashlight

- STATUS: CLOSED
- PRIORITY: 75
- TAGS: backlog, gameplay

## Delivery / research

Attach a spotlight to the player view. Left click toggles it (Automode reversible choice; F stays reserved for interaction). Charge drains while on and recharges while off. At zero it switches off, and cannot switch on again until it reaches a small threshold. Show a bottom-left charge bar opposite the fuse HUD; hide it only when full and off. Preserve headless and transport input parity, pause/menu gating, and a queryable player charge state. Test direction, drain, recharge, exhaustion, toggle, and UI state; review dark-room legibility in game separately.

## Status

Implementation drafted: gameplay Flashlight state and camera-child spotlight, shared enhanced-input left-click toggle, headless-safe drain/recharge, transport `input.flashlight` with persistent held semantics and player snapshot fields, and bottom-left game_ui charge meter through core glue. Provisional tuning: 30 s drain, 15 s recharge, 10% restart threshold, no shadows. Automode chose left click plus auto-off at zero; this was not personal user signoff. Regression tests now cover gameplay charge/toggle/pause/headless beam, UI bar behavior, and transport held-click snapshots. User's `cargo run --features debug` found a compile error: Bevy 0.19 `SpotLight` uses `shadow_maps_enabled`, not `shadows_enabled`. Corrected the field; a rerun is still needed. No commands, tests, formatting checks, or game/example runs were performed by the assistant under the standing restriction. Ask the user to run `nix develop -c cargo test -j 2 -p gameplay --test player_flashlight -p game_ui --test flashlight -p transport --test flashlight`, `nix develop -c cargo check -j 2 --workspace --all-targets`, and `nix develop -c cargo fmt --all -- --check`, then review the beam/bar in a dark room before closing.
