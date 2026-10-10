# Draft Steam achievement definitions, icons, and preview

- STATUS: CLOSED
- PRIORITY: 0
- TAGS: backlog

## User facts
- Plan seven Steam achievements. "Buy some time" must require that the flashbang hits the monster, not just that it was thrown while being pursued.
- Create `art/steam/achievements.html`, preview tiles and icons. Do not implement game mechanics or Steamworks code yet. The owner corrected the destination from `art/content/` to `art/steam/`.

## Decisions
- Automode (reversible): create separate unlocked and locked PNG icons for Steamworks plus an HTML preview using local assets.
- Keep achievement names and conditions explicit; do not imply that current flashbang mechanics detect a monster hit.

## Delivery
- Achievement list with event/eligibility rules and stable proposed API IDs.
- Locally viewable `art/steam/achievements.html` and icons in `art/steam/achievements/`.
- Mark future gameplay and Steamworks integration work as pending, not shipped.

## Planned unlock conditions and icon concepts

| Proposed Steam API ID | Trigger / disqualifier | Icon motif |
| --- | --- | --- |
| `ESCAPE_WITHOUT_FLASHBANG` | Escape; never picked up a flashbang in this run. | Crossed-out flashbang. |
| `ESCAPE_WITHOUT_DETECTOR` | Escape; never picked up the detector. | Crossed-out ear. |
| `FLASHBANG_HIT_MONSTER` | Confirm a monster hit from the flashbang burst. A throw or player-only concealment does not count; the hit event and gameplay effect need design and implementation. | Simple explosion star. |
| `ESCAPE_WITHOUT_BOILER` | Escape; no completed boiler repair in this run. | Crossed-out lightbulb. |
| `RESTORE_BOILER` | Complete a boiler repair and restore power. | Lightbulb with three rays above it. |
| `CAUGHT_AFTER_EXIT_OPEN` | Actually open the unlocked exit door, then get caught before escaping. | Open door. |
| `ESCAPE_UNDETECTED` | Escape with no monster pursuit entered from sight or sound in this run. | Crossed-out eye. |

The SVG sources and separate 256x256 locked/unlocked PNG drafts live in `art/steam/achievements/`. `render.py` reproduces the PNGs using Inkscape. The HTML previews both states; API IDs and art are drafts, not Steamworks configuration. After owner feedback, the detector motif is a crossed-out ear, the flashbang hit is a symmetric explosion star, the boiler pair uses crossed-out/lit lightbulbs, and the exit motif is an open door.

## Verification
- All seven definitions and 14 icon variants exist in `art/steam/achievements.html` and `art/steam/achievements/`. All eight local HTML references resolve; all PNGs are 256x256.
- Chromium rendered the preview from its final `art/steam/` path. The unlocked desktop preview was inspected at 1280x1200.
- No gameplay files changed; no upload or unlock implemented. Keep status OPEN for owner art review and future gameplay/Steamworks integration.

## Done when
- HTML and icon drafts exist and are usable for visual review; keep this task OPEN for owner art review and later gameplay/Steamworks integration.
