# Upgrade Bevy to 0.20.0

- STATUS: CLOSED
- PRIORITY: 0
- TAGS: backlog

## User facts

- Upgrade this game from Bevy 0.19 to Bevy 0.20.0. The initial request was to plan only; the user then approved implementation and asked us to follow the 0.19-to-0.20 migration approach in `/home/alex/personal/template-bevy`.
- Review https://bevy.org/news/bevy-0-20/ and https://bevy.org/learn/migration-guides/0-19-to-0-20/ while implementing; recheck upstream guidance if it changes.

## Agent findings

- `Cargo.toml` pins workspace Bevy `0.19.1`; the workspace also uses
  `bevy_asset_loader 0.27.0`, `bevy_enhanced_input 0.26.0`, `bevy_rand 0.15.0`,
  `bevy_egui 0.40.1`, and `bevy-inspector-egui 0.37.0`. Their Bevy
  0.20-compatible versions and feature flags must be checked before changing
  the lockfile. Optional Steam, Halloween, debug, Tracy, and trace-chrome
  builds also need coverage.
- The migration guide changes lifecycle observers from `On<Add, T>` /
  `On<Insert, T>` / `On<Remove, T>` to `On<Add<T>>` / `On<Insert<T>>` /
  `On<Remove<T>>`. These occur across `crates/app`, `crates/gameplay`,
  `crates/frontend`, `crates/halloween`, and tests. Check all observer event
  signatures, including custom events, rather than blindly replacing text.
- `crates/ui/src/widgets.rs` and `crates/frontend` use deprecated
  `bevy::ui::Button` / `Interaction`; assess the supported
  `bevy_ui_widgets::Button`, `Hovered`, and `Pressed` migration without
  changing menu behavior. The guide also changes default text sizing and UI
  render retention; verify HUD, settings, menus, and achievement UI visually.
- The guide replaces naga_oil shader directives with WESL, changes some render
  scheduling to weak ordering, makes screen-space transmission opt-in, and
  changes `Tonemapping::None` semantics. Search the actual assets and rendering
  code for affected APIs and validate game lighting, transparency, screenshots,
  and capture. Do not adopt new release features such as Solari, DLSS, or BSN
  merely to update the dependency.
- The release workflow builds Linux and Windows native archives plus HTML5.
  Third-party compatibility and platform-specific packaging are upgrade risks,
  not yet proven blockers.

- Template reference: commit `8251c97` pins `bevy = "=0.20.0"`, updates `bevy_asset_loader` to `0.28.1`, migrates lifecycle observers, and replaces menu `Interaction::Pressed` test injection with `ui_widgets::Activate`.

## Delivery

- Intended result: one consistent Bevy 0.20.0 dependency graph, compiling and
  running the existing game with preserved gameplay, persistence, UI,
  rendering, input, and capture behavior on supported targets.
- Deletion list: remove obsolete Bevy 0.19 API calls/imports and superseded
  dependency versions from `Cargo.lock`; do not delete game features or assets
  as a migration shortcut.
- Sequence:
  1. Check the compatible releases, Rust/MSRV and feature flags of Bevy and all Bevy-facing crates; record blockers before updating. Keep the existing architecture and game behavior.
  2. Update workspace dependency declarations in `Cargo.toml` first, then `Cargo.lock`. Resolve one Bevy version across the graph and keep optional features functional.
  3. Migrate owner interfaces and their callers: lifecycle observers and other compiler-identified API changes, then UI widgets and input integration where required. Update associated tests with the owning code.
  4. Inspect shaders, camera/material settings, render ordering, and text/UI semantics against the 0.20 guide. Change only affected sites; compare rendered output and update intended visual baselines only after review.
  5. Run the verification below, fix regressions, and document any supported-target limitation and unrun check before closing this task.

## Verification

- `nix develop --command ./tests.sh --smoke` and `nix develop --command ./tests.sh --full` pass. Run one foreground Cargo command at a time and use the shell job cap.
- Check native feature combinations used by this project (`debug`, `halloween`, `steam`, `tracy`, `trace-chrome`) and the HTML5 build; verify Windows build in CI or record why it cannot be checked locally. Check `Cargo.lock` for duplicate Bevy major/minor versions.
- Exercise a bounded headless run and an asserting stdio/e2e scenario for gameplay and input; verify achievements/settings save and reload. Run rendered tests or manual captures under Xvfb for menu/settings, HUD, lighting/transparency, audio, and screenshot/video paths. Record visual differences attributable to Bevy 0.20.
- Confirm existing GitHub release packaging still starts on Linux and Windows and the web build loads, or record the limits of the available platform checks.

## Implementation evidence (2026-10-29)

- Pinned Bevy `=0.20.0` and updated `bevy_asset_loader 0.28.1`, `bevy_enhanced_input 0.27.2`, `bevy_rand 0.16.0`, `bevy_egui 0.43.1`, and `bevy-inspector-egui 0.38.0`. The lockfile contains one Bevy 0.20 minor line (the Bevy umbrella crate is 0.20.0; internal crates resolve to 0.20.1).
- Migrated lifecycle observers, menu button widgets and activation, and debug Egui schedule cleanup. The input companion crate changed `Cardinal::new` argument order from north/west/south/east to north/east/south/west; corrected player bindings and confirmed all 16 player collision tests pass. Added an identity transform to the exit room parent to avoid a Bevy 0.20 hierarchy warning.
- `nix develop --command ./tests.sh --smoke` and `nix develop --command ./tests.sh --full` passed. The full run included debug/Halloween Clippy and tests plus the rendered Xvfb e2e test. A final smoke run passed after the exit-room hierarchy fix. `cargo check --workspace --all-targets --features steam,tracy,trace-chrome,debug,halloween` passed. `cargo check --target wasm32-unknown-unknown --locked` and `trunk build --release --features halloween` passed. Regenerated `credits/THIRD-PARTY-LICENSES.txt` with `scripts/gen-licenses.sh`; cargo-about logged registry harvest warnings but completed.
- A review found that Bevy 0.20's Tracy integration requires Tracy 0.14.1, while the pinned dev shell provides 0.13.1. Removed the old direct client pins so only one Tracy client remains and documented the need for an external 0.14.1 profiler (or `trace-chrome`) in `docs/tooling.md`. Optional-feature `cargo check` passes; an interactive Tracy session was not run. The unused generic UI button helper now includes the same `Hovered` component as menu buttons. The exit-room identity transform is intentional: the exterior corridor is its child, and Bevy reported missing `GlobalTransform` on the parent.
- No local native Windows build, browser launch, Steam-client test, screenshot/video capture, or manual visual comparison was run. GitHub release workflow builds Windows and web archives, but those release jobs have not run on this working tree. The full suite's rendered Xvfb scenario passed; it is not a complete visual review of every menu and lighting state. No game shader directives or `Tonemapping::None` usages were found in the audited sources and assets.

## Done when

- Workspace and lockfile use Bevy 0.20.0 with compatible integrations; the relevant APIs compile without migration deprecations.
- The checks above pass or have specific recorded blockers; gameplay and supported platform builds show no unreviewed regressions.
- The task records the actual changes, evidence, and any skipped validation before it is closed.
