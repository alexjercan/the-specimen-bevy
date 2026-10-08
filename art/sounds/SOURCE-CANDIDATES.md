# Sound source candidates

Research-only candidates for `tasks/20261007-172901/TASK.md`, checked on
2026-10-08. These leads were initially research-only. The `_stubb` locked-door preview
was downloaded, edited, then rejected by listening and removed from the audio
tree. DrFahrts's recorded doorknob preview edit was approved and promoted for
locked-door attempts; the shelbyshark alternative remains review-only. The other listed sounds were not
downloaded, auditioned, or edited.

## Search and rights checks

Freesound search and item pages were accessible with `curl`. Searches were
filtered to pages displaying `Creative Commons 0`; each item page was then
opened. For every candidate below, I checked its displayed title, creator,
description, exact original download filename shown by the page, OGG preview
URL, and license link. A `curl -I` request to each listed preview returned
HTTP 200. The `_stubb` preview was fetched and rejected. The shelbyshark and DrFahrts
previews were then fetched for review; the remaining preview URLs were checked
with HEAD only. Freesound's download links required
login, so full-resolution originals were not checked or auditioned.

Each item page displays `Creative Commons 0` and links to the CC0 1.0 deed at
<http://creativecommons.org/publicdomain/zero/1.0/>. I checked that deed and
its linked legal code. This records what the uploader's page says; it does not
prove the uploader owns the recording or has cleared every right. CC0's own
terms also give no warranty. Treat each source as unverified until its
provenance and full file are reviewed. The sound descriptions below are
uploader-provided, not independent findings.

The rejected `_stubb` take is no longer catalogued. The DrFahrts edit is
approved for runtime; the shelbyshark edit remains a catalog-only candidate. The approved UI
fuse slot and completion sounds replace the former pickup/panel sounds; do not
use other leads here to replace approved steps, doors, UI, hiding or ambience
without a separate listening decision.

## Candidates

| Proposed cue | Creator and item page | Exact file / preview checked | Displayed license | Proposed use and limits |
| --- | --- | --- | --- | --- |
| `door.locked.rattle` | `_stubb`, [Door Handle Rattle_Quiet_Near_Mono.wav](https://freesound.org/people/_stubb/sounds/406229/) | [Preview](https://cdn.freesound.org/previews/406/406229_6068748-lq.ogg) | `Creative Commons 0`; CC0 1.0 linked from item page | Rejected after audition. Preview and edit removed; do not propose this take again. |
| `door.locked.rattle` | `shelbyshark`, [Rattling Locked Door.wav](https://freesound.org/people/shelbyshark/sounds/513392/) | [Preview](https://cdn.freesound.org/previews/513/513392_8644110-lq.ogg) | `Creative Commons 0`; CC0 1.0 linked from item page | Uploader describes a panicked attempt to open a locked door. 17.95 s preview; 2 s review crop in the catalog, not auditioned. |
| `door.locked.rattle` | `DrFahrts`, [doorknob rattle](https://freesound.org/people/DrFahrts/sounds/727791/) | [Preview](https://cdn.freesound.org/previews/727/727791_10249017-lq.ogg) | `Creative Commons 0`; CC0 1.0 linked from item page | Uploader describes trying a locked doorknob repeatedly. 29.90 s preview; a 2 s crop was approved by listening and promoted to runtime. In-game mix not reviewed. |
| `door.unlock.mag` | `Brickhario`, [door_unlock_brickhario.wav](https://freesound.org/people/Brickhario/sounds/491111/) | Page's download filename: `491111__brickhario__door_unlock_brickhario.wav`; [preview](https://cdn.freesound.org/previews/491/491111_5872811-lq.ogg) | `Creative Commons 0`; CC0 1.0 deed/legal code linked from item page | Possible short latch-release source after panel installation. The uploader describes unlocking a door with a key. Reject it if the sound implies a key action or clashes with the exit's electronic lock; this is not a confirmed magnetic-lock sound. |
| `door.unlock.mag` | `TampaJoey`, [MetalGateUnlock.wav](https://freesound.org/people/TampaJoey/sounds/588504/) | Page's download filename: `588504__tampajoey__metalgateunlock.wav`; [preview](https://cdn.freesound.org/previews/588/588504_10507437-lq.ogg) | `Creative Commons 0`; CC0 1.0 deed/legal code linked from item page | Alternative compact mechanical clunk for the exit unlock. The uploader describes a metal-gate thud made for a game; the page's title says “Unlock.” Audition before use to confirm it reads as release, not a door slam. |
| `door.stop.open` | `Yoyodaman234`, [Metal Shed Door Open 1](https://freesound.org/people/Yoyodaman234/sounds/334681/) | Page's download filename: `334681__yoyodaman234__metal-shed-door-open-1.wav`; [preview](https://cdn.freesound.org/previews/334/334681_2792951-lq.ogg) | `Creative Commons 0`; CC0 1.0 deed/legal code linked from item page | Weak lead for a full-open end-stop: the uploader describes a small metal shed door opening, but does not say the take contains a distinct stop impact. Only consider a cropped endpoint if audition confirms one; do not reuse the whole opening over the approved door swing. |
| `self.cloth` | `xkeril`, [Clothes movements (walking foley)](https://freesound.org/people/xkeril/sounds/788342/) | Page's download filename: `788342__xkeril__clothes-movements-walking-foley.wav`; [preview](https://cdn.freesound.org/previews/788/788342_13504080-lq.ogg) | `Creative Commons 0`; CC0 1.0 deed/legal code linked from item page | Candidate low-level player clothing rustle for physical movement. Uploader says it has no footstep noise and describes sweater, denim jacket, and pants; the 49.29 s recording includes zipper sounds. Segment and audition carefully so it does not mask the approved footsteps or locked-door cue. |
| `ui.hint.appear` | `Jummit`, [Soft UI Button Click](https://freesound.org/people/Jummit/sounds/528561/) | Page's download filename: `528561__jummit__soft-ui-button-click.ogg`; [preview](https://cdn.freesound.org/previews/528/528561_10360410-lq.ogg) | `Creative Commons 0`; CC0 1.0 deed/legal code linked from item page | Candidate faint non-spatial tick when the gameplay `OPEN` / `CLOSE` / `PICK UP FUSE` hint first appears or changes target. Page calls it a soft sci-fi button/hover sound; do not add it to menu hover/focus, which already has approved cues. |
| `ui.fuse.complete` | `CogFireStudios`, [Positive Blip Effect](https://freesound.org/people/CogFireStudios/sounds/531512/) | Page's download filename: `531512__cogfirestudios__positive-blip-effect.wav`; [preview](https://cdn.freesound.org/previews/531/531512_7614679-lq.ogg) | `Creative Commons 0`; CC0 1.0 deed/legal code linked from item page | Optional candidate for a restrained all-fuses-ready confirmation, only if the event does not already use an approved UI cue. Page title is the only sound-content description; duration is 3.00 s. Do not assume it is a short blip or use it as a reward fanfare without audition. |

## Boiler breaker and reset leads (task #181)

OpenGameArt pages inspected on 2026-10-08. These are provisional mechanical-switch
sources, not confirmed field recordings or approved replacements. Neither is in
the sound catalog or runtime assets. Temporary files were inspected outside the
repository; no source audio was added here.

| Cue to review | Item and creator | Inspected contents | Displayed rights and limits |
| --- | --- | --- | --- |
| Breaker trip | [SFX - Circuit breaker](https://opengameart.org/content/sfx-circuit-breaker), CleytonKauffman | The linked [Circuit Breaker.zip](https://opengameart.org/sites/default/files/Circuit%20Breaker.zip) contains `switch off.wav` (44.1 kHz stereo PCM, 0.92 s) and `switch on.wav`. Page describes breaker ON/OFF, but does not state recording method. The page requests `SFX by Cleyton Kauffman - https://soundcloud.com/cleytonkauffman`. | Page displays CC0. Archive has no bundled license or source provenance; author ownership and whether this is a recording remain unverified. Audition `switch off.wav` for a weighty breaker trip. |
| Boiler reset control | [Stove switch](https://opengameart.org/content/stove-switch), TinyWorlds | Page describes a stove switch and links `switch_strength_stove_01.ogg` (Vorbis, 44.1 kHz stereo, 0.51 s) and `turn_stove_on_01_0.ogg`. Only the first linked file was inspected technically. | Page displays CC0; no recording-method or upstream provenance statement was found. Audition as a possible physical control click, not a confirmed boiler reset. |

Neither page's displayed license label establishes source rights. Inspect the
remaining original files, confirm provenance, and get listening approval before
adding any edit to the review catalog or runtime set. Existing approved
`power-down` and `restart` cues remain unchanged.

## Next review

Audition only candidates that still fill a cue gap. Confirm the full file,
creator's rights, exact license/version, edit boundaries, mix, and attribution
needs before approval or use. Keep provenance for any selected source in the
project's sound-source notes and credits. If no recording fits the locked-door
or door-stop cue, use the task's original-synthesis option rather than forcing
a poor match.
