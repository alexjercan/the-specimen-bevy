# Research and implement first-person controller with headless agent input

- STATUS: OPEN
- PRIORITY: 0
- TAGS: backlog

## User facts

- Keep the gameplay character controller in a separate file from `controller/wasd_camera.rs`; that camera is for debugging only.
- No jumping. Gameplay movement is WASD with Shift to run. Use `bevy_enhanced_input` for gameplay input.
- Provide an input layer so commands from an agent can control the same game in `--norender` mode, following Nova Protocol's approach. Begin with research/scouting; do not implement the controller/channel in this research pass.
- Longer-term work includes collisions with walls, doors and solid props, and door animations. Do not conflate that with the first input/controller slice.

## Agent findings (read-only; verified against source)

- Nova uses `bevy_enhanced_input` 0.26.0 in multiple crates (`~/personal/nova-protocol/Cargo.toml`, `crates/nova_ship/Cargo.toml`). Its debug camera (`crates/nova_ship/src/camera/wasd_controller.rs`) has a separate input-context marker, `actions!` bundle and observers. The gameplay flight rig (`crates/nova_ship/src/input/player/flight_rig.rs`) obtains named bindings from `nova_input::InputBindings` and rebuilds the rig on rebind; this is more machinery than the initial WASD slice needs.
- Nova's named input layer lives in `crates/nova_input/src/{lib,registry,dispatch}.rs`. `dispatch::apply(world, name, phase)` resolves the binding and presses/releases the underlying Bevy input resource, not `ActionMock`, so enhanced-input modifiers and conditions still run. It records the pressed source to pair releases correctly. The actual compatibility of 0.26.0 with this repository's Bevy 0.19.1 still needs a compile check.
- Nova's channel lives in `crates/nova_channel/src/{protocol,apply,lib,runner}.rs`. It accepts line-delimited JSON; input names use `<group>.<name>` with `start`/`stop`. The input writer runs in PreUpdate after `bevy::input::InputSystems` and before `EnhancedInputSystems::Prepare`. Releases are processed even when the action context is inactive. In step mode, it uses `TimeUpdateStrategy::ManualDuration` and replaces the headless runner to advance on driver ticks. Boot waits for loading to finish before accepting commands. This is a reference design, not a direct port specification.
- Nova's external agent path (`crates/nova_bench/src/{game,agent/mod,agent/socket}.rs`) spawns the game with `--norender --channel step`, relays commands to its stdin and reads snapshots from stdout; a separate socket connects the referee and agent. The full bench/referee is not needed just to establish a playable headless command channel.
- Here, `crates/core/src/lib.rs` headless composition uses MinimalPlugins and a 10 ms schedule runner, without enhanced input or a command channel. `src/main.rs` builds the first floor in `--norender` but exits after eight frames; the normal game starts `ControllerPlugin` and `PlayerController` from the debug `controller/wasd_camera.rs`. That plugin polls `ButtonInput` and permits vertical free flight. No agent-input transport is implemented here.
- Existing room/door data are authored in `crates/gameplay/src/levels/{builder.rs,first_floor_builder/}`; current DoorState is Closed/Open and door panel rendering is in `levels/render.rs`. The first movement slice need not change these components.

## Delivery (phased proposal, not yet approved implementation)

1. Add a distinct gameplay controller module in `crates/gameplay/src/controller/` and a minimal named gameplay-input layer. Wire `bevy_enhanced_input` into both normal and headless composition; keep the debug fly camera independent and do not run both movement plugins on the same player. Map WASD to a planar Vec2 movement action and Shift to a held run modifier. No jump or vertical movement; preserve yaw/look behavior or decide its mapping before implementation. Start with collision-free movement at constant eye height so input parity can be checked independently.
2. Replace the eight-frame `--norender` placeholder with an explicit, opt-in agent command path. Feed named press/release actions through the same bindings consumed by the enhanced-input rig; drive deterministic ticks, emit machine-readable acknowledgements and player state, and handle held-key cleanup, bad commands, startup readiness, and exit. Keep command transport separate from movement/collision logic. Scope whether this needs a new crate and whether mouse look also needs a command axis before implementation.
3. In later slices, add swept-footprint collision against authored solid walls, door frames, panels and selected solid props; add door interaction and panel animation with collision tied to the visible door state. Verify door clearance and contact/slide edge cases. Do not derive collision from every GLB decoration.

## Open decisions before implementation

- Input-layer boundary: a focused gameplay module or a small reusable input crate; how named bindings/rebinding are represented for the first slice.
- Gameplay mouse-look controls and behavior in headless commands; whether to retain debug camera controls only in debug mode.
- Exact agent transport/protocol and output state, plus how to gate game boot when headless does not load assets.
- Whether Shift changes speed only while held, and initial walk/run speeds (playtest values).

## Verification

- Compile-test `bevy_enhanced_input` against Bevy 0.19.1 without downgrading Bevy; test actual rig actions, simultaneous axes, diagonals, run hold/release, inactivity, and no vertical movement.
- Test the same command stream through input injection and real enhanced-input preparation; compare rendered/headless movement with controlled time. Test command errors, paired releases, startup readiness, no hanging, and no GPU/window dependency for `--norender`.
- Keep integration tests in `crates/gameplay/tests/` or another crate's tests, not in `src`; run fmt, scoped Clippy and workspace checks. Do not launch the game or examples unless the user permits it.

## Done when

- This research task is documented and reviewed before implementation begins; then separately implement and verify each approved slice. No controller, collision system, or agent channel is claimed complete by this task specification.
