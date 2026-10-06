# Sound direction: an inhabited empty building

Research and production sketches, not approved audio assets. Nothing linked
here has been downloaded into the game. Keep the original sound's page, author,
license, license version, and required attribution when selecting a file; a
site's general policy does not replace the license on a particular upload.

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
