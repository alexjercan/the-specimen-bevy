# Changelog

## [Unreleased]

### Achievements

- Track seven achievements and save them on the device (native file or browser storage).
- Show all achievements with their status in a new Achievements screen below Credits, and show a toast when one unlocks.
- Optional `steam` build feature syncs achievements with Steam when Steam is available. See `docs/achievements.md`.

## [0.1.1] - 2026-10-09

### Distribution

- Publish Linux, Windows, and HTML5 archives together as a GitHub Release after all tagged builds pass.
- Deploy itch.io channels from published GitHub Release assets instead of temporary Actions artifacts.

## [0.1.0] - 2026-10-09

First playable release of The Specimen.

### Gameplay

- Explore a facility while a monster pursues you. Open doors and use lockers and other hiding places to evade it.
- Find three fuses, install them at the exit panel, and escape. Use a flashlight, flashbangs, and a detector to survive.
- Experience power outages, facility lighting, spatial audio, and cinematic game screens.

### Distribution and tools

- Tag-triggered release workflow packages Linux, Windows, and browser (HTML5) builds as GitHub Actions artifacts.
- Manual workflows deploy the browser game to GitHub Pages and reuse tagged release artifacts for the itch.io HTML5, Windows, and Linux channels.
- Debug inspector shows grouped entities and lets testers edit the player's fuse inventory.

[Unreleased]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/alexjercan/the-specimen-bevy/tree/v0.1.0
