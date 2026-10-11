# Review the shit out of this thing

- STATUS: CLOSED
- PRIORITY: 0
- TAGS: review

## Verdict

The HEAD commit is not an outlier. It is the house style. The codebase has
roughly 80 `Option<Res<T>>` system params, and almost every one exists so a
test can add one plugin without the plugin that owns the resource. Each one
carries a silent fallback (`map_or`, `is_some_and`, `is_none_or`,
`unwrap_or_default`) that re-states a default owned by another crate. The
"MouseLeft" literal alone is hard-coded in 4 places outside `game_settings`.
Below is the curated list, worst first. Line numbers are as of 0a22cf8.

## A. The HEAD commit (0a22cf8), as the canonical example

1. `crates/core/src/glue/controller_settings.rs` mirrors
   `GameSettings.mouse_sensitivity` into a second resource every frame.
   `gameplay` already depends on `game_settings` and reads `GameSettings`
   directly in the same file (`controller/player.rs:169,177`). The mirror
   decouples nothing.
2. `controller/player.rs:29-40` `PlayerControllerSettings::default()`
   re-states `DEFAULT_SENSITIVITY`. Third copy of this value:
   `controller/wasd_camera.rs:8` still has `LOOK_SENSITIVITY = 0.002`.
3. `controller_settings.rs:19` `is_changed() && a != b` double-guards a plain
   copy.
4. `tests/unit/monster.rs:653-658` re-assembles `PlayerControllerPlugin` by
   hand (`insert_resource(PlayerControlsEnabled)`, two `add_message`,
   `init_resource::<PlayerControllerSettings>`, `add_systems(apply_input)`).
   This is plugin wiring copied into a test. It will diverge silently.
5. Fix: delete the glue plugin and `PlayerControllerSettings`. `apply_input`
   takes `Res<GameSettings>`. Tests insert `GameSettings` (two gameplay tests
   already do: `tests/input_bindings.rs:12`, `tests/player_controller.rs:118`).

## B. Same disease, systemic: `Option<Res<T>>` with a silent default

Group by the resource. Every entry: the resource always exists in the game,
the `None` arm is reachable only from a test that skipped the owning plugin.

6. `GameSettings` (always inserted first, `core/src/lib.rs:120`):
   `gameplay/src/controller/player.rs:169,177`, `flashlight.rs:77,90`,
   `core/src/glue/flashlight_hud.rs:42`, `device_hud.rs:57`,
   `interaction_hint.rs:89`, `audio/src/lib.rs:173,274,338`,
   `audio/src/ambience.rs:71`, `ui/src/fps.rs:45`,
   `transport/src/transport.rs:70-72` (`get_resource().unwrap_or_default()`).
   Fallbacks re-state the defaults: `"MouseLeft"` (flashlight.rs:65,
   flashlight_hud.rs:47), `"MouseRight"` (device_hud.rs:62), `"KeyF"`
   (interaction_hint.rs:60,145), `MovementKeys::default()` (player.rs:172),
   volume `1.0` (audio/lib.rs:141).
7. `PlayerControlsEnabled`: 9 systems take `Res`, 10 take `Option<Res>` with
   `is_some_and(|e| !e.0)`: `levels/devices.rs:195,308,325,383`,
   `levels/monster.rs:494,588,746,769`, `levels/render.rs:316`. Two
   conventions for the same resource in the same crate.
8. `FacilityPower`: `is_some_and(|p| !p.on)` and `is_none_or(|p| p.on)` are
   both used, so "no resource" means "dark" in one system and "lit" in the
   next: `gameplay/src/achievements.rs:199`, `levels/power.rs:119,157,175`,
   `levels/interaction.rs:33`, `levels/render.rs:574,600`,
   `core/src/lighting.rs:49`, `core/src/glue/sound.rs:86,112`,
   `transport/src/perception.rs:172` (`power_on.unwrap_or(true)`).
   Also `first_floor_builder/mod.rs:47-51`: the level geometry builder inserts
   this runtime resource instead of `FacilityPowerPlugin`.
9. `FacilityAssets`: seven identical early-return guards in
   `levels/render.rs:230,356,441,508,623,733,785`, plus `pickups.rs:67,140-150`
   and `devices.rs:165,184` where gameplay behaviour changes (pickup animates
   vs despawns instantly) based on whether render assets exist.
   One `run_if(resource_exists::<FacilityAssets>)` on the set replaces all of
   them.
10. `FuseSeed` / `GamePreset` / `GlobalRng`: `first_floor_builder/mod.rs:27-32,
    54-65` triple fallback, and `levels/fuses.rs:154-157` falls back to
    `ChaCha8Rng::default().next_u64()`. That is a fixed key. A run without
    `FuseSeed` is "random" but identical every launch.
11. `PatrolRng` (`levels/monster.rs:142,772`): a second RNG resource beside
    `GlobalRng` so tests can seed the monster alone.
12. `UiAssets` and `State<GameState>`: `core/src/menu/settings.rs:763,768,977`,
    `menu/achievements.rs:260,356`. The buttons that trigger these only exist
    after `CoreState::Ready`, which requires the assets. Dead guard.
    `menu/tests.rs:133-159` "covers" it with a trivially true assertion.
13. `FacilityAssets` in `menu/main_menu.rs:317-344` and
    `menu/achievements.rs:153-160` selects between two camera setups
    (`background::spawn` vs `screen_camera`). The second path exists only in
    tests. Two camera setups for one screen.
14. `AmbientEmitters`: defined in `game_audio`, read there as `Option`
    (`audio/src/ambience.rs:72`), inserted by `core/src/glue/sound.rs:19`.
    `tests/unit/ambience.rs:46-59` exists to cover the `None` arm that only
    the test can reach.
15. `State<PauseState>` / `State<GameState>` in world-sim glue:
    `core/src/lighting.rs:50`, `glue/sound.rs:148`, `lib.rs:268`,
    `menu/pause.rs:76-80`. "No menu plugin" means "always playing". Lighting
    and audio should not know the menu exists.
16. `CaughtShot` (`menu/game_over.rs:62-63`): `map_or(Transform::IDENTITY)`
    fallback that is unreachable, the only writer of `GameOver` inserts it.
17. `AudioPaused` in `transport/src/perception.rs:544,567`; `WinitSettings`
    in `probe/src/lib.rs:301` (the plugin inserts it itself at :187, so it is
    never None); `RenderAdapterInfo` `probe/src/lib.rs:302,327-332` writes
    `"unknown"` into a perf report so a test without `RenderPlugin` passes.
18. `halloween/src/decor.rs:678-682` takes `Option<ResMut<Assets<Mesh>>>`
    and `Option<ResMut<Assets<StandardMaterial>>>` and silently skips the
    cable. `eyes.rs:44-45` in the same crate does it right.
    `decor.rs:591,642` `Option<Res<HalloweenAssets>>` hides a real ordering
    hole: `replace_plant_visual` has no catch-up path.
19. `capture/src/screenshot.rs:29-33`, `loops.rs:103-115`,
    `autopilot/src/predicate.rs:14-26`: predicates return `false` forever
    when the plugin is missing, so a misconfigured autopilot times out instead
    of failing fast. `loop_start` in the same file panics. Pick one.

Rule to add to coding conventions: a system that reads a resource takes
`Res<T>`. The plugin that owns `T` inits it. Tests add that plugin.

## C. Production code shaped by tests

20. `PlayerControllerPlugin { camera: bool }` + `without_camera()`
    (`controller/player.rs:81-96,132-137`): a production plugin carries a
    test switch, and `tests/controller.rs:42-53` tests the switch. Move
    camera/audio/beam/cursor attach to a presentation plugin the app adds.
21. `achievements/src/lib.rs:39-43` `#[doc(hidden)] pub fn
    install_native_for_test`. Tests run `build_native(app, Some(path))`,
    production runs `build_native(app, None)`. The tests never execute the
    plugin the game uses.
22. `assets/src/lib.rs:178` `UiAssets` derives `Default` (the other three
    collections do not). Only callers are tests. Yields invalid handles and
    breeds `Option<Res<UiAssets>>` + `unwrap_or_default` downstream.
23. `core/src/menu/mod.rs:69-73` `MenuPlugin` inits `GameSettings`,
    `SettingsDirty`, `AchievementProgress`, `GamePreset` and `PlaySound`,
    all owned by other plugins that `AppBuilder` always adds first. Exists so
    `menu/tests.rs:55-66` can add `MenuPlugin` alone. Masks a missing-plugin
    bug in production. Same: `menu/complete.rs:27` registers `ToggleDoor`.
24. Defensive `is_plugin_added` / `add_message` / `init_resource` in every
    plugin so tests can add any subset: `GameUiPlugin` guarded 6 times
    (`glue/{stamina,fuse,flashlight,device}_hud.rs`, `menu/mod.rs:66`,
    `debug/src/lib.rs:44`), `PlaySound` registered in 8 production plugins,
    `PickupPlugin` guarded in `fuses.rs:59`, `devices.rs:140`,
    `CaptureState` inited in 3 capture plugins, `GamePreset` inited in
    `achievements.rs:154` and `monster.rs:164`.
25. `debug/src/lib.rs:44-78` `DebugPlugin::build` probes four plugins and
    builds four different plugins. The only consumer always has
    `DefaultPlugins`. `tests/plugin.rs::headless_app_skips_egui_and_wireframe`
    asserts the test-only shape.
26. Public API that exists for tests: `levels/mod.rs:21-61` re-exports
    (`pulse_interval`, `detector_reading`, `panel_center`, `select_fuse_slots`,
    `FUSE_TABLES`, ...), `controller/mod.rs:7` `pub use collision::{door_frames,
    door_panel, wall_obstacles}`, `Monster::debug_navigation()`
    (`monster.rs:72-90`, shaped for the debug crate), `RenderCeilings(bool)`
    (`render.rs:62-69`), `transport/src/rendered.rs:33` `pub first_tick`,
    `pub mod steam` in achievements, `Sound::requires_power`, `widgets::label`
    and `widgets::button` in `game_ui` (only caller is a test).
27. `#[cfg(test)] #[path = "../tests/unit/*.rs"] mod tests;` in 36 places.
    Files under `tests/` are compiled as in-crate unit modules with private
    access. `cargo test --test <name>` cannot run them, `use super::*` has no
    visible parent, and `menu/tests.rs` plus `halloween/src/lighting.rs:16`
    use the other convention, so the repo has two conventions. Worst case:
    `gameplay/tests/unit/monster.rs` (2497 lines) imports 13 private consts
    and fns, mutates `monster.route` and `pursuit` directly, and calls
    `crate::controller::player::apply_input`. Every private rename breaks it.
    Also `examples/human_deer_viewer.rs:253-255` includes a test file into an
    example binary.
28. Tests that grep the source: `assets/tests/collections.rs:11,37,63` and
    `halloween/tests/assets.rs:36` `include_str!("../src/lib.rs")` and scan
    for string literals. A rustfmt change breaks them. Model the data in a
    `const` table and assert against it.
29. `transport/tests/perception.rs:580-612` copies the finish/cleanup/ready
    loop out of `transport::run` instead of calling it.
    `transport/tests/unit/rendered.rs:40-54,106-116` re-implements the Main
    schedule surgery from `RenderedTransportPlugin::finish` by hand, twice.

## D. Duplicated defaults and constants

30. `settings/src/lib.rs`: defaults declared three times. `#[serde(default)]`
    on the struct (:192), per-field `#[serde(default = "default_volume")]`
    (:194-200), and a hand-written `impl Default` (:210-225) restating `1.0`,
    `DEFAULT_SENSITIVITY`, and enum variants that already have `#[default]`.
    The struct-level attr alone does the job. `default_volume` and
    `default_sensitivity` fns (:147-152) are dead.
31. `SettingsPath(pub Option<PathBuf>, pub bool)` (`settings/src/lib.rs:308`):
    on native `.1` is implied by `.0.is_some()`, on wasm `.0` is always None.
    `save_dirty` has an arm dead on each platform. Enum
    `{ Disabled, File(PathBuf), Browser }`.
32. `const TILE: f32 = 2.5` declared in `collision.rs:15`,
    `interaction.rs:14`, `render.rs:32`, and re-implemented in
    `tests/level_doors.rs:130-134`, `tests/level_signs.rs:112-116`.
33. `probe/src/lib.rs:22-33,54,95-116`: present modes as three tables;
    `DEFAULT_RESOLUTION: &str = "1280x720"` beside
    `UVec2::new(1280, 720)`. `tests/args.rs::probe_uses_defaults_without_a_flag`
    exists to catch them drifting.
34. `debug/src/monster_tracking.rs:8-17`: two palettes for the same four
    colours (`egui::Color32` and bevy `Color`), hand-converted, already
    drifted by one unit.
35. `flake.nix:21` pins `nightly."2026-07-03"` and `rust-toolchain.toml` pins
    it again by hand. Use `fromRustupToolchainFile`.
36. `capture/src/loops.rs:16` `AUDIO_SAMPLE_RATE = 44_100` is never used;
    `:232` passes the literal `"44100"`.
37. `audio/src/ambience.rs:26` `AmbientEmitter.spatial` duplicates
    `AmbientSound::spatial()`. The test builder sets it from
    `position.is_some()` and asserts combinations the game cannot produce.

## E. String-typed enums and hand-rolled tables

38. `MovementKeys` is seven `String`s (`settings/src/lib.rs:49-59`), parsed by
    a 31-arm `parse_key` (:110-145) and positionally validated with
    `keys[..5]` / `keys[5..]` in `unique()` (:76-92). Every consumer unwraps
    (`player.rs:142-161`, `.expect("sanitized player binding")`).
    `KEY_GLYPHS` (`assets/src/lib.rs:87-170`) advertises 83 keys,
    `parse_key` accepts 31, so the rebind UI shows glyphs settings reject.
    Store `KeyCode` / `InputBinding` with serde directly; unit variants
    serialize as `"KeyW"` so files stay compatible.
39. Prop modules are strings matched in five tables: `collision.rs:175-199`
    `prop_footprint`, `lights.rs:77-112` `light_spec`, `sounds.rs:64-120`,
    `hiding.rs:75-79`, `render.rs:104-106,33-39`. Raw literals bypass
    `module_names.rs` (`props.rs:148,154,196`, `lights.rs:39`,
    `rooms.rs:13-17`, `render.rs:741-745`). A typo compiles. Make `Module` an
    enum with methods. Same in halloween: nine `pub const &str`, an array, a
    nine-arm match, nine asset attrs, and `decor.rs:537-559,702` matches the
    strings again.
40. Three binding-to-label formatters with three rule sets:
    `core/src/glue/mod.rs:17-24`, `menu/settings.rs:275-279`,
    `glue/interaction_hint.rs:152`.
41. Hand-written enum cyclers, four of them: `settings/src/lib.rs:30-47`
    `DisplayMode::next` / `GraphicsQuality::next`, `menu/main_menu.rs:95-98`
    `cycle_value`, `:210-211` inline `rem_euclid`, `examples/gallery/mod.rs:101`.
42. `menu/main_menu.rs:125-156` vs `:217-252`: the preset enum tables are
    written twice (decode a `u8`, then cycle). Root cause: `CustomDifficulty`
    stores `u8` indices (`settings/src/lib.rs:165-175`) because
    `game_settings` cannot depend on `gameplay`. Move the preset enums.
43. `transport/src/perception.rs:590-630` 30-arm `Sound -> &str`; transport
    owns the wire names of another crate's enum. `audio/src/lib.rs:345-373`
    26-arm `world_sound` classifier that every new `Sound` variant must be
    added to or it silently becomes a menu sound that ignores pause.
44. `debug/src/lib.rs:196-224` `ENTITY_GROUPS: [&str; 8]` plus
    `entity_group() -> usize`: an enum spelled as an array index.
45. `menu/settings.rs:68-115` `SettingsAction` mixes navigation, toggles and
    seven binding variants, so `name()` returns `""` for three variants,
    `key()` has `_ => return None`, `choice_value` has `unreachable!()`,
    `activate` needs a seven-way pattern. Split into `Bind(Binding)` and
    `Toggle(Choice)`.
46. `examples/prop_gallery.rs:310-386`, `halloween_gallery.rs:153-204`:
    dispatch on exhibit title strings with `_ => unreachable!()`. Renaming a
    title changes camera height.

## F. Duplication in src

47. `core/src/lib.rs:123-151` vs `:160-213`: headless and windowed branches
    each list the same nine `gameplay::levels::*Plugin` tuple plus
    `EnhancedInputPlugin`, `PlayerControllerPlugin`, `AchievementPlugin`. Seed
    to bytes written twice (:130-137, :178-184).
48. Room wall grid with openings computed three times:
    `collision.rs:241-271`, `interaction.rs:171-202`, `render.rs:631-727`.
    Opening resolution (link -> DoorRef -> Door|Passage) written four times
    (`collision.rs:243-253`, `interaction.rs:162-169`, `render.rs:632-648`,
    `monster.rs:885-906`) and again in four test files. Store the opening on
    the Room via a relationship.
49. Five `On<Start<Interact>>` observers each check `enabled.0`, iterate
    players, read `targets.aimed`, match one variant: `doors.rs:50-79`,
    `fuses.rs:77-97`, `hiding.rs:193-213`, `pickups.rs:62-152`,
    `power.rs:149-171`. Any new gate must be added in five places.
50. Six HUD `sync_*` systems with identical shape in
    `core/src/glue/{stamina,fuse,flashlight,device}_hud.rs`;
    `flashlight_hud.rs:41-54` and `device_hud.rs:56-69` are byte-identical
    except the field. `ui/src/flashlight.rs:96-108` == `ui/src/stamina.rs:78-90`;
    three HUD panels repeat the same 184px layout.
51. `menu/settings.rs:690-705` == `menu/achievements.rs:323-338`
    (`scroll_*`, down to the `38.0` line constant). The "hide MainMenu and
    PauseMenu" loop written four times (`settings.rs:784,801,888`,
    `achievements.rs:273,302`). `progress.unlocked.contains(&Escaped) ||
    progress.insane_unlocked()` four times in `main_menu.rs`.
52. `transport/src/transport.rs:258-271` vs `rendered.rs:183-216`: whole
    command validation ladder, three error literals, written twice.
    `Controls::apply` (`transport.rs:55-61`) hard-codes `KeyW/A/S/D/ShiftLeft/
    KeyF` while gameplay reads `MovementKeys` for the same actions. A rebound
    forward key breaks the transport `w` control.
53. `render.rs`: placeholder fuse mesh+material (:251-258 vs :461-473),
    flashbang cylinder (:185-190 vs :381-388), outline material (:369-374 vs
    :479-483) each created twice. `render_exterior_corridor` (:731-781)
    hand-rolls what `render_rooms` already does.
54. `monster.rs:256-291`, `:303-325`, `:704-715`: three copies of the same
    descendant stack walk. Bevy has `iter_descendants`.
55. `collision.rs:213-232` vs `monster.rs:794-813`: blocker assembly, and
    monster rebuilds the list per frame with `.clone()` at :808.
56. `first_floor_builder/rooms.rs:19-583`: forty `commands.spawn(door(..))`
    and twenty-six room blocks differing only in literals. `props.rs:13-646`
    ~100 calls of `prop(NAME, NAME, transform)` where both args are equal.
    This is data. Make it a table.
57. `sounds.write(PlaySound { sound, position: None })` 18 times in
    `core/src/menu`.
58. `core/src/glue/sound.rs:152-161` vs `menu/pause.rs:83-92`: the same
    `#[cfg(feature = "debug")] { debug.is_some_and(..) } #[cfg(not)] { false }`
    block. `lib.rs:265-290` `sync_debug_controls` and `pause.rs:108-122`
    both own `PlayerControlsEnabled` and `Time<Virtual>` pause; one only
    compiled under a feature.
59. `audio/src/lib.rs:183-188,199-212` and `:389-412`: loop bodies and spawn
    bundles copied per sink type / per sound kind.
60. `scripts/`: 16-bit WAV writer copy-pasted in 7 generators, `RATE =
    48_000` in nine files, `decode`/`loop_excerpt`/`save` byte-identical in
    two files, three independent GLB header parsers, `promote-*.sh` and
    `generate-*.sh` pairs identical except two variables.
61. `.github/workflows/release.yaml` repeats the halloween feature ladder
    three times and the platform download block is in both `release.yaml`
    and `deploy-itch.yaml`.

## G. Coupling in the wrong direction

62. `game_ui` depends on `game_settings` for one bool (`ui/src/fps.rs:3,44-56`,
    `ui/Cargo.toml:9`), with an `Option` fallback and a per-frame
    `*visibility =` write that dirties `Visibility` even when unchanged.
63. `gameplay` imports `game_settings::{parse_key, parse_binding}` and panics
    on bad config (`player.rs:12-14,142-161`). Settings should expose typed
    bindings.
64. `levels/devices.rs:18` imports `render::{BURST_LIFETIME, THROW_DISTANCE,
    THROW_GRAVITY}`: the flashbang arc is defined by the render module.
65. `render.rs` (816 lines), `MonsterRenderPlugin`, `LightingSettings` live
    in `gameplay`. Every render system needs the `Option<Res<FacilityAssets>>`
    guard because of it. Presentation belongs in its own crate.
66. `achievements.rs` imports `levels::{Caught, Escaped, FacilityPower}` and
    `levels/{pickups,doors,power,devices,monster}.rs` trigger
    `AchievementSignal`. Bidirectional.
67. `levels/sounds.rs:11` aliases `game_audio::{AmbientSound as
    AmbientSourceKind, Sound as IntermittentSoundKind}`.
68. `debug/Cargo.toml:11` declares `game_settings` as a dependency; no file in
    `debug/src` uses it. `debug/src/lib.rs:13` re-exports `game_ui::fps_label`
    and tests `game_ui` behaviour from the debug crate.
69. `transport/Cargo.toml:12` `gameplay = { path = "../gameplay" }` bypasses
    the workspace dep. `serde`/`serde_json` declared per crate in six
    manifests with no workspace entry. `probe/src/output.rs:78-109,190-207`
    hand-formats JSON with a hand-rolled escaper while `serde_json` is a dep
    of five sibling crates.
70. `core/src/lib.rs:120,122`: `menu` (computed as "windowed, no transport,
    no custom main plugin") is used as a proxy for "persist to disk".
    `main.rs:31-34` validates `--record` and `AppBuilder::build:111` asserts it
    again.

## H. Dead and redundant code

71. `menu/cinematic.rs:27-28,169` `CinematicOptions` spawned, never queried.
72. `core/src/lighting.rs:13-17` inserts `GlobalAmbientLight` with a value
    `apply_ambient` overwrites on frame one. `:33` `base.0 *= modifier` on
    `Added<LightIntensity>` mutates the authored base so the modifier cannot
    change at runtime, and `tests/unit/lighting.rs:4` pins that.
73. `menu/settings.rs:898-900` `if awaiting.is_changed() { return }` uses
    change detection as control flow. Works only because of `.chain()` order.
74. `levels/power.rs:130-141,176`, `interaction.rs:74-75`: the boiler is
    found three ways (scan `SourceSounds`, `With<IntermittentSound>`, string
    compare per frame). No marker component.
75. `levels/builder.rs:25-26` `Door { frame: String, panel: String }`,
    `Floor/Walls/Ceiling(String)`: always the same constant, tests pass
    `String::new()` because the fields do nothing.
76. `render.rs:518,652-659,795-798` `panic!` in Update systems on level data
    errors. Validate at build time.
77. `lights.rs:218-226` fuse panel spawned as two entities with the same
    transform, and `tests/exit_objective.rs:158-166` asserts the split.
78. `monster.rs:768-1023` `patrol`: 255 lines, closures nested seven deep,
    seven-tuple destructuring with `_`. `devices.rs:194-305` `tick_burst`:
    110 lines, five concerns.
79. `monster.rs:1022` `% 6` for a six-entry table. `monster.rs:242` reuses
    `MonsterScene` as two markers. `power.rs:32` private field `repairable`
    beside `pub fn repairable()`.
80. `collision.rs:22-27` `Obstacle` struct exists but the API passes
    `Vec<(Vec2, Vec2, f32)>` tuples. `to_local`/`direction_local` duplicate
    `rotate` with an inverted sign.
81. `core/src/glue/sound.rs:84-107` rebuilds, sorts and reallocates
    `AmbientEmitters` every frame.
82. `menu/main_menu.rs:585-619` `enter_world` duplicates `GamePlugin` via
    `register_system_cached` and tags run roots with a ten-way `Or<(With<..>)>`
    filter. Every new root component must be added or it leaks across runs.
83. `MenuAction::Quit` is `#[cfg(not(wasm32))]`, so nine `#[cfg]` attrs are
    scattered through spawn sites and matches.
84. `glue/interaction_hint.rs:9-15,104-121`: seven hard-coded pixel widths,
    one per label string, plus `+32.0` for custom keys.
85. `achievements/src/store.rs:8,18,42` `StoreData.version` written, never
    read. `native.rs:21-37` `Loaded::disabled()` == `Loaded::empty(None)`,
    `wasm.rs` builds `Loaded` by hand four times.
86. `debug/src/lib.rs:82,94` runs under `resource_changed` then compares
    again. `inspector_ui:170-175` hand-writes `set_if_neq` twice.
87. `capture/src/loops.rs:26,56,71` asserts `fps > 0` three times.
    `autopilot/src/lib.rs:51-53,264-266` `new()` == `default()`,
    `advance_step` wraps `index += 1`. `probe/src/output.rs:112-114`
    `Output::parse` wraps `from_path`.
88. `transport/src/rendered.rs:40-48` `tick()` and `updating()` differ by
    one.
89. `levels/objective.rs:43`, `fuses.rs:150` `info!` in gameplay systems.

## I. Tests

90. No `tests/common` anywhere. The player `app()` builder is copied 12 times
    in `gameplay/tests`, 13 times in `transport/tests`, 4 times in `core`.
    Input helpers (`press_f`, `click`, `aim`, `walk`) re-defined per file.
    `first_floor(seed)` inlined 14 times in `level_props.rs` alone.
    `Heard`/`Cues`/`SoundLog` + collect system copied six times. Temp-dir
    boilerplate copied six times and leaks on panic (no `tempfile`).
91. Tests that assert a constant equals its literal: `tests/devices.rs:425-433`
    (`DETECTOR_RANGE == 25.0`), `settings/tests/settings.rs:86-90`
    (`MIN_SENSITIVITY == 0.0`), `tests/unit/presets.rs:13-26`,
    `tests/unit/achievements.rs:150-159`, `tests/prop_sounds.rs:44-101`.
92. Change detectors on authored level data, not behaviour: `level_props.rs`
    (whole file), `level_doors.rs:18,58,68,75`, `level_signs.rs:216,271,323`,
    `hiding.rs:458-505`, `menu_background.rs`. Every prop move breaks several.
    `halloween/tests/unit/decor.rs:163,216-218,322` expects `DECOR.len() - 12`
    and `DECOR.len() - 4` with no derivation.
93. Tests that re-implement production: three room-graph BFS
    (`level_doors.rs:171-208`, `level_signs.rs:26-39`,
    `level_topology.rs:70-110`), collider math (`level_topology.rs:216-249`,
    `hiding.rs:146-158`), `DoorState` toggle copied three times in
    `tests/unit/monster.rs`, `HidingSpot -> module` table in `hiding.rs:56-60`.
94. Tests that test Bevy or rodio, not this repo:
    `transport/tests/rendered_schedule.rs`, `tests/unit/rendered.rs:200-215`,
    `audio/tests/unit/playback.rs:83-114` (constructs `rodio::Spatial` by
    hand; pins a direct `rodio` dev-dep that must track bevy's by hand).
95. `achievements/tests/plugin.rs:69-97` mutates `XDG_DATA_HOME` process-wide
    for a plugin that returns before reading it.
96. `audio/tests/unit/support.rs:5-67` hand-mirrors all 55 `SoundAssets`
    fields. `ui/tests/device_hud.rs:153,172,200` selects nodes by
    `node.width == px(8)` and `Name` strings because markers are private.
97. `menu/tests.rs:1068-1097` builds `AppBuilder::headless()` inside the menu
    unit module. `core/src/lib.rs:102-107` one-line boolean helpers whose
    only tests restate the boolean.

## J. CI

98. `.github/workflows/ci.yaml` runs only `cargo test --workspace`. `tests.sh`
    runs fmt, `check-comments.py`, `check --all-targets`, a smoke run, and
    clippy. CI and the local gate have already diverged. The comment policy
    is enforced only locally.

## Fix order

1. Items 1-19: one rule, `Res<T>` everywhere, owning plugin inits, tests add
   the plugin. Delete every re-stated default. This is the bulk of the rot and
   it is mechanical.
2. Items 20-26: delete the test-only API surface and plugin switches. Split
   presentation out of `gameplay` (item 65) so the `FacilityAssets` guards
   and `without_camera` disappear together.
3. Item 27: pick one test convention. Inline `mod tests` for white-box, real
   `tests/` for black-box. No `#[path]`.
4. Items 30, 38, 39, 42: typed settings, typed bindings, `Module` enum.
5. Items 47-52, 90: dedupe plugin lists, wall grid, interact dispatch, HUD
   sync, and a `tests/common` per crate.
6. Item 98: CI runs `tests.sh`.
