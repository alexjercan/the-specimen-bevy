# Prototype a boiler-room power outage event

- STATUS: CLOSED
- PRIORITY: 55
- TAGS: backlog, gameplay

## Delivery / research

Research pacing/trigger conditions and an understandable route to boiler recovery before implementation. Prototype a bounded/randomized blackout with deterministic seed or transport-test override, an interactive boiler reset, and readable backup cues. Ensure no softlock or unavoidable darkness; test reset and save/restart behavior. Depends on lighting and objective flow.

## Status

Implemented: a seeded 30-90 second delay triggers an outage during active play. F at the boiler restores power and starts another seeded delay. Normal point lights and their emissive materials switch off during an outage; red emergency fixtures, EXIT signs, boiler fire, and the player flashlight remain independent. Boiler repair uses shared interaction targeting and a message-based processing system to avoid the prior observer query conflict. The approved power-down and restart cues are wired; restart playback is attached to the boiler source entity.

Focused repeat-cycle, light/material, audio, and menu-rebuild tests passed. Menu rebuild constructs a fresh level and power state. No persistence/save system exists, so save/restart behavior cannot be tested yet. No in-game lighting or audio listening review has been performed. Breaker-trip/reset candidates are separate review-only research.
