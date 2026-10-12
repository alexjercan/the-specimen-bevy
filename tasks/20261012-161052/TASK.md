# Refactor to the playbook architecture

- STATUS: OPEN
- PRIORITY: 0
- TAGS: architecture,refactor

## Source

- Design doc: Specimen Playbook, https://claude.ai/artifact/TqT6aUDWByh1uL5dPKSdsd (v22, 2026-10-12).
- Inputs: the-specimen-bevy at ba25142 (v0.3.1) compared with ~/personal/template-bevy at d0f2bfd, plus review task 20261011-115441.
- Every decision below was made by the user in review comments on 2026-10-12. Items marked "agent" are agent recommendations the user accepted.
- Scope: this repository. The playbook was written for future games; the user decided to refactor this game to match it.

## Decisions

### Modes and flags

- `--norender` becomes `--headless`. `--transport` becomes `--stdio`. Same game, same schedule, same input pipeline; stdio swaps stdin/stdout for the window loop.
- `--headless` means logic only: `MinimalPlugins`, no window, no assets, no audio plugin. `--mute` is deleted (today `src/main.rs:21`, `crates/core/src/lib.rs:47,83,145`).
- Flags after the change: `--headless`, `--stdio`, `--seed`, `--record`.
- No `--replay`. The host owns the trace and can resend it over stdin.
- Both stdio protocol lines carry a `version` field.

### Crate layout

- `app` (today `crates/core` minus menus): `AppBuilder`, CLI parsing helpers, flow states (`GameState`, `PauseState`), `FrameStep`, the game-side `impl Playable`, and `glue/` as a module directory. Not a separate glue crate.
- `frontend` (today `crates/core/src/menu` plus `lighting.rs`): main menu, pause, settings panel, cinematic, credits, achievements, loading, game over, complete. Added only in windowed mode. Depends on gameplay, ui, audio, settings, achievements.
- `gameplay`: feature directories. Each feature has a logic plugin, a view plugin and a path-free assets resource. See "Features".
- `audio`: a bevy-only bus. `PlaySound { handle, position, bus }`, pause, per-bus volume from a resource `app` sets. No game `Sound` enum, no `game_assets` dependency, no `game_settings` dependency, no facility rules.
- `ui`: generic widgets only. HUD elements belong to feature views.
- `assets`: the `bevy_asset_loader` collections with paths. Nothing else reads paths.
- `settings`, `achievements`: unchanged in role.
- `stdio` (today `crates/transport`, generic half): bevy-only crate behind `trait Playable`. Game-specific half moves into `app`.
- `capture`, `autopilot`: bevy-only, keep, see "Tooling".
- `probe`: delete.
- `halloween`, `debug`: decorators, unchanged in role; `debug` gains `FrameTimeDiagnosticsPlugin` and `LogDiagnosticsPlugin`.

Allowed edges: gameplay -> audio, ui, settings (read-only resource types). frontend -> gameplay, ui, audio, settings, achievements. app -> everything. Nothing depends on app. Nothing under `crates/gameplay` depends on `game_assets`.

### Features

- Feature axis, not functionality axis. One directory per feature under `crates/gameplay/src/`: player, monster, facility, devices, hiding, fuses, interaction, ambience, presets, achievements.
- Each feature: `logic` plugin (headless-testable, no `Handle`, no `Mesh3d`, no `PointLight`, no `Camera3d`, no `Window`), `view` plugin (spawns meshes, lights, HUD, sends `PlaySound`), `assets.rs` with a feature-owned resource of handles and no paths, `sounds.rs` where needed.
- Assets follow the settings pattern: the feature defines the resource (for example `FacilityAssets { wall: Handle<Scene>, door: Handle<Scene>, hum: Handle<AudioSource> }`), `app/glue` fills it from the loader collection. Accepted cost: two structs per feature plus one fill function.
- No game-wide sound enum anywhere. Sound handles live in feature assets structs. The 50-arm `Sound` enum and match at `crates/audio/src/lib.rs:216` go away. `gameplay/levels/sounds.rs` stops re-exporting audio enums.
- Controller stays pure (`controller/player.rs`); a Player view plugin adds camera, listener, beam light, footstep cues. Both in gameplay.
- Mains power: the facility feature owns a `PowerState` resource and tags powered entities with a `MainsPowered` component at spawn. Each consumer feature reacts in its own code. No shared `requires_power()`; the three copies (`gameplay/levels/render.rs:96`, `crates/audio/src/lib.rs:88`, `ambience.rs:36`, `core/glue/sound.rs:86,119`) are deleted. `PowerState` is read only under `crates/gameplay`.
- Local types: gameplay public items never expose library types. `KeyBindings` wraps `bevy_enhanced_input::Binding` behind a gameplay `enum Input { Key(KeyCode), Mouse(MouseButton) }` (`controller/settings.rs:2,12-13`), with one private conversion into `Binding` in the gameplay player feature (`fn binding` in `player/logic.rs`). The gamepad case that panics at `crates/transport/src/transport.rs:77` gets a variant or an explicit error. `Handle<AnimationClip>` in `levels/monster.rs` moves into the monster feature's `assets.rs`.

### Glue

- Glue is config. It copies data between features while plugins initialize and does nothing else.
- Under `app/src/glue`: no constants, no string compares, no RNG, no `Update` or `FixedUpdate` systems, no `Changed<T>`. Systems register only in `Startup` or `OnEnter`.
- What stays in glue: loader collection to feature assets fill, one-time settings copy, bus volume copy.
- What leaves glue today: `interaction_hint.rs` (becomes a widget in `ui` or part of the interaction feature view); footstep stride constants and the `wall_conduit` compare in `glue/sound.rs` (player and facility features); intermittent-sound timers with RNG (ambience feature); the `*_hud.rs` sync systems (feature views).
- Runtime settings changes: the settings panel in `frontend` writes `ControllerSettings` and the bus volume resource directly and persists through `settings`. The `Changed<GameSettings>` sync in `glue/controller_settings.rs:18-42` becomes a startup copy.

### Messages

- Lean on messages. `DoorCue`, `FlashbangHit`, `FuseCue`, `ExitOpened`, `StartRun { preset }` are gameplay messages observed by views, achievements and the stdio snapshot without touching the emitter.
- Messages are for things that happened. Resources and components are for how things are. Power stays a resource, not a `PowerChanged` stream.
- One owner per message type: the defining plugin calls `add_message`, nothing else. The five gameplay plugins re-registering `PlaySound` and `crates/transport/src/perception.rs:44-52` are the anti-pattern.
- Readers across a crate boundary declare `SystemSet` ordering or accept a one-frame delay and say so.
- The frontend sends gameplay commands; it never builds the world. World construction has one owner.

### Data cleanups

- `Res<T>`, never `Option<Res<T>>` with a fallback. The owning plugin inits the resource. Tests add that plugin. No mirror resources, no `is_plugin_added`.
- `AppBuilder` six bools become `enum Frontend { Headless, Windowed { menu } }` plus a bounded-run option. No audio switch.
- `FacilityPower` bools (`levels/power.rs:27-34`) become `enum PowerState { On { until_outage }, Outage { remaining }, Tripped }` plus `cycles`, `seed`.
- `Prop(String)`, `Walls(String)` (`levels/builder.rs:17,24,92`) become `enum Module` with `footprint()` and `light_spec()`; glue maps `Module -> asset stem`. The string matches at `collision.rs:173`, `render.rs:96`, `lights.rs:77`, `sounds.rs:37`, `glue/sound.rs:75` go.
- `SoundAssets` 55 handle fields plus the `Sound` enum become per-feature assets structs.
- General: two exclusive bools are an enum; config and runtime are two components; a unitless number is a newtype; a string matched twice is an enum.
- Every frame-step constant has one owner: `FrameStep` in `app` feeds capture and stdio. No literal 60 in two crates.

### Tooling

- stdio crate: `trait Playable { type Input: DeserializeOwned; type View: Serialize; fn ready(&World) -> bool; fn apply(&Input, &mut World); fn observe(&mut World, tick: u64) -> View; }`. `HeadlessStdio<P>` and `RenderedStdio<P>`. `Command<P::Input>` and `Reply<P::View>` with `version`. `struct Specimen; impl Playable for Specimen` lives in `app`. No panic on unknown bindings. Input injection stays `ButtonInput::press/release` plus `MouseMotion` so `bevy_enhanced_input` runs unchanged.
- Python host: `playtest/harness.py` generic, `playtest/specimen.py` adapter (today `scripts/playtest_agent.py`, `scripts/PLAYTEST.md`). Host timeout and game pending-frame timeout agree (today 600 s vs 120 s at `crates/transport/src/rendered.rs:164`).
- Capture: rename loops to video. `VideoCapturePlugin`, `video_start`/`video_end`, `VideoProfile { fps, crf, output_size, deadline, staging }` with `PREVIEW` and `FINAL` constants. Today only fps is configurable and `-crf 34` is hardcoded in `crates/capture/src/loops.rs`. Plug points: `loops.rs:19-38`, `core/lib.rs:73-76,126-134`, the `--record` flag.
- Video audio: a frame-indexed audio event log written next to staged frames (frame index, asset path via `AssetServer::get_path`, bus, volume, speed, loop or one-shot, spatial position relative to listener, retune events every fps/4 frames for live spatial sources, start and stop events for loops). A mixer in the capture crate (Rust, rodio decoders, unit tested; the dev shell has no numpy) decodes referenced files once, mixes at 48 kHz into `audio.f32le`, and the existing mux branch at `loops.rs:97-103` becomes live. Observer sits on the audio bus, the only place that spawns `AudioPlayer`. No PCM tap.
- Autopilot: builder arming only (`AppBuilder` plus `AutopilotPlugin`); no env arming. Examples never assert. Asserting autopilot scenarios live in `tests/e2e/`, at least one rendered under Xvfb in CI. Scenarios are defined in the test or example that runs them; `src/` holds only the game library and `main.rs`.
- Examples: `facility_shots`, `human_deer_viewer`, `prop_gallery`, `halloween_gallery` stay as screenshot and manual playtest tools. Compile-checked in CI, never run as tests.
- Probe: delete `crates/probe` (779 lines, unused except `world_instances_ready` in `examples/facility_shots.rs:7`; that helper moves into the example or `app`). Replace with Bevy `trace_tracy` plus a pinned nixpkgs `tracy` matching Bevy's `tracy-client`, `cargo flamegraph` with `linuxPackages.perf` in the dev shell, `trace_chrome` parsed by `scripts/perf/frametime.py` (bounded scenario, p95/p99 plus top systems by self time, deletes the trace, refuses to print GPU numbers from a headless run), and `FrameTimeDiagnosticsPlugin` plus `LogDiagnosticsPlugin` behind `--debug`. One p99 budget check as an asserting e2e scenario, generous until a real regression.
- `tests.sh` smoke asserts a clean exit, never accepts exit 124 from a timeout.

### Docs set and AGENTS.md

- Add under `docs/`: `architecture.md` (layer diagram, allowed edges, feature layout, glue rule, messages rule), `coding-conventions.md` (comment policy plus the data rules), `testing.md` (headless proves logic, rendered proves visuals and audio, examples never assert, e2e asserts, one rendered e2e under Xvfb, smoke never accepts a timeout), `tooling.md` (how to run `--headless`, `--stdio`, `--record`, `--seed`, the playtest host, capture profiles, Tracy and the perf scripts, what each output cannot show, and why the Linux release job uses apt-get), `assets.md` (art is sources, assets is runtime, provenance before distribution, fetch-generate-check-promote, credits).
- `AGENTS.md` holds the project sentence, the layout list, one pointer line per doc, and the tool table. No rule text inline.
- Each mechanical rule names its check in `tests.sh`. Remove the check, remove the line.
- Skills and agents: one skill with the paid-for rules (state the intended result, list deletions, change the owning interface first; `Res<T>` rule; examples-never-assert). `.agents/skills/game-task/SKILL.md:6` heading "Nova Task" becomes "Tatr Task". Scout and review subagent prompts get one line naming the proof tools. Worker blocks in `.pi/subagents.yaml` are deduplicated with anchors; the five copies are not carried forward.

### Comment policy

- Allowed: `NOTE:`, `XXX:`, `WTF:`, plain `TODO:`. No project marker, no ticket ID, no open-ID lookup.
- `scripts/check-comments.py:7` and `docs/coding-conventions.md:4` change in the same commit.
- A grep for `TODO:` goes into the release checklist, not the checker.

### CI and release

- Add a rendered CI job: at least one asserting e2e under Xvfb. If the audio plugin fails without an output device, add a dummy ALSA or PipeWire device to the job; never a game flag.
- Linux release job keeps apt-get and plain `cargo build` (`.github/workflows/release.yaml:44-70`). Binaries built in `nix develop` link `/nix/store` paths and are not portable. `rust-toolchain.toml` keeps the compiler identical. Write the reason in `docs/tooling.md`.
- `flake.nix` `checks.smoke` is dead; either CI runs it or it goes.

## Grep gates (become `tests.sh` steps, each named in its doc)

- `Option<Res<` anywhere in `crates` and `src`: fail.
- `is_plugin_added`: fail.
- `#[path]` includes: fail.
- `Handle<` under `crates/gameplay` outside `*/view.rs`, `*/sounds.rs`, `*/assets.rs`: fail.
- `bevy_asset_loader` or string asset paths under `crates/gameplay`: fail.
- `game_assets` in `crates/gameplay/Cargo.toml`: fail.
- `bevy_enhanced_input` types in `pub` items under `crates/gameplay`: fail.
- Under `app/src/glue`: `const `, `== "`, `Rng`, `rand`, `Update`, `FixedUpdate`, `Changed<`: fail.
- `PowerState` read outside `crates/gameplay`: fail.
- More than one `add_message::<T>` per message type: fail.
- `game_audio` must not depend on `game_assets` or `game_settings`.
- Literal 60 as a frame rate in more than one crate: fail.
- `tests.sh` exit 124: fail.

## Migration plan

Order matters: tooling first so every later step has proof.

1. Flags and modes. Rename `--norender` to `--headless` and `--transport` to `--stdio`. Delete `--mute`. `AppBuilder` becomes `enum Frontend` plus bounded run. Update `scripts/PLAYTEST.md`, `tests/cli.rs`, workflows.
2. Comment policy. Checker and doc in one commit.
3. Docs set and `AGENTS.md`. Write the target state, not the current state. Add the grep gates to `tests.sh` as failing steps are fixed, one gate per step below.
4. stdio crate. Split `crates/transport` into generic `stdio` and `impl Playable for Specimen` in core (renamed to `app` in step 8). Add `version`. Fix the binding panic. Move `scripts/playtest_agent.py` to `playtest/`.
5. Autopilot proof. Create `tests/e2e/` with scenarios defined in the test binary. One headless asserting scenario, one rendered under Xvfb. Wire the rendered CI job. Smoke never accepts a timeout.
6. Capture video. Rename loops to video, add `VideoProfile`, `FrameStep` owned by core. Audio event log and offline mixer, mux branch live.
7. Audio bus. Reduce `crates/audio` to the bus. Delete the `Sound` enum and match, the `game_assets` and `game_settings` dependencies, the mains-power copies. Feature assets structs carry sound handles. Remove the five `PlaySound` re-registrations.
8. Split core. `crates/app` (builder, flow, glue, Playable impl) and `crates/frontend` (menus, lighting). Rename `game_core` consumers.
9. Features, one at a time, each with its own commit and the gates turning green per feature: facility (with `PowerState`, `MainsPowered`, `enum Module`), player (controller pure plus view), monster, devices, fuses, hiding, interaction (takes `interaction_hint`), ambience (takes timers and RNG), presets, achievements. HUD sync systems move into feature views. `levels/render.rs` (720 lines) is dissolved into feature views.
10. Glue to config only. After step 9 glue is asset fill, startup settings copy, volume copy. Settings panel writes gameplay resources directly. Glue gate on.
11. Local types. `enum Input` wrapper, private conversion into `Binding` in gameplay `player/logic.rs`, gamepad variant or error.
12. Probe. Delete the crate, move `world_instances_ready`, add Tracy and perf packages to `flake.nix`, write `scripts/perf/frametime.py`, add the diagnostics plugins behind `--debug`, add the p99 e2e scenario.
13. Skills and agents. Dedupe `.pi/subagents.yaml`, fix the skill heading, add the proof-tools line.
14. Final pass. All gates on in `tests.sh --full`. `flake.nix` `checks.smoke` resolved. CHANGELOG entry.

## Acceptance

- `./tests.sh --full` passes with every grep gate enabled.
- `cargo run -- --headless --seed 1` exits cleanly after a bounded run.
- `cargo run -- --stdio` answers a versioned command line with a versioned reply.
- `tests/e2e` contains at least one headless and one rendered asserting autopilot scenario, and CI runs both.
- `--record` with `VideoProfile::PREVIEW` and `FINAL` produces WebM with audio from the event log.
- `crates/probe`, `--mute`, `--norender`, `--transport`, the `Sound` enum and `requires_power()` copies no longer exist.
- `crates/gameplay/Cargo.toml` has no `game_assets`. `crates/audio/Cargo.toml` has no `game_assets` and no `game_settings`.
- `docs/` holds the five files and `AGENTS.md` points at each.

## Out of scope

- No new gameplay. No changes to art, assets or credits beyond what the asset-fill pattern needs.
- No release, no push, no PR, no itch or pages deploy.
- No behavioral agent rules added preventively. Rules are added when a mistake happens.
- The halloween feature stays a compile-time decorator; it is not merged into gameplay.

## Status

- 2026-10-12: task written. Implementation not started; the user asked for the task first.
- 2026-10-12: steps 1-5 done on master, uncommitted. `--headless`, `--stdio`, `--seed`, `--record`; `enum Frontend`; comment checker; docs set; `crates/stdio` + `impl Playable for Specimen`; `playtest/harness.py` + `specimen.py`; `tests/e2e` headless and rendered escape runs; rendered CI job. Scenarios live in `tests/e2e/escape_run.rs` (user: nothing but the lib and `main.rs` under `src/`).
- 2026-10-12: step 6 done. `capture::video` with `VideoProfile::{PREVIEW, FINAL}`, `VideoCapturePlugin::new(profile, step)`, `video_start/video_end/video_written`; `FrameStep` in core feeds stdio and capture; audio event log `audio.jsonl` from the capture crate's `AudioPlayer` watcher (the `bus` field came in 2026-10-13; the mix uses effective volumes); Rust mixer; `--video-profile`; `rendered_record_run` e2e scenario proves both profiles mux opus audio.
- 2026-10-12: step 7 done. `game_audio` is bevy only: `PlaySound { handle, bus, placement }`, `Bus`, `Volumes`, `Falloff`, `AudioPaused`, `AmbientEmitter`/`AmbientEmitters`/`AmbienceActive`. Gameplay sends `noise::Noise { kind, origin }` (interim `NoiseKind` until step 9); core glue maps kinds to `SoundAssets` handles and fills `MenuSounds`. Gates `audio_manifest` and `message_owners` in `tests.sh`.
- 2026-10-12: step 8 done. `crates/core` split into `crates/app` (`game_app`: `AppBuilder`, `Frontend`, `FrameStep`, `Specimen`, glue) and `crates/frontend` (`game_frontend`: `CoreState`, `GameState`, `PauseState`, `MenuPlugin`, `MenuSounds`, `FacilityLightingPlugin`). Menu tests and the menu fixture live under `crates/frontend/tests`, stdio and builder tests under `crates/app/tests`.
- 2026-10-12: step 13 done. `.pi/subagents.yaml` uses anchors for the shared tool lists, delegate lists and system prompts (one copy each); scouts and reviews name the proof tools. `.agents/skills/game-task/SKILL.md` heading is `Tatr Task` and carries the paid-for rules.
- 2026-10-12: step 12 done. `crates/probe` deleted (`world_instances_ready` lives in `examples/facility_shots.rs`); dev shell has `tracy` 0.13.1, `cargo-flamegraph`, `perf`; root features `tracy` (tracy-client pinned to 0.18.4 to match the Tracy 0.13.1 protocol) and `trace-chrome`; `scripts/perf/frametime.py` with unit tests in `tests.sh`; `DebugPlugin` adds the frame time diagnostics; `headless_frame_budget` e2e scenario (p99 budget 50 ms, measured 0.54 ms).
- 2026-10-12: step 9 facility done. `gameplay::facility` with `Module` enum (54 variants, `footprint`, `light_spec`, `mains_powered`), `Prop(Module)`, `FacilityPower { state: PowerState, .. }` with `is_on()`/`outage_in()`, `MainsPowered` tag, `FacilityPlugin` + `FacilityViewPlugin`, feature `FacilityAssets` filled by `glue/facility_assets.rs` (`resource_exists` is the readiness signal). `levels` re-exports the moved items for now. Behavior note: `FacilityPower::restore()` from blackout now arms a repairable trip (debug checkbox only); stdio `module` field reports the Debug name (`StorageCrate`).
- 2026-10-12: step 9 monster done. `gameplay::monster` with `logic.rs`, `view.rs` (`MonsterViewPlugin`, animations built when `MonsterAssets` appears), `assets.rs` (`MonsterAssets { scene, idle, walk, chase, attack }`), glue `monster_assets.rs`. `MonsterRenderPlugin` renamed `MonsterViewPlugin`. Only `levels/render.rs` still imports `game_assets`.
- 2026-10-12: step 9 player done. `gameplay::player` with `logic.rs`, `flashlight.rs`, `stamina.rs`, `collision.rs`, `settings.rs`, `footsteps.rs` (moved out of glue, now headless logic) and `view.rs` (camera, listener, beam, stamina and flashlight HUD syncs with `Single`). `controller` is a re-export shim until the last readers move. Glue `stamina_hud.rs` and `flashlight_hud.rs` deleted.
- 2026-10-12: step 9 devices, fuses, hiding, interaction done. `gameplay::devices` (`DevicePlugin` + `DeviceViewPlugin` with the flashbang, detector and flash HUD), `gameplay::fuses` (`FusePlugin` + `FuseViewPlugin` with the fuse HUD, `Module::FusePickup`), `gameplay::hiding` (logic only), `gameplay::interaction` (`targets.rs`, `pickups.rs`, `view.rs` with the interaction hint, `InteractionAssets { font, interact_glyph }` filled by `glue/interaction_assets.rs`). `levels/render.rs`, `glue/{device_hud,fuse_hud,interaction_hint}.rs` deleted. `gameplay` no longer depends on `game_assets`; gates `gameplay_handles` and `gameplay_manifest` on in `tests.sh`. `glue_config_only` fails only on `fps_overlay.rs` and `sound.rs`.
- 2026-10-12: step 9 sounds done. `noise.rs`, `NoisePlugin`, `NoiseKind` and `NoiseOffsets` deleted. Each feature owns a cue message (`DoorCue`, `PowerCue`, `PlayerCue`, `MonsterCue`, `DeviceCue`, `PickupCue`, `FuseCue`, `HidingCue`, `AmbientCue`) that implements `gameplay::audible::Audible`, and a `sounds.rs` with `XSounds` handles plus a plugin that maps cues to `PlaySound`. `gameplay::ambience` (`AmbiencePlugin`, `AmbienceViewPlugin`, `AmbienceSounds`) replaces `levels/sounds.rs` and the glue emitter sync. `app/glue/sound.rs` only fills `MenuSounds`; `app/glue/sounds.rs` fills every `XSounds`. The stdio snapshot reads cues through `listen::<T: Audible>`. `glue_config_only` passes for every glue file except `fps_overlay.rs`.
- 2026-10-12: steps 10 (settings half) and 11 done. `gameplay::player::Input { Key, Mouse }` replaces `Binding` in `KeyBindings`; the player feature converts privately; stdio `apply` can no longer meet a gamepad binding. The settings panel writes `ControllerSettings` and `Volumes` directly through `game_frontend::{controller_settings, volumes}`; `glue/settings.rs` (`SettingsGluePlugin`) copies both once at build. The `resource_changed::<GameSettings>` syncs are gone. Gate `gameplay_input_types` passes. Round 4 verified together: smoke, clippy, rendered e2e, wasm check all green.
- 2026-10-12: step 9 objective, achievements, presets done; `levels` and `controller` shims deleted; `glue/fps_overlay.rs` deleted (`game_ui::FpsOverlayPlugin { visible }` set in `app`, the settings panel writes the node visibility). Achievements read `PickupCue`, `PowerCue`, `ExitOpened`, `MonsterCue`, `FlashbangHit` in `PostUpdate` before `facility::ObjectiveSystems`; `AchievementSignal` is gone. Behavior note: opening the exit and being caught in the same frame no longer unlocks `CaughtAfterExitOpen` (same-frame race a player cannot produce).
- 2026-10-12: step 14 done. `tests.sh` runs every gate in both modes; `flake.nix` `checks.smoke` removed (CI runs `tests.sh --full` through `nix develop`; `nix flake check` builds the package only); CHANGELOG Unreleased entry. Final proof: `./tests.sh --full` exit 0 (430 tests, 12 gates, headless and rendered e2e, `headless_frame_budget` p99 0.41 ms), wasm check ok, `--headless --seed 1` exits 0, `--stdio` answers a version 1 command with a version 1 reply. All work on master, uncommitted.
- 2026-10-12: user asked to hide the per-second diagnostics log. `DebugPlugin` no longer adds `FrameTimeDiagnosticsPlugin` (owned by `game_ui::FpsOverlayPlugin`; a second copy panicked) or `LogDiagnosticsPlugin`; it logs one `fps, frame time` line per second at debug level. Chatty info logs (autopilot steps, facility assets ready) are debug level; fuse seed, screenshot and video paths, Steam status stay info.
- 2026-10-12: fix D1 done. `facility::RunPlugin` owns `StartRun { preset }`; one system sets `GamePreset`, builds the first floor (power reset for the run), spawns the actors and tags top-level run entities `RunRoot`. The `ActiveRun` anchor restores mains when it despawns. Menu Play/Retry and app `GamePlugin` only write `StartRun` and set `Playing`; menu scopes `RunRoot`, `GamePlugin` scopes `ActiveRun` to `Playing`. `enter_world` and `reset_facility_power` deleted.
- 2026-10-12: fix D2 done. Tests add `FacilityPowerPlugin` (directly or through `CueOwners`) and set states with `resource_mut`. Gate `test_mirror_resources` added.
- 2026-10-12: fix D3 done. HUD widgets moved into features: `devices/hud.rs` (`DeviceHudPlugin`), `fuses/hud.rs` (`FuseHudPlugin`), `player/stamina_hud.rs` (`StaminaHudPlugin`), `player/flashlight_hud.rs` (`FlashlightHudPlugin`), each added by its view plugin. `ui` holds `theme`, `widgets`, `slider`, `fps`. HUD tests moved to `crates/gameplay/tests`.
- 2026-10-12: fix D4 done. `FacilityLightingPlugin` lives in `gameplay::facility::lighting`. Gate `power_state_scope` matches `PowerState|FacilityPower` with an allowlist (gameplay, stdio snapshot, debug, tests).
- 2026-10-12: fix D5 done. `requires_power()` deleted; ambient and intermittent sources that need mains carry `MainsPowered` (intermittent sounds are child entities of the prop). Gate `requires_power` added.
- 2026-10-12: fix D6 done. `crates/flow` (`game_flow`, bevy only) holds `CoreState`, `GameState`, `PauseState`; `app` and `frontend` import them; `gameplay` does not.
- 2026-10-12: fix B4 done. Settings `activate` takes a `Query` for `FpsText`; settings buttons work without `FpsOverlayPlugin`.
- 2026-10-12: fix B6 done. Video staging is removed on every terminal path (`warn!` on a remove error).
- 2026-10-12: fix B7 done. The e2e record run waits `profile.deadline + 180 s` per profile.
- 2026-10-12: fix B8 done. Menu and cinematic leak tests assert looping `AudioPlayer` voices return to their baseline; an ambience fixture proves voices exist while playing.
- 2026-10-12: review fixes verified: `./tests.sh --full` exit 0 (436 tests, 14 gates, headless and rendered e2e), clippy with `debug,halloween` clean, wasm check ok. Uncommitted on master.
- 2026-10-13: fix item 1 done. `AmbientCue { kind, prop, offset }`; `origin()` is `Origin::From(prop)` and the ambience sounds plugin plays `PlaySound::from(handle, prop, offset, falloff)`, so the voice is a child of the prop and ends with the run. The range check reads `GlobalTransform`; headless adds `TransformPlugin` so props carry their world pose. Tests: voice position is the prop world pose plus offset, voices despawn with their prop, perception hears the cue in range.
- 2026-10-13: fix item 2 done. Rendered `--stdio` wakes the loop after every reply, so a burst of commands runs without the 60 s reactive timeout. Burst of ticks 1, 60, 120, 200, 260, 300 under Xvfb: last reply at 17.7 s (was 245 s).
- 2026-10-13: fix item 3 done. `tests/e2e` switches: none runs the headless runs, `--rendered` the rendered escape run, `--record` the record run only. `tests.sh` and CI never run `--record`; stdio and record sessions are local only.
- 2026-10-13: fix item 4 done. `tests.sh --full` adds clippy and tests with `--features debug,halloween`. `cargo check --locked --features steam` passes by hand; it is not in `tests.sh`.
- 2026-10-13: fix item 5 done. Frontend reads `MenuAssets` (font, key glyphs by name, achievement icons by stem), filled by `app/glue/menu_assets.rs` on `OnEnter(GameAssetsState::Ready)`. `game_assets` is a frontend dev-dependency only. Gate `frontend_assets`.
- 2026-10-13: fix item 6 done. `facility/lighting.rs` is `lighting_view.rs`; `PointLight`/`SpotLight` code moved from `lights.rs` to `lights_view.rs` (`PropLightsPlugin`, added only by `FacilityViewPlugin`). Gate `gameplay_render_types` allows render types only in `view.rs`, `*_view.rs`, `hud.rs`, `*_hud.rs`.
- 2026-10-13: fix item 7 done. `update_pause` also pauses world-bus sinks added while `AudioPaused` is true. Tests for sinks added during and outside a pause.
- 2026-10-13: fix item 8 done. `playtest/harness.py --encode-timeout` defaults to the profile deadline plus 180 s; docs fixed.
- 2026-10-13: fix item 9 done. Stdio marks readiness in `PostUpdate` (`mark_ready`), so cues on the ready frame are heard at tick 0. Test added.
- 2026-10-13: fix item 10 done. `run.rs` asserts the world entity count after a run ends equals the count before the first `StartRun`, over two runs (cached systems and their id resources excluded).
- 2026-10-13: fix item 11 done. `IntermittentSound` (config) and `IntermittentTimer` (runtime); `StaminaState`, monster `LongPatrol`, `ThrowPhase`; `GamePreset::{Normal, Custom(PresetRules), Insane}`; `LightingSettings` uses `AmbientNits` and `LightScale`; `CustomDifficulty` uses `CustomLevel`, saved as the same numbers 0-2 (test loads a settings file in the old format and saves it back unchanged).
- 2026-10-13: fix item 12 done. Every audio log event carries `bus` (`world`, `ui`, `unrouted`); capture reads `game_audio::Bus`. The mixer ignores it (effective volumes). Mixer and plugin tests updated and extended.
- 2026-10-13: fix item 13 done. Capture `DEFAULT_STEP` and `CapturePlugin::default()` removed; the screenshot examples pass `FrameStep::SCREENSHOTS`. Gate `frame_rate_owner` also rejects frame step literals outside `crates/app/src/lib.rs`.
- 2026-10-13: fix item 14 done. "Release checklist" section in `docs/tooling.md`; the conventions `TODO:` line points at it.
- 2026-10-13: fix item 15 done. TASK message names match the code (`DoorCue`, `FlashbangHit`, `FuseCue`, `ExitOpened`, `StartRun`); the `Binding` conversion is in gameplay `player/logic.rs`.
- 2026-10-13: fix item 16 done. `docs/architecture.md` says `Module::mains_powered()` is kind data that only the facility tagger reads.
- 2026-10-13: fix item 17 done. `FUSE_*` colors moved from `game_ui::theme` to `gameplay::fuses` (`fuses/hud.rs`); device and player HUDs import them.
- 2026-10-13: fix item 18 done (documented, not moved). The menu background stays in `gameplay::facility`: the power tagger reads its marker, it uses the private level builders, and gameplay and halloween tests build it. Moving it would need `gameplay`/`halloween` -> `frontend`.
- 2026-10-13: fix item 19 done. `apply_window_mode` and `apply_graphics` removed from `app`. The settings panel writes `Window.mode` and camera `Msaa` at once; `app` sets the window mode at build and adds `game_frontend::CameraQualityPlugin` (new cameras get the saved quality). Gate `app_settings_sync`.
- 2026-10-13: fix item 20 done. CHANGELOG Unreleased frontend entry, `playtest/README.md`, `docs/assets.md`, `docs/testing.md`, `docs/coding-conventions.md` and the complete edge list in `docs/architecture.md` (with the reason for `crates/flow`) updated.
- 2026-10-13: fix item 21 done. Smoke uses `${CARGO_TARGET_DIR:-target}`; `--video-profile` requires `--record` (cli test); the Python test asserts `"version": 1` on each sent line.
- 2026-10-13: fix item 22 done. The `tests.sh` rendered step uses `VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-$BEVY_SOFTWARE_VK_ICD}"`; `flake.nix` exports the lavapipe ICD path. `ci.yaml` unchanged (its rendered job already sets the ICD; no record run).
- 2026-10-13: fix item 23 done. `Recording` removes its staging on drop (`warn!` on error). Stdio calls `capture::video::video_close_at_exit` after every step: after a plain exit (window closed) it encodes the frames already captured; after an error exit or a failed encode it logs one `video ... was not written` error and the exit status is an error. Tests: drop removes staging; an open recording at exit gives an error status.
