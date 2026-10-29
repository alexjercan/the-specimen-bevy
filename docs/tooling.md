# Tooling

Everything runs inside `nix develop`. The dev shell has the Rust toolchain
from `rust-toolchain.toml`, Python, ffmpeg, `xvfb-run` and `trunk`.

## Game flags

| Flag | Effect |
| --- | --- |
| `--headless` | Logic only: `MinimalPlugins`, no window, no assets, no audio. Without `--stdio` the run is bounded and exits by itself. |
| `--stdio` | Reads JSON command lines from stdin and writes one JSON reply line per command to stdout. Works headless or windowed. The menu is skipped. |
| `--seed N` | Fixes the run's RNG seed (`u64`). Fuse placement and every other random choice follow it. |
| `--record PATH` | Writes a WebM video of a windowed `--stdio` run. Requires both. |
| `--video-profile preview\|final` | Picks the capture profile for `--record` and requires it. Default `preview`. |

There is no mute flag. Headless has no audio plugin. A rendered run plays
audio on the host device; CI jobs that need silence add a dummy output
device.

## stdio protocol

Each command line and each reply line carries a `version` field. The host
owns the input trace and can resend it; the game never replays on its own.

The Python host lives in `playtest/`: `harness.py` is generic and
`specimen.py` adapts it to this game. The host waits for the encode at the
end of a recording. Its `--encode-timeout` default is the selected profile
deadline plus a 180 s margin (300 s for `preview`, 780 s for `final`), the
same margin the record e2e uses.

## Video

`--record` uses `VideoProfile::PREVIEW` (640x360, crf 40, 120 s deadline) or
`VideoProfile::FINAL` (native size, crf 24, 600 s deadline). Both run at 30
fps and take every second simulation frame of the 60 Hz `FrameStep`.
`FrameStep::DEFAULT` is the one step owner; the screenshot examples use
`FrameStep::SCREENSHOTS` (30 Hz). The
profile sets fps, crf, output size, encode deadline and the staging
directory. Audio comes from `audio.jsonl`, a frame-indexed event log written
next to the staged frames by the capture crate, which watches every
`AudioPlayer` spawn, despawn and retune. Each event carries the voice's
`bus` (`world`, `ui`, `music`, or `unrouted` for a player without a `Bus`). The
logged volume is already the effective volume, so the mixer does not scale
by bus. The capture crate mixes the log
offline at 48 kHz into `audio.f32le` (same ear model as rodio) and ffmpeg
muxes it with the frames. There is no PCM tap; a run without an audio device
still records sound.

The staging directory goes away with its recording, also when the app
stops early. A recording still open when the app exits is finished or
reported: on a plain exit (the window closed) the game encodes the frames it
already captured; after an error exit (stdout closed, a frame failed to
save) or a failed encode it logs one `video ... was not written` error and
the exit status is an error.

What a recording cannot show: anything the headless logic decided without a
view. Record a windowed run or read the stdio replies.

## Profiling

The dev shell has `tracy` 0.13.1, `perf` and `cargo-flamegraph`. The root
crate has two cargo features that forward to Bevy: `tracy` enables
`bevy/trace_tracy`, `trace-chrome` enables `bevy/trace_chrome`.

### Tracy

Bevy 0.20 enables `tracing-tracy` 0.12, which uses `tracy-client` 0.19 and
bundles Tracy 0.14.1. The pinned nixpkgs still ships the Tracy 0.13.1
profiler; its protocol does not match the game client. Use a Tracy 0.14.1
profiler outside the dev shell until nixpkgs provides a matching version, or
use `--features trace-chrome` to record spans without Tracy. Do not pin an
older Tracy client alongside Bevy: that links two client versions and does
not make Bevy's spans compatible with the old profiler.

Start a Tracy 0.14.1 profiler and run `cargo run --features tracy` in another
shell. The profiler connects to the game on localhost and shows system spans
live.

### Flamegraph

`cargo flamegraph -- --headless --seed 1` samples the bounded run with
`perf` and writes `flamegraph.svg`. Add `--features tracy` or nothing; the
flamegraph needs no Bevy feature. If `perf` refuses to sample, lower
`kernel.perf_event_paranoid` on the host or run with the right capability.
The flamegraph shows CPU hot spots across all threads, not frame
boundaries.

### Chrome trace and `frametime.py`

```
RUST_LOG=bevy_ecs=info cargo run --features trace-chrome -- --headless --seed 1
python3 scripts/perf/frametime.py
```

The game's log filter sets `bevy_ecs=warn`, which drops the per system
spans. `RUST_LOG=bevy_ecs=info` overrides that one directive and keeps the
rest. Bevy writes `trace-<unix micros>.json` into the working directory, or
the path in `TRACE_CHROME`. The script takes the trace path or picks the
newest `trace-*.json`. It prints the frame count, frame time p50, p95 and
p99 from the `update` spans and the top 10 systems by self time. Then it
deletes the trace unless `--keep` is given. Without system spans it prints
the `RUST_LOG` hint. `python3 -m unittest scripts.perf.test_frametime` tests
the parser on a hand written trace.

### Frame diagnostics

Every windowed app has `FrameTimeDiagnosticsPlugin` (added by the fps
overlay); turn on the fps overlay in the settings panel to read it. A build
with the `debug` cargo feature also logs one `fps, frame time` line per
second at debug level, shown with `RUST_LOG=debug` or
`RUST_LOG=debug=debug` for that line only.

### What the numbers cannot show

A headless run has no render app, so a headless trace or flamegraph shows
logic cost only. `frametime.py` refuses to print GPU numbers from a headless
trace. A chrome trace never holds GPU time, even from a rendered run; use
Tracy on a rendered run for GPU spans. A rendered run under Xvfb uses the
software Vulkan driver and does not represent GPU cost on real hardware.
The e2e scenario `headless_frame_budget` asserts a p99 wall clock budget
per `app.update()` on the headless escape run. See `docs/testing.md`.

## Release builds

The Linux release job installs system libraries with apt-get and runs plain
`cargo build` on purpose. A binary built inside `nix develop` links
`/nix/store` paths and does not run on another machine. `rust-toolchain.toml`
keeps the compiler identical in both environments. The HTML5 job uses the
dev shell because `trunk` output has no native links.

## Release checklist

The `release` workflow (`.github/workflows/release.yaml`) runs on a `vX.Y.Z`
tag. It checks the tag form, builds the Linux, Windows and HTML5 archives
and takes the release notes from the tagged `CHANGELOG.md` section. Before
the tag:

1. Bump `version` in the root `Cargo.toml` and refresh `Cargo.lock`.
2. Move the `## [Unreleased]` notes in `CHANGELOG.md` under
   `## [X.Y.Z] - YYYY-MM-DD`. The workflow fails without that section.
3. Run `grep -rn 'TODO:' crates src` and resolve or accept each hit.
4. Run `nix develop --command ./tests.sh --full` and
   `nix develop --command cargo check --locked --features steam`.
5. Commit, tag `vX.Y.Z` on that commit and push the tag. A Halloween build
   runs the workflow by hand with `flavor: halloween` after the normal
   release exists.

## Scripts

| Script | Purpose |
| --- | --- |
| `tests.sh [--smoke|--full]` | Format, comment and architecture checks, tests, build, bounded headless smoke. Full mode adds clippy, the `debug,halloween` clippy and test runs and the rendered e2e. |
| `scripts/check-comments.py` | Rejects comments without an allowed marker. |
| `scripts/check-architecture.sh [gate...]` | Runs the architecture grep gates, all of them without arguments. |
| `scripts/gen-licenses.sh` | Regenerates `credits/THIRD-PARTY-LICENSES.txt`. |
| `scripts/perf/frametime.py [TRACE] [--keep]` | Summarizes a `trace_chrome` trace and deletes it. |
| `playtest/specimen.py` | Runs a model-driven playtest over stdio. |
