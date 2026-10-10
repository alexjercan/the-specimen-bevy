# Changelog

## [Unreleased]

## [0.2.0] - 2026-10-10

### Gameplay and achievements

- Track seven achievements with local native or browser persistence, an in-game viewer, icon and sound notifications, and an optional Steam sync feature. The release builds do not enable Steam sync.
- Award "In the dark" only when escaping during a power outage without restoring the boiler.
- Confirm flashbang hits within four meters and clear line of sight before granting five seconds of protection from the affected monster. Flashbangs do not stun monsters.

### Menus and presentation

- Add credits to the main menu and post-escape sequence, over the red-lit closed exit door.
- Show Settings and Achievements over their menu scenes; add both views to the pause menu without resuming play.
- Add a bounded, scrollable Settings panel, mouse-button control glyphs, slider and button sounds, and a 0-4 mouse sensitivity scale.
- Make the FPS overlay available outside debug builds, off by default.

### Store preparation

- Draft Steam store copy, capsule artwork, and achievement icons. These materials are not published to Steam.

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

[Unreleased]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/alexjercan/the-specimen-bevy/tree/v0.1.0
