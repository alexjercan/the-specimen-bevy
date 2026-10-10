# The Specimen Steam page draft and capsules

- STATUS: CLOSED
- PRIORITY: 0
- TAGS: backlog

## User facts

- Inspect Nova `art/steam/README.md` and produce The Specimen equivalents: `STEAM.md` upload sheet, a full standalone `steam.html` visual draft, and the Steam capsules generated in content-machine in the itch banner/cover style.
- The HTML is a local replica, not an upload. The Steam description field uses BBCode and `{STEAM_APP_IMAGE}` tokens; mirror that distinction.
- Validate nine slots: header 920x430, small 462x174, main 1232x706, vertical 748x896, page background 1438x810, library capsule 600x900, library header 920x430, library hero 3840x1240 with no text, library logo transparent.
- Keep v0.1.1 pinned source and provenance. Do not invent system requirements or confirmed Steamworks account fields. Do not claim unsupported features or Steam brand endorsement. Do not publish or upload.

## Decisions

- Owner (automode, reversible): build all capsules from the existing pinned v0.1.1 gallery stills with the itch crop and grade. A HUD-free 4K recapture for the hero and page background is a recorded pre-upload quality gap.

## Agent findings

- `art/content/screenshots/` (4 of 5) and all of `art/content/gifs/` are from the 17:47 capture of the unpinned `master` capsule. The v0.1.1 recapture at 19:31 replaced the stills and WebMs in content-machine media but not the GIFs. Only `fuse-table.png` matches byte for byte. The itch page files are unchanged.
- `art/content/trailer.mp4` is content-machine `media/the-specimen-bevy-poc/final.mp4` (17:35), from before the v0.1.1 pin.
- Steamworks docs (fetched 2026-10-10): capsule text is the title only; small capsule logo should nearly fill it; hero has no text and a centered 860x380 safe area; logo is transparent, 1280 wide and/or 720 tall; at least 5 screenshots, 1920x1080 minimum.
- v0.1.1 builds Windows and Linux x86_64 plus HTML5. No gamepad bindings. English only. MIT.

## Delivery

- content-machine `projects/the-specimen-bevy-gallery/steam.py` (new): nine slots from `lab-blackout.png` above the HUD (y < 885), itch banner/cover grade and title; re-encodes the three v0.1.1 WebMs to GIF; fails on a wrong size or a non-transparent logo edge. Output in untracked `media/the-specimen-bevy-gallery/steam/`.
- `art/steam/STEAM.md`: upload sheet, BBCode description, provenance, reproduction, open decisions.
- `art/steam/steam.html`: standalone local replica plus a slot appendix. No network dependency.
- `art/steam/capsules/` (9), `art/steam/screenshots/` (5 v0.1.1 stills, numbered), `art/steam/extras/` (3 v0.1.1 GIFs).

## Verification

- All nine sizes match; logo RGBA 1280x455; every edge row and column has zero alpha (checked by `steam.py`). Each capsule inspected at full size; small capsule inspected at 184x69 and 120x45.
- Rerunning `steam.py` gives byte-identical capsules and GIFs.
- Every `src`/`href`/`poster`/`url()` in `steam.html` resolves locally except the two GitHub links. Rendered with headless Chromium at 1280 and 600 px widths and inspected.
- `git diff --no-index --check` clean on the new text files; all ASCII.
- Independent review (review-code). Fixed: "SteamOS" platform claim (now Linux only, Deck untested); logo glow wording; no-JS trailer thumbnail wording; logo edge check now covers all edges, which found faint glow on the edge, so the trim now uses all nonzero alpha. Rejected with evidence: "HUD from y 876". In `lab-blackout.png`, the first HUD row is y 894 (detector rim at x 848); rows 860-892 have no panel edge, so `HUD_TOP = 885` keeps a 9-row margin.
- Second review report (arrived late). Fixed: missing CC-BY-4.0 credit for "The Human Deer" (`credits/CREDITS.md` at `fc51cf5`); the required credit line is now in the BBCode and the replica. Hero crop moved up 50 source rows so skull, mouth and upper chest sit in the 860x380 safe area; checked with an overlay. BBCode and HTML About text checked identical by script. Rerender is byte-identical.
- `art/content/itch.html` also lacked the creature credit. The exact required credit line from `credits/CREDITS.md` was added there after the Steam draft review; this is a local edit, not an itch.io page update.

## Gaps

- Hero and page background are upscaled drafts; replace with a HUD-free 3840x2160 capture before upload.
- Trailer predates the pin; audio provenance unchecked.
- Only three all-ages screenshot candidates (01, 04, 05); Steam asks for four.
- Tags are a proposal, not harvested from anchors. Price, date, genre, developer/publisher, mature content answers, system requirements, itch.io URL: owner decisions.

## Done when

- `art/steam/` holds `STEAM.md`, `steam.html`, nine capsules and the logo master at the exact sizes, and the HTML references resolve.
- content-machine has a rerunnable generator for the capsules.
- Gaps are recorded. Do not mark the Steam page as published.
