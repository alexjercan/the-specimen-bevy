# Prepare The Specimen itch.io page copy and images

- STATUS: CLOSED
- PRIORITY: 0
- TAGS: backlog

## User facts

- Create `art/content/itch.html` with a game description for the itch.io page and include gameplay images. The user first requested the task folder, then moved all deliverables to `art/content/`.
- Provide a color palette and page settings for BG, BG 2, Text, Link, Headers, Buttons, BG2 Alpha, font Lato, size Large, layout screenshots Auto, image banner/background, and Embed BG.
- Image `src` values must start with `https://img.itch.zone/`; the user will upload images and replace placeholders. Use the captures in content-machine `media/the-specimen-bevy-gallery/recordings/image/*.png` and `media/the-specimen-bevy-gallery/recordings/video/*.gif`.
- Copy screenshots, GIFs, and the proof-of-concept trailer into `art/content/`; include GIFs in the description. Keep this task OPEN until the user finishes the itch.io page.

## Agent findings

- content-machine has a 25-second trailer at `media/the-specimen-bevy-poc/final.mp4` and a current gallery of 1920x1080 PNGs: `assembly-stalker.png`, `fuse-table.png`, `hide-prompt.png`, `exit-locked.png`, and `lab-blackout.png`. The gallery also has `locker-watch.gif`, `boiler-restore.gif`, and `fuse-install.gif`. The user will upload selected media to itch.io.
- Older `art/visuals/screenshots/` files include prototype material. The manual deploy-itch workflow now reuses the tagged release's HTML5, Windows, and Linux archives.

## Decisions

- Automode initially selected repository-hosted images. The user superseded that decision: use `https://img.itch.zone/` placeholders for manual upload. The user requested copies of screenshots, GIFs, and the trailer in `art/content/`, together with `itch.html`; keep this task file here. Do not publish the itch.io page or upload anything.

## Delivery

- Add `art/content/itch.html` with accurate gameplay copy and `https://img.itch.zone/` placeholders for `assembly-stalker.png`, `fuse-table.png`, `hide-prompt.png`, `exit-locked.png`, `locker-watch.gif`, `boiler-restore.gif`, and `fuse-install.gif`. Copy five screenshots to `art/content/screenshots/`, three GIFs to `art/content/gifs/`, and the trailer to `art/content/trailer.mp4`. Preserve user authorship and avoid promises about unverified browser behavior.
- Keep the upload checklist, image-placeholder mapping, palette, and presentation settings in `art/content/README.md` for use during manual page setup. `art/content/banner.png` is a 960x240 itch.io header and `art/content/cover.png` is a 630x500 itch.io cover, both composed from a v0.1.1 pinned game capture.

## Verification

- Gallery captures and GIFs were inspected; selected placeholders correspond to copied media. All seven HTML image `src` values begin with `https://img.itch.zone/` and need real URLs after manual upload. Suggested palette foreground/background contrast ratios exceed 9:1. `art/content/README.md` records the actual colors and media mapping.
- Placeholder URLs are intentionally not valid hosted images. itch.io HTML editor rendering is not verified; use source/HTML mode, replace placeholders, and preview before publishing.
- The supplied stalker URL ending `/250x600/%2F%2F2xU3.png` returns a 250x140 PNG. Changing that URL to `/original/`, `/1280x720/`, or other guessed size paths returned HTTP 404. Keep the stalker placeholder until a verified full-resolution itch.io image URL is available; do not upscale the thumbnail.

## Done when

- Keep this task OPEN until the user has uploaded media, replaced the image/GIF placeholders with verified full-size itch.io URLs, previewed the description, and approved closure. Do not change the itch.io page without explicit user action.
