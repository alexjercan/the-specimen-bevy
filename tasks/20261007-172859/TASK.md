# Prototype a boiler-room power outage event

- STATUS: OPEN
- PRIORITY: 55
- TAGS: backlog, gameplay

## Delivery / research

Research pacing/trigger conditions and an understandable route to boiler recovery before implementation. Prototype a bounded/randomized blackout with deterministic seed or transport-test override, an interactive boiler reset, and readable backup cues. Ensure no softlock or unavoidable darkness; test reset and save/restart behavior. Depends on lighting and objective flow.

## Status

Implementation draft: a seeded 75-120 second delay triggers an outage during active play; F at the boiler restores power and starts the next seeded delay. Normal point lights and their glow are disabled during outage; the red wall lamp, EXIT sign, boiler fire, and flashlight remain independent. The shared interaction target handles boiler line of sight and prompt arbitration. Generated outage/repair sounds are review candidates only and are not in the game. A user-reported B0002 startup panic from conflicting FacilityPower system parameters led to a message-based repair path; this fix and the repeat-cycle tests have not been run. No runtime lighting or audio review is established.
