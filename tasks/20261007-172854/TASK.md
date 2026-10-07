# Research and prototype darker facility lighting

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: backlog, visuals

## Delivery / research

Research + visual spike: compare dimmer/removed practical lights and local contrast against the approved dark/red room baseline. Check sign, door and route readability with and without a flashlight; avoid a uniform brightness reduction. Keep a reversible lighting variant and review captures. Done when settings and screenshots are reviewed; do not assume runtime appearance from tests.

## Research and reversible review variant

Source inspection: `first_floor_builder/lights.rs` has cool ceiling point lights at 90,000 intensity, amber lights at 110,000-140,000, a 60,000 red storage light, and a 6,000 EXIT light. `builder.rs::light` sets shadow maps off; `render.rs::animate_lights` uses `LightIntensity` as the base value for flicker/pulse. `examples/facility_layout.rs` also adds artificial overhead 900,000 light and ambient brightness 20 for a ceiling-free top-down review image; this is NOT the game's actual eye-level lighting.

A reversible `--dark` variant in the review example cuts its overhead light to 180,000 and ambient brightness to 6, and multiplies authored point-light base intensity by 0.45. It writes a distinct `art/visuals/screenshots/facility_lighting_dark.png`, leaving the standard screenshot path and main game unchanged. These values are hypotheses, not tuned settings. Compare both captures plus an eye-level game view after the user approves running examples. Check fuse outlines, EXIT signs, door panels, route lines and paths in red storage and unlit corridors. No captured result or visual conclusion yet.

Custom shading is not the first lever: existing StandardMaterial GLBs already use emissive lamps/signs and light animation. First compare spatially selective practical-light intensity/range, color, and retained emissive signs. A custom full-screen grade could lower midtones while preserving highlights, but requires HDR/tonemapping/color-space review and may suppress fuse outlines/UI or create dark crushed surfaces; defer until paired captures demonstrate a deficiency that stock lighting cannot solve. Do not add a shader to the main game on source-only evidence.

## Status

The user reviewed the `--dark` example and preferred it to the brighter variant. A separate main-game adaptation is now authored in `crates/core/src/lib.rs`: default windowed GamePlugin/MenuPlugin composition uses ambient brightness 6.0 and scales newly added authored `LightIntensity`/`PointLight` pairs by 0.45 once. Headless, rendered transport, and custom-plugin examples are excluded. The overhead review light is not copied into the game. The one-shot regression test is in `crates/core/tests/unit/dark_lighting.rs`.

The main-game edit and test have not been compiled or run. Eye-level main-game readability with and without a flashlight cannot be judged yet (flashlight implementation is separate). Keep this task OPEN pending code checks and visual review.
