# Testing

`./tests.sh --smoke` is the fast loop. `./tests.sh --full` is what CI runs.
It adds clippy, a second clippy and test run with
`--features debug,halloween`, and the rendered e2e under `xvfb-run`.

The rendered step sets `VK_ICD_FILENAMES` to `$BEVY_SOFTWARE_VK_ICD` when
the caller did not set it. The dev shell (`flake.nix`) exports that path to
the Mesa lavapipe driver, so the rendered e2e runs on a machine or CI runner
without a Vulkan driver. A caller that sets `VK_ICD_FILENAMES` keeps its own
driver.

The `steam` feature is not in `--full`: the Steamworks SDK is missing at run
time. Run `cargo check --locked --features steam` by hand before a release.

## What proves what

- Headless (`--headless`) proves logic: states, messages, resources, the
  facility layout, fuse placement, the monster's decisions. It cannot prove
  visuals or audio.
- Rendered (windowed, under Xvfb in CI) proves visuals and audio: meshes
  spawn, lights react, sounds are sent to the bus.
- A clean exit is not gameplay evidence.

## Kinds of tests

- Unit and plugin tests live next to the code in each crate. They add the
  owning plugin and assert on behavior, never on plugin presence.
- `tests/cli.rs` runs the built binary with each flag combination.
- `tests/e2e/` holds asserting autopilot scenarios. Each scenario is defined
  in the test binary that runs it, never under `src/`. At least one headless
  and one rendered scenario run in CI. The rendered one runs under
  `xvfb-run`. The switches pick the runs:
  `cargo test --test e2e` runs the headless ones, `-- --rendered` runs the
  rendered escape run, and `-- --record` runs the record run only.
- Stdio sessions and the record run are local only. Neither `tests.sh` nor
  CI runs `--record`. Run it with:
  `BEVY_ASSET_ROOT=$PWD nix develop --command bash -c 'VK_ICD_FILENAMES="$BEVY_SOFTWARE_VK_ICD" xvfb-run -a cargo test --locked --test e2e -- --record'`.
  The in-process stdio tests and the `tests/cli.rs` flag tests stay in the
  normal test run.
- Examples under `examples/` are screenshot and manual playtest tools. They
  are compile-checked in CI and never run as tests. They never assert.

## Smoke

`tests.sh` runs `the-specimen-bevy --headless --seed 1`, which exits by itself
after a bounded number of frames. The step fails on any non-zero status. A
timeout is a failure, never a pass.

## Architecture gates

`scripts/check-architecture.sh` runs the grep gates named in
`docs/architecture.md` and `docs/coding-conventions.md`. `tests.sh` runs
every gate in both modes. A gate and its rule line in the doc are added or
removed together.

`nix flake check` builds the game package only. The test suite runs through
`nix develop --command ./tests.sh --full`, which is what CI does; there is
no sandboxed flake check for it.

## Performance

The e2e scenario `headless_frame_budget` in `tests/e2e/main.rs` runs the
escape autopilot headless, measures the wall clock time of every
`app.update()` and fails when the p99 exceeds `FRAME_BUDGET_P99` (50 ms). It
prints the measured p50 and p99. The budget stays generous until a real
regression sets a tighter number. See `docs/tooling.md` for profiling.
