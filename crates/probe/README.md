# probe

Frame-time measurement plugin for applications that install `ProbePlugin`. The
main game and the current facility screenshot example do not install it. The
old facility probe examples and their runner script were retired in favor of
`facility_shots`, which captures images rather than timing data.

The probe waits for its application's readiness callback, discards warm-up
frames, and records a fixed window of `Time<Real>` deltas. It reports mean,
p50, p95, p99, minimum, maximum, mean FPS, resolution, present mode, adapter,
backend, and build profile. Invalid arguments or incomplete measurement cause
a nonzero exit. See `ProbeArgs` in `src/lib.rs` if integrating the plugin with
a new application. Do not compare absolute frame times across different Xvfb
setups or display configurations.

For current facility images:

```sh
BEVY_ASSET_ROOT="$PWD" nix develop -c xvfb-run -a cargo run --example facility_shots
```

Shots are written under `target/facility-shots/`; set `FACILITY_SHOTS_DIR` to
change the output directory.
