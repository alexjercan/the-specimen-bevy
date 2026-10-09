# Credits

When adding third-party art, audio, fonts, or other assets, record the creator,
source URL, license, changes made, and any required attribution here. Copy
license text into `credits/licenses/` when required for distribution. Use
`cargo-about` for dependency licenses. The audio credits below are voluntary
source credits requested by the project owner; CC0 does not require attribution.

## Iosevka Term

`assets/ui/fonts/SGr-IosevkaTerm-Medium.ttf` is Iosevka Term by Renzhi Li
(Belleve Invis), from https://typeof.net/Iosevka/ . Copied unchanged from the
Nova Protocol asset bundle. Licensed under SIL Open Font License 1.1. The
required copyright notice and license text are in
`credits/licenses/Iosevka_OFL-1.1.md`. Distribute that file with the font.

## FREE Input Prompts

The keyboard and mouse key glyphs in `assets/ui/input-prompts/` are from the
FREE Input Prompts Pack v1.4 by JulioCacko,
https://juliocacko.itch.io/free-input-prompts , copied unchanged from the Nova
Protocol asset bundle. Nova Protocol records the pack as CC0 1.0 Universal;
this project has not independently verified that against the upstream page.
CC0 does not require attribution; this is a courtesy credit. License text:
`credits/licenses/FREE-Input-Prompts_CC0-1.0.md`. Copied files and source
commits are in `credits/ASSET-SOURCES.md`.

## The Human Deer

"The Human Deer" by ceeleste, https://sketchfab.com/3d-models/the-human-deer-7644694337404bb18eb68e6b637740a1 , licensed under CC-BY-4.0 according to its bundled license. Required credit: This work is based on "The Human Deer" (https://sketchfab.com/3d-models/the-human-deer-7644694337404bb18eb68e6b637740a1) by ceeleste (https://sketchfab.com/ceeleste) licensed under CC-BY-4.0 (http://creativecommons.org/licenses/by/4.0/). The runtime `assets/monster/human_deer_animated.glb` is copied from `art/visuals/generated/monster/`. WALK, CHASE, and ATTACK animations were added and IDLE was resampled; see its README and the bundled source notice copied to `credits/licenses/The-Human-Deer-source-license.txt`. License and provenance are source-provided claims, not independently verified.

## Monster patrol recordings

Gristi, [snd_footsteps_metal_floor_inside.wav](https://freesound.org/people/gristi/sounds/562195/): the low-quality preview at `assets/sounds/monster/amb/metal-footsteps-gristi.ogg` is used for occasional monster presence. Ultra-Edward, [Crawling Through a Vent](https://freesound.org/people/Ultra-Edward/sounds/795872/): six edited step clips at `assets/sounds/monster/step/` derive from its preview. The pages display CC0; preview terms and uploader rights have not been independently verified. Jofae, [Growl and Roar](https://freesound.org/people/Jofae/sounds/366837/): detection and attack edits are retained at `assets/sounds/monster/detected/` and `assets/sounds/monster/attack/`. under_the_hood, [Real heartbeat sound fastest](https://freesound.org/people/under_the_hood/sounds/455440/): an edited loop is at `assets/sounds/monster/chase/`. These three cues are loaded but not played by patrol. All four source pages display CC0; preview terms and uploader rights have not been independently verified. Source hashes and edit details are in `art/sounds/sources/freesound/monster/README.md`.

## Recorded footsteps

GboxMikeFozzy, [Footsteps](https://opengameart.org/content/footsteps-0),
OpenGameArt item labeled [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
The author's subway-walking recordings `01`, `02`, and `04` are shipped unchanged
at `assets/sounds/step/subway/` and used as footstep variations. Source-page
licensing and authorship have not been independently verified. Original file
hashes and filenames are in `art/sounds/README.md`.

## Recorded door and hiding sounds

rubberduck, [100 CC0 metal and wood SFX](https://opengameart.org/content/100-cc0-metal-and-wood-sfx),
OpenGameArt item labeled [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
The project's edited unlatch, swing-open and shut cues at
`assets/sounds/door/` use recordings from this pack. The four approved hiding cues at `art/sounds/generated/hiding/` also use
its metal and wood recordings, filtered and mixed by
`scripts/render_hiding_sounds.py`. Runtime copies are in `assets/sounds/hiding/`.
Source-page licensing and authorship have not been independently verified.
Source archive and edit details are in `art/sounds/README.md`.

## Recorded locked-door rattle

DrFahrts, [doorknob rattle](https://freesound.org/people/DrFahrts/sounds/727791/),
Freesound item labeled [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
The two-second edit at `assets/sounds/door/locked-rattle.wav` was cut from
its low-quality preview. The item page describes trying a locked doorknob
repeatedly. Only the preview was obtained; the page label does not
independently verify uploader ownership or original-file terms. Source and
edit hashes are recorded in `art/sounds/README.md`.

## Boiler breaker switch-off

CleytonKauffman, [SFX - Circuit breaker](https://opengameart.org/content/sfx-circuit-breaker),
OpenGameArt item labeled [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
The `switch off.wav` from the linked archive is retained at
`art/sounds/sources/opengameart/amb/boiler/switch-off-cleytonkauffman.wav` and
copied unchanged to `assets/sounds/amb/boiler/switch-off-cleytonkauffman.wav`.
The item page asks for `SFX by Cleyton Kauffman -
https://soundcloud.com/cleytonkauffman`. The archive includes no bundled
license or provenance; source-page licensing and authorship have not been
independently verified. The file hash is in `art/sounds/README.md`.

## Recorded sprint-exhaustion breathing

mikeask, [Breathing Tired](https://opengameart.org/content/breathing-tired),
OpenGameArt item labeled [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
The recording at `assets/sounds/self/breathing-tired-mikeask.wav` is used as a
one-shot when sprint stamina runs out. Its retained original and hash are in
`art/sounds/README.md`. The source-page license and uploader ownership have
not been independently verified.

## Recorded flashlight switch

Ralph0o7, [Flashlight switch](https://freesound.org/people/Ralph0o7/sounds/690300/),
Freesound item page labeled [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
The low-quality preview is used unchanged for manual flashlight on/off clicks
at `assets/sounds/flashlight/click-ralph0o7.ogg`. The item page's license
label does not independently verify uploader ownership or original-file terms.
The retained source preview and hash are recorded in `art/sounds/README.md`.

## Edited Freesound ambience previews

The following Freesound pages display [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
Only the low-quality previews were obtained; page labels do not independently
verify authorship, recording rights, or original-file terms. Keep this provenance
with any game distribution that includes the edits.

- iankath, [Furnace.WAV](https://freesound.org/people/iankath/sounds/173991/):
  the 24-second filtered, crossfaded boiler loop at
  `assets/sounds/amb/furnace/burning.wav`.
- willstepp, [Water Drop](https://freesound.org/people/willstepp/sounds/188293/):
  the filtered 5-second water burst at `assets/sounds/amb/water/faucet.wav`.
- DBlover, [Howling Wind Ambience](https://freesound.org/people/DBlover/sounds/405601/):
  the 18-second filtered, crossfaded vent wind at
  `assets/sounds/amb/vent/wind.wav`.

Preview sources, hashes, and edit parameters are in
`art/sounds/sources/freesound/README.md` and
`scripts/render_selected_ambience.py`.
