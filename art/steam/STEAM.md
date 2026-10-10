# Steam store page: the upload sheet

This sheet lists every field and file for a future Steam page, in upload order,
with copy that you can paste. The files beside this document are the files
that upload. `steam.html` does not upload.

The Steam page does not exist. There is no app ID, no Steamworks account
name, no price and no release date. Nothing here is uploaded or published.

## The two forms of the page

| File | What it is | What it is not |
| --- | --- | --- |
| `steam.html` | A local replica of the store layout, for review with people who do not know the game. It uses only the files in this folder and `../content/trailer.mp4`. It needs no network. | An upload. Steam does not accept HTML in any store field. It is not a Valve page, and it shows no Valve logo. |
| This file | The text for the Steamworks admin fields. The description is BBCode. | A preview. Steam renders the BBCode, not the replica. |

The replica renders the About section from the BBCode below by hand. If you
change the copy, change both. Its screenshot strip uses JavaScript; without
JavaScript, the first thumbnail opens the trailer and the others open the full image. The store links and the
wishlist button are inert. The separate [achievement preview](achievements.html)
shows planned icons and rules. The game tracks and saves these achievements
locally; Steam sync is an optional build feature with no App ID yet. See
`docs/achievements.md`.

## Source and provenance

All images and loops come from the v0.1.1 release tag
(`fc51cf5f4b10585e5f183662e112a7c0741d495e`), captured by content-machine with
the `the-specimen-bevy/v0.1.1` capsule and the `gallery` scene (seed 7, monster
AI off, puppet monster, real HUD, `debug` feature off).

| Files here | Source in content-machine |
| --- | --- |
| `screenshots/*.png` | `media/the-specimen-bevy-gallery/recordings/image/*.png`, copied without change and renamed with the upload order. |
| `extras/*.gif` | `recordings/video/*.webm`, encoded to GIF by `projects/the-specimen-bevy-gallery/steam.py` with the gallery GIF command. |
| `capsules/*.png` | `projects/the-specimen-bevy-gallery/steam.py`, from `recordings/image/lab-blackout.png`, with the itch.io banner and cover crop, grade and title. |

Reproduce:

```sh
cd ../content-machine
scripts/capture.sh the-specimen-bevy-gallery
nix develop -c python projects/the-specimen-bevy-gallery/steam.py
cp media/the-specimen-bevy-gallery/steam/capsules/*.png ../the-specimen-bevy/art/steam/capsules/
cp media/the-specimen-bevy-gallery/steam/extras/*.gif ../the-specimen-bevy/art/steam/extras/
```

Then copy the five stills into `screenshots/` with the numbered names below.
A full rerun of the capture gives visually identical frames, not identical
bytes. A subset run (`SPECIMEN_SHOTS=...`) changes lamp pulse and idle phase.

`steam.py` stops if a capsule has the wrong size, or if the logo is not 1280
wide with a transparent edge.

Do not mix in `../content/screenshots/` or `../content/gifs/`. Four of those
five stills and all three GIFs come from an earlier capture of the unpinned
local checkout. `../content/screenshots/fuse-table.png` is the only byte match.

## Upload order

1. Trailer: `../content/trailer.mp4`, 25.2 s, 1920x1080 at 60 fps, H.264 and
   AAC. This is the proof-of-concept cut, rendered before the v0.1.1 pin. See
   "Not settled".
2. `screenshots/`, all five, in numbered order. Steam requires at least five.
   The first four show in the hover tooltip: the creature in a lit room, a
   hiding spot, the creature in the flashlight, a fuse.
3. `extras/`, three GIFs, into the app's Extras image area, under these exact
   file names. The description references them by token.
4. `capsules/`, one file per slot. See the table.
5. The copy fields, below.

| File | Content |
| --- | --- |
| `01-assembly-stalker.png` | The creature among assembly benches, power on. |
| `02-hide-prompt.png` | A locker under a red lamp with the real `HIDE` hint; the creature in the corner. |
| `03-lab-blackout.png` | Blackout; the flashlight on the creature beside the lab tank. |
| `04-fuse-table.png` | A fuse on a table with the real `PICK UP FUSE` hint. |
| `05-exit-locked.png` | The exit door under the exit sign with the real `LOCKED` hint. |

Every still shows the real game HUD and no debug overlay. Steam allows HUD in
screenshots; it does not allow marketing text, so none is added.

Candidates for "suitable for all ages": 01, 04, 05. 02 and 03 show the
creature's bloody mouth at close range. Steam asks for at least four all-ages
screenshots. That needs one more still or an owner decision on 02.

## Short description

    Find three fuses, open the exit, and stay out of its sight. The Specimen is a first-person horror escape through a dark facility while a creature hunts the halls.

162 characters. The limit is 300. Confirm the live counter before saving.

## Description

The Steam description field is BBCode. Each `{STEAM_APP_IMAGE}/extras/...`
token resolves after you upload the GIF of that name to the Extras area.

    Find three fuses. Open the exit. Stay out of its sight.

    The Specimen is a first-person horror game set inside a dark facility. Search its rooms for three fuses, install them at the exit panel, and escape while a creature hunts through the halls.

    [h2]Search the facility[/h2]
    Follow the signs, explore the rooms, and gather the three fuses needed to open the exit. A flashlight helps you find your way when the lights go out.

    [h2]Hide or buy time[/h2]
    The creature patrols the halls. Hide in lockers or under tables, use the detector to track it, and throw a flashbang when you need an opening.
    [img]{STEAM_APP_IMAGE}/extras/locker-watch.gif[/img]

    [h2]Restore the power[/h2]
    When the lights fail, find the boiler to bring the facility back online.
    [img]{STEAM_APP_IMAGE}/extras/boiler-restore.gif[/img]

    [h2]Reach the exit[/h2]
    Install the fuses at the panel, open the door, and make it outside.
    [img]{STEAM_APP_IMAGE}/extras/fuse-install.gif[/img]

    [h2]Features[/h2]
    [list]
    [*]One facility to escape: find three fuses and install them at the exit panel.
    [*]A creature that patrols the halls and pursues you. If it catches you, the run ends.
    [*]Lockers and tables to hide in, a detector that tracks the creature, and flashbangs that buy you time.
    [*]A flashlight, power outages, and a boiler that brings the lights back.
    [*]Spatial audio. Play with headphones.
    [*]Single player, keyboard and mouse.
    [*]The source code is public on GitHub under the MIT license.
    [/list]

    [h2]Early release[/h2]
    This is an early playable release. This page is for a future Steam release; no date and no price are announced.

    [h2]Credits[/h2]
    The creature model: This work is based on "The Human Deer" (https://sketchfab.com/3d-models/the-human-deer-7644694337404bb18eb68e6b637740a1) by ceeleste (https://sketchfab.com/ceeleste) licensed under CC-BY-4.0 (http://creativecommons.org/licenses/by/4.0/). Full credits ship with the game in credits/CREDITS.md.

The copy follows `../content/itch.html`, which already went through review.
Each feature line is true at v0.1.1:

| Claim | Source at `fc51cf5` |
| --- | --- |
| Three fuses, exit panel | `crates/gameplay/src/levels/` (`FusePanel`, `PickupKind::Fuse`), `CHANGELOG.md` 0.1.0 |
| Caught ends the run | `crates/core/src/menu/game_over.rs` (`Caught`, `GameOverScreen`) |
| Lockers and tables | `crates/gameplay/src/levels/hiding.rs` (`HidingSpot::Locker`, `HidingSpot::Table`) |
| Detector, flashbangs | `crates/gameplay/src/levels/devices.rs`, `crates/core/src/glue/device_hud.rs` |
| Outages, boiler | `crates/gameplay/src/levels/power.rs`, the `boiler-restore` loop |
| Spatial audio | `CHANGELOG.md` 0.1.0 |
| Keyboard and mouse only | No gamepad bindings in `crates/` |
| MIT license | `LICENSE` |
| Creature model credit | `credits/CREDITS.md`, "The Human Deer": CC-BY-4.0, required credit line copied word for word |

Do not add claims for gamepad, Steam Deck, macOS, achievements, cloud saves or
other languages. v0.1.1 has none of them, or none was tested.

## Tags, proposed order

    Horror, Survival Horror, First-Person, Stealth, Atmospheric, Exploration,
    Dark, Singleplayer, 3D, Indie

This list is a proposal. It is not harvested from anchor games. Before you
save, pick two or three anchor games, read their top tags, and reorder. The
first five tags show in the hover tooltip.

## The other fields

| Field | Value |
| --- | --- |
| Release date | "Coming soon" wording. No date. |
| Price | Not set. Owner decision. |
| Genres | Owner decision. Adventure is the closest Steam genre. |
| Supported languages | English: interface. No full audio; no other language. |
| Platforms | Windows and Linux. v0.1.1 builds x86_64 archives for both. Steamworks names the Linux checkbox "SteamOS + Linux"; tick it only for the Linux build. Steam Deck is not tested, and the game has no controller support. The HTML5 build is not a Steam platform. |
| Controller support | None. Keyboard and mouse. |
| Mature content survey | Owner decision. Proposed free text: "A creature with blood on its body pursues the player through a facility. If it catches the player, a game-over screen shows it. No human violence or gore is shown." |
| Website link | GitHub: `https://github.com/alexjercan/the-specimen-bevy`. The itch.io URL is not recorded in this repository; add it when known. |
| Developer / publisher | Not set. Must match the Steamworks account name exactly. |

### System requirements

Not measured. Leave every field empty in this sheet. Take the numbers from
the v0.1.1 or later release builds on real hardware. Do not estimate them.

## Capsules

| File | Size | Slot | Content |
| --- | --- | --- | --- |
| `capsules/header-capsule.png` | 920x430 | Store header capsule. | Banner layout: title left, creature right. |
| `capsules/small-capsule.png` | 462x174 | Small capsule. Steam makes 184x69 and 120x45 from it. | The title nearly fills it; the scene is dimmed. Checked legible at 184x69 and 120x45. |
| `capsules/main-capsule.png` | 1232x706 | Main capsule: front page and sale rows. | Banner layout. |
| `capsules/vertical-capsule.png` | 748x896 | Vertical capsule: seasonal sale pages. | Cover layout: creature above, title below. |
| `capsules/page-background.png` | 1438x810 | Store page background, optional. | No title. Dim, edges darkened. |
| `capsules/library-capsule.png` | 600x900 | Library grid tile. | Cover layout. |
| `capsules/library-header.png` | 920x430 | Library header. | Cover layout, title centered. |
| `capsules/library-hero.png` | 3840x1240 | Library hero. | No text. The creature's skull, mouth and upper chest are in the centered 860x380 safe area; the antlers and legs extend outside it. |
| `capsules/library-logo.png` | 1280x455 | Library logo, laid over the hero. | Title only, RGBA, transparent background, with the same soft glow as the capsule titles. |

Valve's rules for every capsule: game art and the game title only. No quotes,
review scores, "coming soon" banners or other text. All nine files keep to
this. The title is the only text, and the hero and background have none.

Valve's asset page lists JPG for several slots. These masters are PNG. If the
upload form refuses PNG for a slot, export JPG at high quality from the same
file. Largest file: `library-hero.png`, 1.22 MB.

The Nova sheet also asks for a separate logo master with margins. Here
`library-logo.png` is trimmed to its ink and glow and is the only logo file.

## What gets a page rejected

- Screenshots must be gameplay. No marketing text, concept art or logos in
  the carousel. The capsules do that job.
- No debug overlay. The capture has the `debug` feature off; no still has the
  FPS counter.
- Capsule text other than the title.
- A description with no feature list.

## Not settled

- Steamworks onboarding, the Steam Direct fee, the app ID, and the developer
  and publisher name. Every upload waits on the app ID.
- Price, release date, genres, the final tag order, and the mature content
  answers.
- System requirements.
- Hero and page background quality. Both scale a 1920x620 or smaller crop of a
  1920x1080 still, so the 3840x1240 hero is soft at full size. Before upload,
  replace them from a HUD-free 3840x2160 capture of the same pose.
- The trailer. It predates the v0.1.1 pin, and its audio sample provenance is
  not checked. Recut it from the v0.1.1 capsule, or check it, before upload.
- One more all-ages screenshot.
- The itch.io URL for the website and the description.
- Every capsule shows the CC-BY-4.0 creature model. The credit is in the description; keep it there.
- All capsules use one still, `lab-blackout`. A second key-art pose would make
  the store and library sets less repetitive.
- Show `steam.html` to people who do not know the game. Ask what game it
  reminds them of and what they think you do in it.
