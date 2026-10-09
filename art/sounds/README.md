# Sound direction: an inhabited empty building

## Library organization

`art/sounds/generated/` contains all selected synthetic cues and processed
recordings. `art/sounds/sources/` holds retained originals in `freesound/`
and `opengameart/`. Within `generated/`, audio is grouped by category and cue:
`generated/amb/furnace/burning.wav` is an example. `assets/sounds/` holds
runtime copies by cue. No `recorded/` or `review/` directory remains.

`python3 scripts/generate_sounds.py` reproduces approved original UI and
roomtone clips. `python3 scripts/generate_more_sounds.py` reproduces the four
approved UI fuse clips: slot-1, slot-2, slot-3 and complete. Slots play on
successive pickups; complete plays on panel installation. The old spatial
fuse-pickup and panel-install clips were removed to avoid overlapping cues.
`python3 scripts/generate_ambience_review.py` reproduces the approved
synthesized ambience, including `generated/amb/pressure/low.wav`.
`python3 scripts/render_selected_ambience.py` re-renders the selected
Freesound edits from their retained low-quality previews.
`python3 scripts/build_sound_catalog.py` builds `art/sounds/catalog.html`.
Yellow cards are catalog-only; other cards have runtime copies. ffmpeg is
needed for rendering previews and drawing OGG waveforms. Selected monster
source previews are in `sources/freesound/monster/`; `python3
scripts/render_monster_sounds.py` makes approved edits in `generated/monster/`.
Item links, hashes, licensing caveats, and cut choices are in
[`monster/README.md`](sources/freesound/monster/README.md). The selected
metal-floor recording and edited growl, heartbeat, and crawl clips have runtime
copies and approved catalog cards. Patrol currently plays the metal recording
as occasional presence and six crawl impacts as timed walking steps; detection,
attack, and chase audio are loaded for later behavior but not triggered yet.
Rejected previews remain off-catalog.

The rejected `_stubb` handle-rattle edit, synthetic locked-door rattle and
open-door stop are not in the catalog or runtime assets. DrFahrts's selected
locked-door take now plays on locked-door attempts; the shelbyshark alternative
is retained off-catalog:

| Edited review WAV | Freesound source and creator | Retained preview SHA-256 | Edit SHA-256 |
| --- | --- | --- | --- |
| `generated/candidates/door/locked/rattling-locked-door-shelbyshark.wav` | [Rattling Locked Door.wav](https://freesound.org/people/shelbyshark/sounds/513392/) by shelbyshark, preview `sources/freesound/door/locked/rattling-locked-door-shelbyshark.ogg` | `501c652dc56268c6443aa151d314f95893c41321580844125199ba2a4bd936da` | `83884e3936042d971290e9267a81475587c73e823f84c5406e4c80e317d31464` |
| `generated/door/locked-rattle.wav` (runtime copy: `assets/sounds/door/locked-rattle.wav`) | [doorknob rattle](https://freesound.org/people/DrFahrts/sounds/727791/) by DrFahrts, preview `sources/freesound/door/locked/doorknob-rattle-drfahrts.ogg` | `4c622088b3b96daa015c8d834733e43bdd07e5acf1f7d8408d4c763c6b392cb0` | `5cb6860a7f6e363070e656826d5929b5975cdfc74a5009e842a2004b7c3a5d04` |

Both item pages display CC0 1.0, but only low-quality previews were
available without login. The source claims and uploader ownership are not
independently verified. The WAVs are mono 48 kHz PCM with short fades, cut
from preview times 1.35-3.35 s and 1.65-3.65 s, respectively. The DrFahrts edit was approved by listening; its in-game mix has not been
reviewed. The shelbyshark crop has not been auditioned or approved. Generation scripts no
longer make rejected drafts.

## Flashlight review sounds

`python3 scripts/generate_flashlight_sounds.py` creates three original,
reproducible mono 48 kHz PCM candidates under
`generated/candidates/flashlight/`: `switch-on.wav`, `switch-off.wav`, and
`battery-empty.wav`. They are dry, short mechanical-click sketches; the
empty cue is two weak clicks. These drafts are retained off-catalog. Three
recorded switch previews were inspected; the preferred Ralph0o7 click is
selected for runtime playback on both manual switch-on and switch-off:

| File under `sources/freesound/flashlight/` | Uploader / item | Page description | Preview SHA-256 |
| --- | --- | --- | --- |
| `click-ralph0o7.ogg` | [Ralph0o7, 690300](https://freesound.org/people/Ralph0o7/sounds/690300/) | Flashlight switch clicked into a microphone; 0.19 s | `7f861f08917776b8e3fbca833a72f9ca406855db617dd871e74d224e019908c0` |
| `thumb-switch-lunardrive.ogg` | [Lunardrive, 48979](https://freesound.org/people/Lunardrive/sounds/48979/) | Rubber flashlight thumb switch on/off; 0.56 s | `f3a8728d745fa8eacbc809fb88a9c439d5d728148c9362a9052eebc762fc760d` |
| `spring-switch-eskildnp.ogg` | [EskildNP, 855455](https://freesound.org/people/EskildNP/sounds/855455/) | One on and three off switch sounds from an old flashlight; 8.15 s | `41478c69a366f2dbe10b2ce57383149fb718bb7ee91671f9bf6c82c49f8e6ad8` |

These are low-quality Freesound OGG previews. Each item page displayed a CC0
1.0 link when checked; original downloads and uploader provenance were not
verified. The user selected Ralph0o7's click and requested game integration. The same
preview is copied unchanged to `assets/sounds/flashlight/click-ralph0o7.ogg`
and wired to manual switch-on and switch-off, as the reversible Automode
choice. Automatic shutdown on depletion does not produce a mechanical click;
the empty-battery cue is undecided. Other recorded previews remain off-catalog.
Recharge has no continuous sound candidate: a constant recharge noise could
mask footsteps and facility ambience. Playback volume and mix still need
in-game listening review.

## Sprint exhaustion breathing

`art/sounds/sources/opengameart/self/breathing-tired-mikeask.wav` is the
original download of [Breathing Tired](https://opengameart.org/content/breathing-tired)
by mikeask. The item describes a person breathing tired after running and
labels the file CC0 1.0. SHA-256:
`08d9c9e15426f0cd889b63867a3bc68bafd985e8d39fd57f805d4372b23aeeca`.
It is a 3.17 s, stereo, 44.1 kHz PCM WAV. The user approved this recording for a one-shot on sprint exhaustion. A
byte-identical copy is required at `assets/sounds/self/breathing-tired-mikeask.wav`
for the runtime asset loader; the source remains in the approved catalog. The
license is a page-displayed claim; uploader provenance and recording rights
have not been independently verified. The in-game level and timing still need
listening review.

## Recorded player steps

GboxMikeFozzy's [Footsteps](https://opengameart.org/content/footsteps-0)
OpenGameArt page displays CC0 1.0. The author describes subway walking,
normalization and noise reduction. The three user-approved source takes are
unchanged OGGs in `art/sounds/sources/opengameart/step/subway/` and have
byte-identical game copies in `assets/sounds/step/subway/`:

| Art file | Original download | SHA-256 |
| --- | --- | --- |
| `subway-step-a.ogg` | `01-footstep_0.ogg` | `33c9bef5e8aeb1069455699a34a0c5e1ef1787fd3f61594b0859d7e6bb9f9dec` |
| `subway-step-b.ogg` | `02-footstep.ogg` | `f05396c807eb8eaecffb2a1ead611ec971c5a7914d1d081c91d7796e852692a2` |
| `subway-step-c.ogg` | `04-footstep.ogg` | `b373ff981e44d6f7ba2291225a4803078771029615ed81606136fdb9599906d5` |

The CC0 and authorship claims come from the source page, not an embedded
license. Tile surface and footwear were not verified. Run and walk vary step
cadence; both currently choose among these three takes.

## Recorded door and hiding edits

The approved door cues in `art/sounds/generated/door/` and approved
hiding cues in `art/sounds/generated/hiding/` use rubberduck's
[100 CC0 metal and wood SFX](https://opengameart.org/content/100-cc0-metal-and-wood-sfx),
whose OpenGameArt page displays CC0 1.0. Archive SHA-256:
`be6eba63b03409ac0c77787a956b1503a7c186403d04aef9725c52644a4b7878`.
Door source OGGs and its original edit script were removed earlier; obtaining
the archive and reconstructing those edits is necessary to rerender doors.
`scripts/render_hiding_sounds.py` retrieves the verified archive into a
temporary directory to regenerate hiding edits. The approved rendered hiding
file hashes are:

| Cue | Recording(s) | SHA-256 |
| --- | --- | --- |
| `locker/open.wav` | `metal_open_01.ogg` | `7d774a8385141209e9037d02d3a9b5bf76cd1a2f25aa30701119b66cbabf3171` |
| `locker/close.wav` | `metal_close_01.ogg` | `f205dec7851f16a6308fb4a2418ec08710e98b58d6fb2ee91433cd9583b4dd03` |
| `table/enter.wav` | `wood_squeak_01.ogg`, `wood_misc_01.ogg` | `16222b747bb69fb1d347bcda724c10474621fdaed38117abef3b8742715f30a7` |
| `table/leave.wav` | `wood_misc_02.ogg`, `wood_close_01.ogg` | `524adb9a3558265a45b522aa51ad8ea6f687bf1fef37da16d80a71af4004d9e7` |

These are metal and wood recordings, not independently confirmed locker or
cloth recordings. Preserve source-specific voluntary credits in
`credits/CREDITS.md`.

## Facility ambience

The approved synthesized cues are `generated/amb/roomtone/plain.wav`,
`generated/amb/roomtone/conduit.wav`, `generated/amb/boiler/tick.wav`,
`generated/amb/vent/hvac.wav`,
`generated/amb/tank/hum.wav`, `generated/amb/light/cool-buzz.wav`, and
`generated/amb/pressure/low.wav`. The low-pressure bed is approved for
runtime at restrained gain; test it on ordinary speakers as well as headphones.
The former `generated/amb/boiler/rumble.wav` and
`assets/sounds/amb/boiler/rumble.wav` remain on disk by decision, but the
runtime no longer loads the rumble; the boiler uses the furnace loop instead.

Three edited Freesound low-quality previews now have game copies:
`generated/amb/furnace/burning.wav` is a 24-second crossfaded boiler
burner loop, `generated/amb/water/faucet.wav` is a 5-second water burst
near the pipe manifold, and `generated/amb/vent/wind.wav` is a
low-passed 18-second vent-wind loop. The long preview sources are retained
under `sources/freesound/amb/` only to reproduce the edits, not listed in
the catalog and not shipped. Per-file source pages, preview hashes and edit
hashes are in `art/sounds/sources/freesound/README.md`; creator credits are
in `credits/CREDITS.md`. Item-page CC0 labels do not independently prove
recording ownership or original-file rights. Loop quality, gain and spatial
placement still need in-game listening review. Removed draft files are not
reproducible from current generation scripts.

## Boiler outage review candidates

`python3 scripts/generate_boiler_outage_sounds.py` creates four original,
reproducible mono 48 kHz PCM candidates under
`generated/candidates/amb/boiler/` for a prototyped boiler-room power-outage
event: `breaker-trip.wav` (0.22 s, a bright metallic snap over a low thunk,
for the fault that cuts power), `power-down.wav` (1.4 s, a descending
motor/fan hum with decaying blade-noise texture, for the boiler losing
power), `reset.wav` (0.42 s, a double mechanical clack, for pressing a
physical reset control), and `restart.wav` (2.3 s, igniter-style ticks
followed by a rising ignition whoosh that settles into a rumble tail, for
the boiler relighting). All four favor a recognizable mechanical character
(impacts, motor pitch, igniter clicks) over an abstract electronic tone, per
the task brief. They are deterministic synthesis with no recorded or
third-party material, so no source, credit, or license caveat applies.

The generated `breaker-trip.wav` was rejected and is off-catalog. The
OpenGameArt `switch off.wav` from CleytonKauffman's circuit-breaker pack is
approved for the boiler's outage cue. The retained source is at
`sources/opengameart/amb/boiler/switch-off-cleytonkauffman.wav`; an unchanged
copy is at `assets/sounds/amb/boiler/switch-off-cleytonkauffman.wav` (both
SHA-256 `42c6be41a0082d602b0af647bac7c2009b4ff2b6228d6a1ed3c21d76290bd1a8`).
The item page labels it CC0 and asks for `SFX by Cleyton Kauffman -
https://soundcloud.com/cleytonkauffman`. The ZIP has no bundled provenance or
license; ownership and recording method are not independently verified. The
breaker trip plays at the boiler alongside the approved global power-down cue.
The generated `reset.wav` plays from the boiler on repair alongside the
previously approved restart cue. Actual in-game playback and mix need review.

## Compose dread rather than continuous music

Build a quiet baseline from facility sounds: HVAC, fluorescent buzz, distant
pumps, ventilation rattle. Record your own room tone, keys, footsteps on
concrete, door latches and metal taps with a phone or recorder; remove handling
noise and record several takes for variation. Use those recordings as source
material for an original score: slow or pitch down a metal scrape, stretch a
fan recording into a drone, filter it, then add a sparse high texture and an
occasional dissonant note. An unresolved minor second or tritone can suggest
instability, but silence and changing context do more work than a permanent
dissonant chord.

Put audible low bass beneath the soundscape to suggest machinery or pressure;
try a restrained 40-80 Hz layer, then test on ordinary speakers and headphones.
Do not claim that sub-20 Hz 'infrasound' automatically causes fear: the
evidence is disputed, many playback systems cannot reproduce it, and excessive
bass can be uncomfortable. Check the mix at low volume, without headphones, and
with a reduced-dynamic-range option. Reference: the controlled [Haunt Project
study](https://www.researchgate.net/publication/51409650_The_Haunt_project_An_attempt_to_build_a_haunted_room_by_manipulating_complex_electromagnetic_fields_and_infrasound)
found no significant increase in unusual sensations from infrasound/EMF in its
test; [Science Wows'
review](https://sciencewows.ie/blog/infrasound-and-the-paranormal/) discusses
the contested ghost-frequency story. These are cautions, not a recipe for a
psychoacoustic effect.

### A practical first mix

| State | Bed | Distinctive cue | Mix change |
| --- | --- | --- | --- |
| Explore | Quiet ventilation and sparse room tone | Directional electrical fault or water drip | Leave room for player steps and key clues. |
| Suspicion | Same bed, subtly narrowed/filtered | One distant metal knock or muffled step from a real location | Do not use a threat cue if no threat can be present. |
| Investigate | Slight drone swell | Monster steps and door contact with audible direction/distance | Lower competing ambience so the player can interpret the cue. |
| Chase | Pulse or faster repeated industrial rhythm, not necessarily a new song | Monster movement and player breathing | Duck the bed under important direction cues; do not mask escape signage feedback. |
| Relief | Drone and pulse recede | Residual room tone | Give the player a real breather before rebuilding tension. |

Crossfade beds instead of hard-switching loops; use non-repeating short details
with safe gaps. Reserve a strong stinger for a rare, earned reveal. Set a
ceiling for stinger loudness; a sudden extreme level is not a substitute for
fear. Compare your mix with monster sounds muted: if the space stops feeling
uneasy, improve the ambient storytelling first.

Make three sound families: (1) navigation/interaction (keys, doors, footsteps),
(2) believable facility mechanisms (ventilation, power fault, pipes), and (3)
pursuer (distinct footfalls, scraping, breathing). Record or synthesize several
variations of each short event to avoid obvious repetition. Keep key-collection
confirmation clear; an ambiguous pickup makes the objective frustrating. A
low-passed sound behind a wall can imply distance, but visual geometry and
audio direction must agree. Do not rely only on stereo panning: provide
readable visual or subtitle cues for critical threat states.

For process examples, see [Loopmasters' drone design
guide](https://www.loopmasters.com/articles/2620-Drone-Sound-Design-8211-How-to-Design-a-Background-Drone-Sound),
[ModeAudio's soundscape
tutorial](https://modeaudio.com/magazine/halloween-tutorial-creating-a-creepy-soundscape),
[Randy Coppinger's dynamic mixing
notes](https://randycoppinger.com/2014/10/22/dynamic-mixing-for-games/), and
[Game Developer on
ducking](https://www.gamedeveloper.com/audio/game-audio-theory-ducking). These
are technique references, not licenses to reuse embedded samples.

## Reusable source candidates

| Source | What to audition | Rights / caveat |
| --- | --- | --- |
| [OpenGameArt: Ambient horror](https://opengameart.org/content/ambient-horror) | Dark ambient bed or structure to study | Item page lists CC0; check the exact file and suitability before import. |
| [OpenGameArt: Horror Sound Effects Library](https://opengameart.org/content/horror-sound-effects-library) | Breathing, creature and impact source sounds | Item page lists CC-BY 3.0; attribution is required. Avoid blindly following third-party credit-domain links. |
| [OpenGameArt: CC0 Dark Music](https://opengameart.org/content/cc0-dark-music) | Compare sparse score textures | Confirm license per item and whether any included loops have separate terms. |
| [Freesound](https://freesound.org/help/faq/) | Search for HVAC, metal creak, door latch and ventilation; filter by license | Each sound has its own license. Prefer CC0; CC BY requires attribution; avoid NC for possible commercial use. Check the individual sound page before download. |

Work with freely licensed *sources* as raw material, not necessarily finished
music. Keep an asset ledger when promoting a sound to `assets/`. Confirm
runtime audio capabilities against this project's Bevy version before choosing
a spatial-audio plugin; the current manifest uses Bevy 0.19.1. Prototype
directional cues in-engine rather than assuming a plugin or stereo effect is
supported.
