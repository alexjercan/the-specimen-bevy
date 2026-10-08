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
