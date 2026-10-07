# Integrate spatial SFX, UI sounds and ambience

- STATUS: OPEN
- PRIORITY: 50
- TAGS: backlog, audio

## Delivery / research

Based on approved audio prototypes, add footsteps by movement/surface, door/latch, pickup/panel, flashlight, menu, boiler and ambient loops. Keep UI cues non-spatial and world cues spatial; tune loudness, concurrency and pause/headless behavior. Retain provenance and test event-to-sound triggering without relying on audio output. Depends on audio research.

## Approved catalog selection (2026-10-07)

The approved runtime audio consists of 22 clips: subway step takes 01, 02,
and 04, three edited recorded-door cues, four hiding cues, and current
objective, UI, and ambience cues. The HTML catalog uses an explicit allowlist. All uncataloged
audio, including the source recordings for the rendered doors, was removed
at the user's request; source URLs, license claims, and hashes remain in
`art/sounds/README.md`. Matching copies are in `assets/sounds/`.

## Hiding audio and ambience review

The user approved all four hiding clips (locker open/close and table enter/leave).
Source WAVs remain under `art/sounds/review/hiding/`; byte-identical runtime
copies are in `assets/sounds/hiding/`. Gameplay emits a cue on each enter/leave
transition, and core maps it to `game_audio` for positional playback; Rust
compilation and runtime mix remain unchecked. The source archive, license claim,
and per-file rendered hashes are in `art/sounds/README.md`.

Seven original ambience candidates are under `art/sounds/review/amb/` for
listening, not runtime playback. The HTML catalog groups audio by the inventory's
lettered sections; yellow cards mark these unapproved candidates. Existing
approved objective, UI, step, door, and ambience cues remain unchanged.
Catalog tests cover per-file provenance and review labels; gameplay audio checks
and in-game audition remain outstanding.

## Integration draft (unvalidated)

`game_assets` now declares explicit audio paths, and `game_audio` handles
playback, ambience, spatial attenuation, and sink pause. Gameplay emits
asset-independent door/fuse/panel sound messages. Core glue maps these
messages and player movement to audio, picks among the three approved steps,
adds the player listener, and maps menu actions/hover. `bevy_rand` 0.15
ChaCha8 replaces handwritten fuse selection and footstep random sequences;
explicit `--seed` retains reproducible fuse selection and seeded windowed
step sequences. No Rust compilation, Rust tests, or game/audio runtime review
has been performed after this draft. Footstep cadence, sound levels, spatial
mix, menu transitions, audio concurrency, and missing-asset handling need
validation. The boiler loop is loaded but not yet wired to a positional
emitter; flashlight and monster audio await those gameplay features.

## Starter sound catalog (historical prototype)

The user approved original starter cues plus a browsable catalog before game audio integration. `scripts/generate_sounds.py` produces 11 deterministic 48 kHz mono PCM WAV prototypes (walk/run tile steps, door unlatch/swing/shut, fuse pickup, panel install, UI hover/press, room tone, boiler rumble) in `art/sounds/generated/`. `scripts/build_sound_catalog.py` scans `art/sounds/` and `assets/sounds/`, groups files by category folder, draws WAV peak waveforms and offers browser playback in `art/sounds/catalog.html`. For compressed files it lists a playback link but cannot draw a waveform without a decoder. No game SFX hooks, audio buses, spatial emitters, pause handling or runtime audio are included in this first prototype. Two Python catalog/generator tests and the facility_layout example compile check passed; no browser or game was run. Audition and approve the prototypes before shipping or integrating them. Source and provenance are recorded in `art/sounds/README.md`.

## Sound inventory research (step 1, 2026-10-07)

Research only. No sound is generated, sourced, downloaded or wired in. No
command, build, game run or test was used for this note. Facts come from a
read of the repository files that are named below. Values for loudness, range,
cadence and concurrency are proposals for tuning, not measurements.

Overlap: `tasks/20261007-172900/TASK.md` (audio palette and Python
generation research) covers the same source question. This note gives the
cue inventory for integration. Task 172900 must still approve the prototypes
before integration starts.

### Baseline facts

- No audio file exists in `assets/` or `art/`. `art/sounds/` has only
  `README.md`. `credits/` has no audio entry.
- No Rust code uses Bevy audio (`AudioPlayer`, `AudioSource`,
  `PlaybackSettings`, `SpatialListener`, `AudioSink`): scout search, no match.
- `Cargo.toml:33` sets `bevy = { version = "0.19.1", features = ["jpeg"] }`
  with default features on. VERIFY that the 0.19.1 defaults include
  `bevy_audio` and the Vorbis decoder before you choose a file format.
- Windowed path uses `DefaultPlugins` (`crates/core/src/lib.rs:103`).
  Headless `--norender` uses `MinimalPlugins` with no audio plugin
  (`crates/core/src/lib.rs:89-98`). Headless tests cannot hear sounds by
  design. Tests must assert a cue-event record, not audio output.
- The camera is on the `PlayerController` entity
  (`crates/gameplay/src/controller/player.rs:115-123`). Spawn is at eye height
  1.6 (`crates/core/src/lib.rs:23,161-163`). The `SpatialListener` goes on this
  entity. Headless spawns no camera (`.without_camera()`), so listener
  insertion must not depend on the camera in tests.
- Pause freezes `Time<Virtual>` (`crates/core/src/menu/pause.rs:47-53`). An
  audio sink does not follow virtual time. The audio code must pause or duck
  world sinks on `OnEnter(PauseState::Paused)` and resume them on exit.
- `art/sounds/README.md` gives the direction: an inhabited empty building,
  dread from facility sound and silence instead of continuous music, a
  restrained 40-80 Hz layer (no infrasound claims), five mix states (Explore,
  Suspicion, Investigate, Chase, Relief), three families
  (navigation/interaction, facility mechanisms, pursuer), several variants
  for each short event, a stinger loudness ceiling, and visual or subtitle
  backup for critical threat cues.

### Trigger status key

- IMPL: the trigger exists in code now.
- FLUX: another worker edits this now (fuse panel, EXIT lock, win state,
  `tasks/20261007-172857/TASK.md`). Do not hook these until that task lands.
- PLAN: feature is not implemented (flashlight 172855, hiding 172858,
  blackout 172859, monster 172902/172903).

Priority key: MVP-A (hooks IMPL triggers), MVP-B (objective cues that need
FLUX to land), L1 (with flashlight, hiding, blackout), L2 (monster, music,
polish).

Source key:
- SYN: original Python synthesis (numpy/scipy, offline render to file).
- REC: original recording by the user (foley).
- LIC: third-party source with a verified per-file license (CC0 preferred).
- HYB: REC or LIC raw material processed by an original Python script.

### Implemented hook points

| Hook | Location | Notes for audio |
| --- | --- | --- |
| Move input and speed | `controller/player.rs:177-211` | XZ only. Walk 3.0 m/s, run 6.0 m/s (`:11-12`). No crouch, jump, land, stairs or slopes. Step cue must come from distance travelled. |
| Collision slide | `controller/collision.rs` `move_player` | No contact event. A bump cue needs a new signal (requested delta vs applied delta). |
| Floor per room | `levels/builder.rs:8-9`, `first_floor_builder/rooms.rs` | `Floor` holds a texture name only: `floor_tile` or `floor_tile_marked`. Both are the same surface. No surface type exists. |
| Door toggle | `levels/doors.rs` | `ToggleDoor` message and `DoorState::{Closed, Open}`. A new `DoorLock` guards only the outside EXIT door, including direct toggle messages. Objective implementation is uncompiled and untested. |
| Door swing | `levels/animation.rs:7-24` | 90 degrees in 0.45 s. No "settled" event. Detect arrival at target. Reverse mid-swing is possible. |
| Fuse pickup | `levels/fuses.rs:43-65` | Same observer increments `FuseInventory` and despawns the fuse. Read the position before despawn, or emit a message from this observer. |
| Interact hint | core `glue` interaction hint plugin | Text `OPEN` / `CLOSE` / `PICK UP FUSE`. Visual only. |
| Fuse HUD | `crates/ui/src/hud.rs` via core `glue` fuse HUD plugin | Slot fill on inventory change. |
| Menu buttons | `crates/ui/src/widgets.rs` `paint_buttons`, `crates/core/src/menu/mod.rs:54-73` | `Changed<Interaction>`: hover and press. `Quit` writes `AppExit` at once, so a click sound is cut. |
| States | `crates/core/src/menu/mod.rs:16-30`, `crates/core/src/lib.rs:25-30` | `CoreState::{Loading, Ready}`, `GameState::{MainMenu, Playing}`, `PauseState::{Running, Paused}`. Esc toggles pause (`pause.rs:26-37`). |
| Light effects | `levels/builder.rs:95-151` | `Flicker(phase)` is deterministic: factor 0.06 or 0.5 or 1.0 from `sin(t*1.3+p) + 0.6 sin(t*4.1+2p)`. `Pulse` is `0.55 + 0.45 sin(2t)`, period about 3.14 s. Audio can use the same function for sync. |

Fixed world anchors (from `first_floor_builder/lights.rs` and `props.rs`):

- `boiler_unit` at (-10, 0, 0) in `boiler`, with FIRE light, `Flicker(-7.0)`.
- `concept_containment_tank` at (0, 0, 0) in `lab`, SPECIMEN light,
  `Flicker(0.0)`. `lab_console` at (-3.1, 0, -2.5).
- `pipe_manifold` at (-13.65, 1.6, 0) in `boiler`.
- `wall_vent` at (-13.65, 0.45, -1.1) boiler, (-13.65, 1.9, -20) maintenance,
  (13.65, 1.9, -21.25) security_link. `vent_grille` (-13.05, 0, 3) and
  `concept_crawl_vent` (-13.65, 0.55, 3.1) in boiler.
- Lit `ceiling_light_cool` x10 (4 flicker), `ceiling_light_amber` x5,
  `ceiling_light_dead` x8 (no light, keep silent).
- `wall_lamp_red` at (13.65, 2.2, -12.5) in `storage`, `Pulse`.
- `exit_sign` at (0, 2.62, -31.15) and `fuse_panel` at (5, 1.85, -31.15) in
  `exit`, beside the outside door "exit / outside" at (0, -31.25).
- `concept_locker` x6 and `concept_table` x4: hiding candidates (PLAN).
- Rooms with `wall_conduit` walls: maintenance, utility, prep, west_hall,
  west_link, intake, boiler. These suit a more mechanical bed than the
  plain-wall rooms (exit, security, service, office, security_link, hiding,
  reception, east_hall, storage, east_link, lab).

### Mix conventions (proposal)

Cue ID form: `<family>.<cue>[.<variant>]`, for example `door.latch.03`.

Buses and suggested starting gain relative to master:

| Bus | Content | Space | Start gain | Paused |
| --- | --- | --- | --- | --- |
| `self` | Player steps, cloth, breath, flashlight click | Non-spatial, near-centre, small random pan | -6 dB | Pause sinks |
| `world` | Doors, pickups, panel, props, monster | Spatial, mono source | -4 dB | Pause sinks |
| `ambience` | Room tone, machine loops, buzz | Spatial loops plus one non-spatial bed | -14 dB | Keep bed at -12 dB extra and low-passed, pause spatial loops (VERIFY filter support) |
| `ui` | Menu, HUD confirm, completion | Non-spatial, stereo | -8 dB | Play |
| `score` | Drone, pulse, stingers (L2) | Non-spatial | -16 dB | Pause |

Loudness targets (render-time, before bus gain): one-shot peak at or below
-6 dBFS; UI peak at or below -9 dBFS; loop beds about -30 to -36 LUFS
integrated after render; stinger ceiling -3 dBTP and no louder than the
loudest door slam in practice. Check at low volume and on laptop speakers
(README rule).

Variation policy: random choice without immediate repeat, pitch jitter
+/-3 to 6 percent, gain jitter +/-1.5 dB. Use the run seed only in tests so
choice is deterministic; the cue log records cue ID, not the variant, so tests
stay stable.

Voice budget: Bevy does not cap voices. Enforce a global cap (start 24),
per-cue caps (see tables) and a priority order: UI > objective > self > door
> monster > ambience one-shots > loops. Steal the oldest, quietest voice of
equal or lower priority.

Attenuation: VERIFY what the Bevy 0.19.1 spatial sink does. The expected
model is simple left/right ear gain by distance with no rolloff control and
no filter. Plan an app-side gain curve: full gain inside `ref` distance,
linear-in-dB fall to silence at `max` distance, stop or pause loops beyond
`max`. Approximate occlusion with the room graph (`DoorOf`/`Doors`/`Passage`):
same room 0 dB; next room through an open door or passage -6 dB; through a
closed door -14 dB (plus a low-pass if the backend supports it); two or more
rooms away -24 dB or mute. Recompute when a door state changes or the
listener changes room. Visual geometry and audio direction must agree.

Loops: render 8-30 s seamless loops with crossfaded seams; give each loop a
length that is not a simple ratio of another loop length, to prevent audible
phase patterns. Mono for spatial sources, stereo for beds and UI.

Format proposal: 48 kHz render, ship OGG Vorbis (VERIFY decoder feature),
keep the lossless WAV master and the script outside `assets/`.

### A. Player self (bus `self`)

| ID | Cue | Trigger / context | Status | Variants | Type | Concurrency | Priority | Source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `step.walk.tile` | Footstep on hard tile, soft shoe | Every 1.4 m of applied XZ travel while walking (about 2.1 steps/s at 3.0 m/s). Alternate L/R pan +/-0.1. | IMPL (needs step accumulator) | 8 | One-shot | 1, retrigger cuts tail | MVP-A | REC best; SYN fallback: heel transient (short low-passed noise, 80-200 Hz body thump) plus toe tick 60-90 ms later, random spectral tilt |
| `step.run.tile` | Heavier, faster step with scuff | Every 1.9 m while running (about 3.2 steps/s). +2 dB, brighter. | IMPL | 8 | One-shot | 1 | MVP-A | Same as walk, more energy, add 40 ms scuff noise |
| `step.stop` | Settle scuff | Movement goes from moving to zero after at least 0.5 m of travel | IMPL | 4 | One-shot | 1 | L1 | REC or SYN filtered-noise scuff |
| `step.start` | Weight shift | Zero to moving | IMPL | 3 | One-shot | 1 | L2 | Same |
| `step.wall_bump` | Soft shoulder or hand on wall | Applied delta much smaller than requested delta for more than 0.15 s; cooldown 1 s | IMPL (needs signal) | 3 | One-shot | 1 | L2 | REC; SYN muffled thump |
| `step.surface.debris` | Paper rustle or grit under foot | Step within 0.6 m of `clutter_papers` or `clutter_tools` | IMPL anchors, needs proximity | 4 | One-shot layered on step | 1 | L2 | REC paper; SYN crackle (Poisson clicks through band-pass) |
| `step.surface.wet` | Wet scuff | Step near `drum_spilled` (-5.6, 0, -22) | IMPL anchor | 3 | Layer | 1 | L2 | REC |
| `self.cloth` | Jacket rustle | Look turn rate above a threshold, or start of run | IMPL | 4 | One-shot | 1, cooldown 0.8 s | L2 | REC |
| `self.breath.exert` | Faster breathing | After 4+ s continuous run, fade out over 3 s after stop | IMPL | Loop 2 takes | Loop | 1 | L1 | REC (voice); avoid synthetic breath |

Decision: one surface for MVP (all floors are tile). If more surfaces come
later (grate, metal stairs, outside asphalt), add a surface tag to `Floor`
instead of parsing texture names.

### B. Doors (bus `world`, spatial at door hinge or panel centre)

| ID | Cue | Trigger / context | Status | Variants | Type | Range ref/max | Concurrency | Priority | Source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `door.unlatch` | Handle and latch release click | `ToggleDoor` read while `Closed` | IMPL | 5 | One-shot | 1 / 18 m | 1 per door | MVP-A | REC (any household latch) best; SYN modal metal: 3-6 damped partials 1.5-6 kHz, 5-30 ms decay, plus a small noise transient |
| `door.swing.open` | Hinge creak and air move, 0.45 s | Same frame as unlatch, length matches swing | IMPL | 5 | One-shot (time-stretched to 0.45 s) | 1 / 15 m | 1 per door; restart on reverse | MVP-A | SYN stick-slip creak: jittered pulse train 20-80 Hz rate into resonant band-pass 400-1800 Hz, slow pitch drift; plus band-passed noise whoosh |
| `door.swing.close` | Shorter creak, different pitch | `ToggleDoor` read while `Open` | IMPL | 5 | One-shot | 1 / 15 m | 1 per door | MVP-A | Same generator, other seed |
| `door.stop.open` | Soft bump at full open | `DoorSwing` reaches pi/2 | IMPL (detect arrival) | 3 | One-shot | 1 / 10 m | 1 per door | MVP-A | SYN low thump plus wood/metal partials |
| `door.shut` | Latch catch and frame thud | `DoorSwing` reaches 0 from above | IMPL | 6 | One-shot | 1.5 / 25 m | 1 per door | MVP-A | REC or HYB; SYN: 60-120 Hz thump, latch click layered 10 ms later |
| `door.reverse` | Short creak stop | Toggle while swing is between 0 and pi/2 | IMPL | 3 | One-shot, cuts current swing | 1 / 12 m | 1 per door | L1 | SYN |
| `door.locked.rattle` | Handle rattle, no movement | F on EXIT outside door while locked | FLUX (new lock/hint code unverified) | 4 | One-shot | 1 / 15 m | 1, cooldown 0.4 s | MVP-B | REC; SYN rattle: 3-5 metal ticks in 200 ms |
| `door.unlock.mag` | Magnetic lock or bolt release clunk, low hum stops | EXIT door unlock after panel install | FLUX | 2 | One-shot | 1.5 / 30 m | 1 | MVP-B | SYN relay clunk plus 50/60 Hz hum release |
| `door.exit.outside_air` | Wind and wide air when outside door opens | EXIT outside door swing > 0 | FLUX | Loop 1 | Loop, gain by swing angle | 2 / 20 m | 1 | MVP-B | SYN filtered pink/brown noise wind with slow gust LFO |

Per-door identity: derive a small fixed pitch offset (+/-4 percent) and seed
from the door `Name`, so each door has a stable voice. Passages have no cue.
Repeated F spam must not stack: one voice per door and per cue.

### C. Objective, pickup and panel (`world` plus `ui`)

| ID | Cue | Trigger / context | Status | Space | Variants | Concurrency | Priority | Source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `fuse.pickup` | Small glass/ceramic fuse clink and hand grab | `pick_up` observer, at fuse position before despawn | IMPL | Spatial, ref 0.5 / max 8 m | 4 | 1 | MVP-A | SYN modal clink (glass-like partials 2-7 kHz, 50-150 ms) plus cloth grab noise |
| `ui.fuse.slot` | HUD slot fill confirm, rising by count (1/3, 2/3) | `FuseInventory` increases | IMPL | Non-spatial | 3 (one per count) | 1 | MVP-A | SYN soft sine/triangle blip with short decay, pitched steps |
| `ui.fuse.complete` | All 3 held, clear "objective ready" | `FuseInventory` reaches `FUSE_COUNT` | IMPL | Non-spatial | 1 | 1 | MVP-A | SYN, two-note figure, not a reward fanfare |
| `fuse.idle.hum` | Faint electrical tick near an uncollected fuse | Optional locator loop at each fuse table | IMPL anchors | Spatial, ref 0.5 / max 4 m | Loop 1 | 3 | L2 | SYN; decide if it helps or makes search too easy |
| `panel.hum` | Fuse panel idle buzz with missing fuses | Loop at `fuse_panel` (5, 1.85, -31.15) | FLUX | Spatial, ref 0.5 / max 8 m | Loop 1 | 1 | MVP-B | SYN 50/60 Hz harmonics plus sparse arc crackle |
| `panel.denied` | Dull buzz or empty-socket clack | F at panel with fewer than 3 fuses (only if the panel accepts this input) | FLUX | Spatial | 2 | 1, cooldown 0.5 s | MVP-B | SYN |
| `panel.install` | Sequence: 3 fuse seats (click, click, click, 0.25 s apart), breaker lever thunk, relay chatter, power hum rises | Single panel activation | FLUX | Spatial at panel | 2 | 1, not restartable | MVP-B | HYB: REC lever plus SYN relay ticks and hum swell |
| `panel.power_on` | Lights or exit sign brighten tick | End of install | FLUX | Spatial | 1 | 1 | MVP-B | SYN |
| `run.exit_cross` | Exterior air swells, interior bed drops | Player crosses the outside threshold | FLUX | Bed crossfade | 1 | 1 | MVP-B | SYN wind bed (see `door.exit.outside_air`) |
| `ui.complete` | Completion screen tone, quiet and final | Completion screen spawn | FLUX | Non-spatial | 1 | 1 | MVP-B | SYN low pad swell 2-4 s |

Headless and transport: the win state stays queryable in the snapshot
(172857). Cue logging must use the same events so a test can assert
`panel.install` once and `run.exit_cross` once without audio output.

### D. Facility ambience (bus `ambience`)

| ID | Cue | Anchor / context | Status | Variants | Type | Range ref/max | Concurrency | Priority | Source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `amb.roomtone` | Global quiet room tone and far HVAC | Non-spatial bed, always on in play | IMPL | 2 beds (conduit rooms, plain rooms), crossfade 1.5 s by listener room | Loop 30 s | n/a | 1-2 during crossfade | MVP-A | SYN brown/pink noise, gentle band-pass, slow LFO; or HYB from a REC of a quiet room |
| `amb.low` | 40-80 Hz pressure layer | Under the bed, stronger near boiler | IMPL | 1 | Loop | n/a | 1 | L1 | SYN: two detuned sines 45 and 47 Hz plus filtered noise; test on small speakers; offer reduce-bass option |
| `amb.boiler.rumble` | Boiler body rumble and burner roar | `boiler_unit` (-10, 0, 0) | IMPL | 1 | Loop 20 s | 2 / 14 m | 1 | MVP-A | SYN low noise through resonances plus burner noise band 300-900 Hz with turbulence LFO |
| `amb.boiler.fire` | Fire crackle synced to FIRE flicker | Same anchor, gain follows `LightEffect::factor` | IMPL | 1 | Loop | 1 / 6 m | 1 | L1 | SYN Poisson crackle |
| `amb.boiler.tick` | Thermal pings and knocks | Random interval 6-20 s at boiler or `pipe_manifold` | IMPL | 6 | One-shot | 1 / 18 m | 1 | MVP-A | SYN modal metal ping, long decay 0.5-1.5 s |
| `amb.pipe.flow` | Water or steam in pipes | `pipe_manifold` (-13.65, 1.6, 0) | IMPL | 1 | Loop 15 s | 1 / 8 m | 1 | L1 | SYN filtered noise with bubbling |
| `amb.vent.hvac` | Vent airflow and fan rattle | 3 `wall_vent` anchors | IMPL | 2 loop takes | Loop 17 s and 23 s | 1 / 10 m | 3 | MVP-A | SYN noise plus fan blade tone (20-40 Hz rate amplitude ripple), rattle hits |
| `amb.vent.rattle` | Loose grille rattle burst | Random 10-40 s at a vent, `vent_grille`, `concept_crawl_vent` | IMPL | 5 | One-shot | 1 / 15 m | 1 | L1 | SYN metal rattle; NOTE: later reuse for monster vent movement, so keep the ambient version clearly lighter |
| `amb.light.buzz` | Fluorescent ballast buzz | Each lit `ceiling_light_cool` and `ceiling_light_amber` | IMPL | 2 (cool, amber) | Loop 11 s | 0.5 / 5 m | Nearest 4 only | MVP-A | SYN mains harmonics (100/120 Hz and up) plus high-frequency hiss; mains choice is a decision |
| `amb.light.flicker` | Arc tick and buzz dropout | Flicker lights; when factor falls to 0.5 or 0.06 | IMPL | 4 ticks | One-shot plus gain sync on buzz | 0.5 / 7 m | Nearest 2 | MVP-A | SYN click and short crackle |
| `amb.alarm.pulse` | Low fault tone synced to red lamp | `wall_lamp_red` in storage, gain follows `Pulse` | IMPL | 1 | Loop, period about 3.14 s | 1 / 9 m | 1 | L1 | SYN filtered square or sine at low level; avoid a real alarm siren unless it means danger |
| `amb.tank.hum` | Containment tank pump hum and bubbles | `concept_containment_tank` (0, 0, 0) | IMPL | 1 | Loop 19 s | 1 / 9 m | 1 | MVP-A | SYN hum plus Minnaert bubbles (short rising chirped sines 300-1500 Hz) |
| `amb.lab.console` | Console fan and rare beep | `lab_console` (-3.1, 0, -2.5) | IMPL | Loop 1 plus 3 beeps | Loop and one-shot | 0.5 / 6 m | 1 | L1 | SYN |
| `amb.exit_sign.hum` | Faint transformer hum | `exit_sign` | IMPL | 1 | Loop | 0.3 / 3 m | 1 | L2 | SYN |
| `amb.distant` | Building settle creak, far metal knock, drip | Random 25-60 s, at a random point 2+ rooms away; no threat-like cue while no threat can exist (README) | IMPL | 10 | One-shot | n/a (room-graph gain) | 1 | L1 | SYN plus HYB |
| `amb.drip` | Water drip in a fixed place | Near `drum_spilled` or `maintenance` | IMPL | 4 | One-shot, random 2-6 s | 0.5 / 6 m | 1 | L1 | SYN: short sine drop with pitch fall; or REC |
| `amb.outside.wind` | Wind behind the outside door | Exit room, gain rises when "exit / outside" opens | FLUX | 1 | Loop | 2 / 12 m | 1 | MVP-B | SYN |

Dead ceiling lights stay silent. Silence near dead lights is a design tool.

### E. UI and front end (bus `ui`, non-spatial)

| ID | Cue | Trigger | Status | Variants | Concurrency | Priority | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `ui.hover` | Very short soft tick | `Interaction::Hovered` on a menu button | IMPL | 2 | 1, cooldown 60 ms | MVP-A | SYN short filtered click |
| `ui.press` | Firm click | `Interaction::Pressed` | IMPL | 2 | 1 | MVP-A | SYN click plus low body |
| `ui.play` | Transition into the facility, bed fades in | `MenuAction::Play`, enter `GameState::Playing` | IMPL | 1 | 1 | MVP-A | SYN low swell |
| `ui.pause.open` | Muffle sweep, world ducks | `OnEnter(PauseState::Paused)` | IMPL | 1 | 1 | MVP-A | SYN; snapshot change matters more than the cue |
| `ui.pause.close` | Reverse sweep | `OnExit(PauseState::Paused)` | IMPL | 1 | 1 | MVP-A | SYN |
| `ui.mainmenu` | Back to menu | `MenuAction::MainMenu` | IMPL | 1 | 1 | L1 | Reuse `ui.press` |
| `ui.quit` | None, or accept cut | `MenuAction::Quit` writes `AppExit` at once | IMPL | 0 | n/a | n/a | Do not add a delay to quit only for sound |
| `ui.menu.bed` | Main menu low drone | `GameState::MainMenu` | IMPL | 1 | 1 | L1 | SYN drone from processed room tone |
| `ui.loading` | Silent, or very low bed | `CoreState::Loading` | IMPL | 0-1 | 1 | L2 | Audio assets may still be loading here, so keep it silent |
| `ui.hint.appear` | Faint tick when the interact hint appears | Hint target changes from none to some | IMPL | 1 | 1, cooldown 0.3 s | L2 | SYN; risk of noise, test before keeping |

### F. Flashlight (PLAN, 172855)

| ID | Cue | Trigger | Space | Variants | Concurrency | Priority | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `flash.on` | Rubber switch click | Toggle on | `self` | 4 | 1 | L1 | REC best (any torch); SYN two-part click |
| `flash.off` | Slightly lower click | Toggle off | `self` | 4 | 1 | L1 | Same |
| `flash.spam` | No extra cue; same clicks with 80 ms limit | Fast toggles | `self` | n/a | 1 | L1 | n/a |

No battery, so no low-battery cue. If the light is LED, keep it silent while on.

### G. Hiding (PLAN, 172858)

| ID | Cue | Trigger | Space | Variants | Priority | Source |
| --- | --- | --- | --- | --- | --- | --- |
| `locker.open` | Thin sheet metal door, hinge squeal | Enter or leave a `concept_locker` | `world` spatial, ref 1 / max 20 m | 4 | L1 | REC; SYN modal sheet metal plus creak generator |
| `locker.close` | Metal clatter, latch | After entry or exit | `world` spatial, max 25 m | 4 | L1 | REC |
| `hide.enter.locker` | Body shuffle against metal | Enter state | `self` | 3 | L1 | REC |
| `hide.table.crawl` | Floor scrape and cloth | Enter or leave under `concept_table` | `self` | 3 | L1 | REC |
| `hide.inside` | Mix snapshot: world low-passed and -6 dB, own breath louder | While hidden | Snapshot | n/a | L1 | Backend filter support needed |
| `hide.breath.held` | Controlled breathing | While hidden and a threat is near (needs monster) | `self` | Loop 2 | L2 | REC voice |

Noise from hiding props is a later monster detection input. Keep their
loudness in the cue table data so AI can read the same value.

### H. Blackout and boiler recovery (PLAN, 172859)

| ID | Cue | Trigger | Space | Priority | Source |
| --- | --- | --- | --- | --- | --- |
| `power.fail` | Relay clunk, hum spin-down, all light buzz stops, HVAC winds down over 2-3 s | Blackout start | `world` at boiler plus global bed change | L1 | SYN: pitch-down of hum harmonics, noise filter close |
| `power.dark.bed` | Changed room tone, no HVAC, more building creak | During blackout | Non-spatial bed | L1 | SYN |
| `power.backup` | Readable backup cue: a slow emergency beep or a battery lamp click near the route | Blackout start and every N s | Spatial at route anchor | L1 | SYN; must pair with a visual cue (task asks for readable backup cues) |
| `boiler.valve` | Valve wheel turn, metal squeak | Boiler reset interaction | Spatial | L1 | REC or SYN creak |
| `boiler.ignite` | Igniter clicks (3-6) then burner whoomp | Reset success | Spatial | L1 | SYN click train plus low noise burst |
| `boiler.fail_try` | Igniter clicks without catch | Failed or early reset (if design has it) | Spatial | L2 | SYN |
| `power.restore` | Hum spin-up, lights strike flutter in sequence, HVAC returns | Recovery | Spatial plus bed | L1 | SYN reverse of `power.fail` structure, not a reversed file |

### I. Monster (PLAN, 172902 and 172903; design not approved)

| ID | Cue | Context | Space | Priority | Source |
| --- | --- | --- | --- | --- | --- |
| `mon.step` | Heavy, irregular footfall, distinct from the player | Monster moves; rate by speed | Spatial, max 30 m, room-graph gain | L2 | HYB from REC impacts, pitched down; must not be mistaken for player steps |
| `mon.drag` | Drag or scrape | Matches `trace_drag_marks` story | Spatial | L2 | HYB |
| `mon.claw` | Claw on metal or wall | Matches `trace_claw_marks` | Spatial | L2 | HYB |
| `mon.breath` | Breathing or low vocal | Near or searching | Spatial, max 12 m | L2 | REC voice processed; or LIC with CC0 check |
| `mon.door` | Door bang or forced door | Monster contacts a door | Spatial at door | L2 | Reuse door chain, heavier |
| `mon.vent` | Movement in vents | Vent routes, if any | Spatial at vents | L2 | HYB; keep apart from `amb.vent.rattle` |
| `mon.search` | Suspicion cue (one distant knock or step at a real location) | Suspicion state (README table) | Spatial | L2 | Only when a threat can exist |
| `mon.detect` | Short alert cue when monster commits to chase | Investigate to chase | Non-spatial plus spatial vocal | L2 | Needs visual or subtitle backup |
| `mon.catch` | Catch stinger | Player caught | Non-spatial, ceiling -3 dBTP | L2 | Rare and earned; no extreme level |
| `score.chase` | Industrial pulse bed | Chase state | `score` | L2 | SYN from processed facility sounds |
| `score.relief` | Drone and pulse recede | Relief state | `score` | L2 | SYN |

### J. Score and drone (L2)

Follow the README: no continuous music. Use a drone bed made from processed
facility sound (`amb.roomtone` stretched and filtered) with rare dissonant
notes, crossfaded by mix state. One rare stinger for an earned reveal.

### Source approach and provenance

- SYN first for MVP-A: clicks, latches (modal), creaks (stick-slip), thumps,
  hums, buzz, noise beds, drips, bubbles, crackle and UI tones are all
  practical with numpy/scipy and an original script. Store each script and
  its seed with the output. A file is original only when the script and all
  inputs are original; do not use third-party sample libraries or impulse
  responses inside SYN. Build any reverb from a synthetic impulse response.
- REC for footsteps, cloth, breathing, locker metal and lever foley. These are
  hard to make convincing by synthesis. Record several takes, remove handling
  noise, keep the raw take and date.
- LIC only when SYN and REC fail. Use the README source list as candidates
  only. Check the individual file page, author, license and version. Prefer
  CC0; CC-BY needs an attribution entry in `credits/`; do not use NC or ND.
  Do not follow third-party credit links blindly. Nothing was downloaded for
  this note.
- Ledger: one row per shipped file in `credits/ASSET-SOURCES.md` (or a new
  audio ledger, decision): cue ID, file, method (SYN/REC/LIC/HYB), script path
  and seed or recordist and date, raw source URL and license for LIC/HYB,
  attribution text.

### Test strategy (for the integration step, not done)

- Add a cue-event message or record that game systems write in all modes
  (headless included). The playback layer reads it only when audio exists.
- Assert cue IDs, counts and positions from events in headless tests: door
  toggle gives unlatch plus swing; shut gives one `door.shut` at arrival;
  fuse pickup gives one `fuse.pickup` at the fuse position and one
  `ui.fuse.slot`; F spam does not stack voices; pause sends the pause snapshot.
- Do not assert on audio device output.

### Outstanding decisions

1. Research split resolved: keep the cue inventory in 172901. Use 172900 for
   sound-creation research and prototypes; integration in 172901 follows
   approved prototypes. Do not count the inventory as audio implementation.
2. Backend: built-in `bevy_audio` (simple; VERIFY spatial model, filters and
   default features in 0.19.1) or a third-party audio plugin with filters and
   buses (VERIFY 0.19 support and license). Occlusion low-pass, pause muffle
   and hiding muffle depend on this.
3. File format: OGG Vorbis only, or also WAV for very short UI cues.
4. Player self sounds: non-spatial (proposed) or spatial at feet.
5. Mains frequency for hum and buzz: 50 Hz (100 Hz buzz) or 60 Hz (120 Hz).
6. Current unverified panel implementation allows F interaction only when
   all 3 fuses are held; it does not provide `panel.denied` via F with fewer.
   Decide later whether a distinct missing-fuse cue or visual hint is needed.
7. Uncollected fuse locator hum (`fuse.idle.hum`): yes or no.
8. Music: drone-only score as in the README, or no score in MVP.
9. Python toolchain and location: which packages go into the Nix shell
   (numpy, scipy, a WAV/OGG writer), where scripts live (`scripts/` or
   `art/sounds/`), and whether generated files are committed or rebuilt.
10. Foley recording: will the user record footsteps, cloth, latches and
    breath, or must MVP be SYN only.
11. Pause policy: pause all world sinks (proposed) or keep a muffled bed.
12. Accessibility: master and per-bus volume settings, reduced bass option,
    and visual or subtitle backup for critical cues.
13. Ledger location: extend `credits/ASSET-SOURCES.md` or add an audio ledger.

## Status

Integration remains planned only. The selected review clips above have user
audio approval, but no sound is wired into the game and no runtime mix review
has occurred. Step 1 (sound inventory research) is written above.
