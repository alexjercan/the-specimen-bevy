# Architecture

The Specimen is one Bevy app with two frontends. The same gameplay schedule
runs headless (logic only, no window, no assets, no audio) or windowed. A
stdio protocol can drive either one from a host process.

## Layers

```
app          AppBuilder, Frontend, FrameStep, impl Playable, glue/
  |
  +-- frontend   menus, settings panel, cinematics, credits
  |
  +-- gameplay   features: logic + view + assets per feature
  |
  +-- flow       flow states: CoreState, GameState, PauseState
  +-- audio      bus: PlaySound { handle, bus, placement }, ambience, pause, per-bus volume
  +-- ui         generic widgets
  +-- assets     bevy_asset_loader collections (halloween has its own)
  +-- settings   persisted settings resource
  +-- achievements
  +-- stdio      generic protocol behind trait Playable
  +-- capture    screenshots and video
  +-- autopilot  scripted input for examples and e2e tests
  +-- halloween, debug   compile-time decorators
```

Allowed edges. This is the complete list of first-party `[dependencies]`;
crates not named on the left depend on bevy and third-party crates only
(`flow`, `audio`, `ui`, `assets`, `settings`, `autopilot`).

- `gameplay` -> `audio`, `ui`, `settings` (read-only resource types).
- `frontend` -> `flow`, `gameplay`, `ui`, `audio`, `settings`,
  `achievements`, and `debug` behind its `debug` feature. Not `assets`:
  menus read `MenuAssets` (font, key glyphs, achievement icons, no paths),
  which `app/glue/menu_assets.rs` fills on `OnEnter(GameAssetsState::Ready)`.
  `game_assets` is only a dev-dependency for the menu test fixtures.
  Check: `gate frontend_assets`.
- `achievements` -> `gameplay` (and `steamworks` behind `steam`).
- `debug` -> `gameplay`.
- `halloween` -> `assets`, `gameplay`. It owns its own asset collection.
- `stdio` -> `capture`.
- `capture` -> `audio`, to log the `Bus` of each voice.
- `app` -> everything above. Nothing depends on `app`.
- The root crate -> `app`, `flow`, `frontend`, `settings`, `assets`,
  `gameplay`, `autopilot`, `capture`, and `halloween` behind its feature.
  It re-exports them for the binary, examples and e2e tests.
- `flow` depends on bevy only. `app` inits the states; `app` and `frontend`
  read them. `gameplay` never reads them. The states have their own crate
  because `frontend` needs them and nothing may depend on `app`.
- Nothing under `crates/gameplay` depends on `game_assets`.
  Check: `gate gameplay_manifest` in `scripts/check-architecture.sh`.
- `game_audio` depends on bevy only, never on `game_assets` or `game_settings`.
  Check: `gate audio_manifest`.

## Features

`crates/gameplay/src/` holds one directory per feature: `player`, `monster`,
`facility`, `devices`, `hiding`, `fuses`, `interaction`, `ambience`,
`presets`, `achievements`. Feature axis, not functionality axis.

Each feature has:

- `logic.rs`: a plugin that runs headless. No `Handle`, `Mesh3d`,
  `PointLight`, `Camera3d` or `Window`.
- `view.rs` (and `*_view.rs` for a second view file): a plugin that spawns
  meshes, lights and HUD and sends `PlaySound`. Added only in windowed mode.
  In `facility`, `lights_view.rs` turns `Prop` light specs into
  `PointLight`/`SpotLight` (`PropLightsPlugin`) and `lighting_view.rs` scales
  them with `LightingSettings` (`FacilityLightingPlugin`); `lights.rs` keeps
  only the data (`LightingSettings`, `LightConfig`, markers).
- `hud.rs` (`*_hud.rs` in `player`): the feature's HUD widgets and a
  `XHudPlugin` that paints them, added by the view plugin. `ui` holds only
  generic widgets (`theme`, `widgets`, `slider`, `fps`).
- `assets.rs`: a feature-owned resource of handles with no paths. `app/glue`
  fills it from the loader collection at startup.
- `sounds.rs` where the view needs sound handles.

The exit objective (`Escaped`, reach of an unlocked exit door) belongs to
`facility`. So does facility lighting (`FacilityLightingPlugin`, added by
`app` when lighting is on). `presets` owns `GamePreset` through
`PresetsPlugin`. The fps overlay takes its initial visibility as a
`game_ui::FpsOverlayPlugin` field set in `app`; the settings panel writes the
node visibility directly.

Check: `gate gameplay_handles` allows `Handle<` only in `view.rs`,
`sounds.rs` and `assets.rs`. `gate gameplay_asset_paths` rejects
`bevy_asset_loader` and string asset paths under gameplay.
`gate gameplay_render_types` allows `Mesh3d`, `MeshMaterial3d`,
`PointLight`, `SpotLight`, `DirectionalLight`, `Camera3d` and `Window` under
`crates/gameplay/src` only in `view.rs`, `*_view.rs`, `hud.rs` and
`*_hud.rs`.

The HUD colors of a feature live with its HUD. The fuse colors
(`FUSE_EMPTY`, `FUSE_EMPTY_EDGE`, `FUSE_CERAMIC`, `FUSE_METAL`,
`FUSE_HAZARD`) are in `fuses/hud.rs`; the device and player HUDs import them
from `gameplay::fuses`.

The main menu background scene (`facility::build_main_menu_background`)
stays in `gameplay` although only the windowed menu spawns it. The facility
power tagger reads its `MenuBackground` marker so menu props never get
`MainsPowered`, the scene is built from the private level builders (`room`,
`passage`, `prop`), and the gameplay and halloween tests build it. Moving it
to `frontend` would need `gameplay` and `halloween` to depend on `frontend`,
which the edge list forbids. `frontend` adds the cameras and the accent light
around it.

There is no game-wide sound enum. Each feature's logic sends its own cue
message (`DoorCue`, `PowerCue`, `PlayerCue`, `MonsterCue`, `DeviceCue`,
`PickupCue`, `FuseCue`, `HidingCue`, `AmbientCue`), registered by the plugin
that owns it. Every cue implements `gameplay::audible::Audible` (`name`,
`origin`, `falloff`) with `Origin::{Listener, At, From}`. The feature's
`sounds.rs` holds `XSounds { Handle<AudioSource> .. }` and a plugin that maps
each cue to `PlaySound`, run only while `XSounds` exists; `app/glue/sounds.rs`
fills every `XSounds` from the loader collection. The stdio perception view
reads each cue type through one generic `listen::<T: Audible>` and lists the
heard cues by name.

## Audio bus

`game_audio` knows nothing about the game. `PlaySound { handle, bus,
placement }` plays one cue on `Bus::World` or `Bus::Ui`. `Placement::Global
{ gain }` is flat, `At { position, falloff }` and `From { source, offset,
falloff }` are spatial, attached as a child of the source for `From`.
`Falloff { range, scale, curve }` gives the per-cue gain by distance.
`Volumes { master, world, ui }` scales each bus and `AudioPaused` pauses the
world bus only. Ambience is data: `AmbientEmitters` lists `AmbientEmitter {
source, handle, volume, spatial }` and the bus keeps one looping voice per
source while `AmbienceActive` is true. Menus hold their cue handles in
`MenuSounds`, filled by glue.

Gameplay public items never expose library types. Input bindings are a
gameplay `enum Input { Key, Mouse }`; the player feature converts them into
`bevy_enhanced_input::Binding` in a private fn. `frontend` builds
`ControllerSettings` and `Volumes` from `GameSettings`
(`game_frontend::{controller_settings, volumes}`); `app/glue` copies them
once at build time. Check: `gate gameplay_input_types`.

## Mains power

The facility feature owns `PowerState` and `FacilityPower` and tags powered
entities with a `MainsPowered` component at spawn. Ambient and intermittent
sound sources that need mains carry `MainsPowered` too. Each consumer reacts
in its own feature. There is no shared `requires_power()` rule.
`Module::mains_powered()` is kind data that only the facility tagger reads;
consumers read the `MainsPowered` tag, never the kind. Check:
`gate power_state_scope` keeps `PowerState` and `FacilityPower` reads under
`crates/gameplay`, the stdio snapshot (`crates/app/src/playable`), the debug
decorator and tests. `gate requires_power` keeps the old shared
`requires_power()` rule from coming back.

## Glue

Glue is config. It copies data between features while plugins initialize and
does nothing else. Under `crates/app/src/glue`:

- systems register only in `Startup` or `OnEnter`;
- no constants, no string compares, no RNG, no `Update`, no `FixedUpdate`,
  no `Changed<T>`.

What glue does: loader collection to feature assets fill (also
`MenuAssets`), one-time settings copy, bus volume copy. Runtime settings
changes are written by the frontend settings panel directly into the
gameplay and audio resources, the primary `Window.mode` and the `Msaa` of
every `Camera3d`. At build, `app` sets the window mode from the saved
settings (`game_frontend::window_mode`) and adds
`game_frontend::CameraQualityPlugin`, whose observer gives each new
`Camera3d` the saved quality (`game_frontend::msaa`).

Check: `gate glue_config_only`. `gate app_settings_sync` keeps
`crates/app/src/lib.rs` free of `resource_changed::<GameSettings>`,
`Changed<GameSettings>` and `is_changed()` settings systems.

## Messages

Messages are for things that happened: `DoorCue`, `ExitOpened`,
`FlashbangHit`, `PickupCue`. Resources and components are for how things are.
Power is a resource, not a `PowerChanged` stream.

Achievements observe feature messages (`PickupCue`, `PowerCue`, `ExitOpened`,
`MonsterCue`, `FlashbangHit`) in `PostUpdate` before
`facility::ObjectiveSystems`; emitters never call into achievements.

One owner per message type: the defining plugin calls `add_message`, nothing
else. Check: `gate message_owners`.

Readers across a crate boundary declare `SystemSet` ordering or accept a
one-frame delay and say so in a `NOTE:`.

The frontend sends gameplay commands; it never builds the world.
`facility::RunPlugin` owns `StartRun { preset }`. Its one system sets
`GamePreset`, builds the first floor (which resets `FacilityPower` for the
run), spawns the actors and tags each top-level run entity with `RunRoot`.
The run anchor (`ActiveRun`) restores mains when it despawns, so menus and
cinematics are lit after a run. The menu Play and Retry and the app
`GamePlugin` only write `StartRun` and set `GameState::Playing`; they scope
`RunRoot` or `ActiveRun` to `Playing` with an observer.

## Frame step

`FrameStep` in `app` is the one owner of the simulation step. Capture and
stdio read it; capture has no default step. `FrameStep::DEFAULT` is 60 Hz
and `FrameStep::SCREENSHOTS` (30 Hz) is the step of the screenshot examples.
Check: `gate frame_rate_owner` allows `60` in one crate only and frame step
literals (`1_000_000_000 /`, `16_667`, `1.0 / 30.0`, `1.0 / 60.0`) only in
`crates/app/src/lib.rs`.
