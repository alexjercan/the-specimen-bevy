# Changelog

## [Unreleased]

## [0.3.1] - 2026-10-11

### Distribution

- Add normal and Halloween archives to one versioned GitHub Release, with separate uploads to itch.io channels and a selectable GitHub Pages build flavor.

## [0.3.0] - 2026-10-11

### Insane mode

- Add Custom mode after the first escape, with four three-step monster controls and four gameplay toggles. Winning with settings identical to Insane awards its hidden achievement and unlocks the Insane selector. Save the last selected mode and Custom options across native and browser restarts.
- Add Insane mode, unlocked after all eight visible achievements or by the hidden Insane-win achievement. After the unlock, a `<  Normal  >` mode selector appears next to **Play** on the main menu; **Retry** keeps the mode of the last run.
- In Insane mode, the monster walks at 1 m/s, chases at twice sprint speed, and hears walking at 6 m and sprinting at 18 m. Sprint stamina stays empty, flashbang and detector pickups are absent, and the power is out from the start with no boiler repair. Normal mode is unchanged.
- Add "Escaped" for any successful escape, with a first-win icon; require it alongside the other visible achievements to unlock Insane mode.
- Add the hidden achievement "Out of your mind" for escaping in Insane mode. The viewer shows it as `?` until it unlocks.

### Facility art

- Reorient the prep-room and main-menu workbenches. Place fuses at the visible front of randomly selected workbenches while fixed room dressing stays the same across runs.
- Add composed, nonblocking tabletop and floor decorations, hallway plants, larger walk-through tipped chairs, filled shelf bins, an open crate, and an open-drawer file cabinet. Give every crawl vent either a bolted cover or a nearby floor grille.
- Add an orbiting prop gallery with furniture, pickups, devices, wall and doorway variants, boiler-room pipes and tool panels. Fix the lab console's upright panel and boiler valve face flicker.
- Replace the old facility gallery, layout, and probe examples with `facility_shots`, which captures top-down, overhead, and room views of the current dressed floor under `target/facility-shots/`.
- Add two walk-through floor tool piles and stock two more tables with toolboxes, keeping props fixed across runs.

### Halloween theme (opt-in)

- Add the `halloween` Cargo feature. It adds sparse lit jack-o'-lanterns and small pumpkins on facility and main menu props, a candy-striped fuse model with the same blade terminals, and glowing orange monster eyes. Gameplay, collision, and monster behavior are unchanged, and builds without the feature look the same as before.
- Add `halloween_gallery`, an orbiting gallery of the Halloween models, placements, and monster eyes. `prop_gallery` now shares its gallery code.
- Dim facility lights only in Halloween builds and add more jack-o'-lanterns. Uncarved pumpkins match the lantern body size without a glowing face or light. Remove the artificial fog/ambient color tint; normal builds retain their lighting.
- Add jars, candles, and lanterns to tables, crates, and safe floor corners. Dress every room label with spider-web trim and a spider without covering its text.

### Gameplay and debug tools

- Show mouse sensitivity as 0.0-4.0 in Settings, with 1.5 at the default sensitivity.
- Send the monster toward the farthest reachable room at least 24 meters away after several local patrol legs; it sometimes runs there to avoid lingering near a hiding player.
- Prevent the monster from stopping at its attack distance while a doorway blocks sight to a witnessed hiding spot; it can cross the office door and finish the encounter.
- Let the monster hear moving players through walls based only on distance and the selected walking or sprinting hearing range; sight and capture still require clear paths.
- Let every flashbang in a stack launch, follow its own world-space gravity arc, and burst at its landing point without following the player. Each confirmed hit renews protection from that monster; misses give no protection.
- Show the white flash overlay only when the player faces a nearby, unobstructed burst, independently of monster-hit protection.
- Add quick-access debug cheats for fuse and flashbang counts, detector, flashlight charge, sprint stamina, and facility power.
- Add an F12 monster tracking toggle with a through-wall monster marker and floor-level navigation lines. Different colors mark walking, running, long-range targets, and the last sensed player position without changing monster behavior.
- Pause gameplay, timers, and world audio while the F12 inspector is open, then resume on close unless the pause menu is still open.

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

[Unreleased]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/alexjercan/the-specimen-bevy/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/alexjercan/the-specimen-bevy/tree/v0.1.0
