# probe

Frame-time capture for dedicated windowed examples. Running either example
starts the probe. The main game does not install it.

## Run

```sh
scripts/run-probes.sh
cargo run --release --example facility_probe -- --probe-out target/probe/facility.json
cargo run --release --example facility_overhead_probe -- --probe-out target/probe/overhead.json
```

The script runs both examples in order and writes separate JSON reports under
`target/probe/`. Pass extra probe flags to the script to apply them to both
runs, for example `scripts/run-probes.sh --probe-frames 600`. The first-person
example uses the main game's camera, fog and ceilings, but does not move its
camera. The overhead example hides ceilings and adds review lighting; it is a
different scene configuration, not a direct A/B comparison.


| Flag | Default | Meaning |
| --- | --- | --- |
| `--probe-warmup N` | `180` | Frames to discard after the scene is ready. At least one frame is discarded. |
| `--probe-frames N` | `900` | Frames in the measured window. Must be 1 or more. |
| `--probe-res WxH` | `1280x720` | Logical window size. The window is not resizable. |
| `--probe-present MODE` | `autonovsync` | `immediate`, `mailbox`, `fifo`, `fiforelaxed`, `autovsync` or `autonovsync`. |
| `--probe-label NAME` | `facility` | Row label. No commas, quotes or control characters. |
| `--probe-out PATH` | none | `.csv` appends one row (creates the header). `.json` overwrites one object. |

## What is measured

1. The probe waits until `GameAssetsState::Ready`, every facility part is
   rendered, a 3D camera exists, and every `WorldAssetRoot` scene instance is
   spawned. Setup frames are not measured.
2. It discards the warm-up frames.
3. It records the `Time<Real>` delta of each frame in the window.
4. It logs one `probe:` summary line with mean, p50, p95, p99 (nearest rank),
   min, max, mean FPS, window size, present mode, adapter, backend and build
   profile. Then it writes the output file, if one is set, and exits with code 0.

## Failure

The probe exits with a non-zero code and writes no stats when:

- the facility assets fail to load;
- the scene is not ready after 120 seconds;
- the window size or present mode changes, or there is not exactly one
  primary window;
- `WinitSettings` is not continuous;
- the app exits (for example, the window is closed) before the window is full;
- a frame time is not a finite positive value;
- the output file cannot be written, or a CSV file has a different header.
  The file is written to `PATH.tmp` and then renamed, so a failed write does
  not leave a partial row or object.

Invalid flag values are CLI errors (exit code 2).

## Valid A/B comparison

- Compare only rows with the same `width`, `height`, `present_mode`, `adapter`,
  `backend` and `profile`. Use `--release` for numbers you keep.
- Use the same `--probe-warmup` and `--probe-frames` for both sides.
- Under Xvfb the absolute frame times and FPS are distorted. The X server has
  no real display, so presentation and pacing do not match a desktop session.
  Use Xvfb numbers only to compare two runs from the same Xvfb setup, with the
  same screen size and the same probe flags.
- A tiling window manager can resize the window. The probe then fails. Float
  the window or use Xvfb.
- `autonovsync` can fall back to vsync on some surfaces. If p50 is near the
  display period (for example 16.7 ms), the run is capped by the display.
