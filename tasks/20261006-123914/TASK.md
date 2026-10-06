# Implement autopilot crate

- STATUS: CLOSED
- PRIORITY: 0
- TAGS: autopilot

Imported `crates/autopilot` from `~/personal/template-bevy/crates/autopilot` (MIT,
Copyright (c) 2025 Alexandru Jercan; license retained in
`crates/autopilot/LICENSE`). It has no dependency on `crates/capture`.
The example uses both independent crates. Capture adapts the screenshot and loop patterns in
`~/personal/nova-protocol/crates/nova_autopilot/src/{capture,loops}.rs`; it does
not copy Nova-specific gameplay or its audio implementation.

`facility_gallery` reads `art/visuals/generated/facility.glb` through
`AssetPlugin.file_path`, with no copy into `assets/`. Generate the glTF first.
The camera looks from (0, 1.6, -0.6) toward (0, 1.3, -16.5). Corridor and
room point lights illuminate the ceiling-enclosed geometry. Bevy's JPEG
feature decodes the glTF's embedded textures.

From the repository root:

```sh
# Capture a PNG and a 30 fps corridor-walk WebM (requires ffmpeg):
nix develop -c cargo run --example facility_gallery
# On a host with no normal display:
nix develop -c xvfb-run -a cargo run --example facility_gallery
```

The example adds `capture::CapturePlugin::new(30)`, which initializes
screenshot capture and shared state, and includes the loop capture plugin.
Registering `AutopilotPlugin` starts the script without an enable flag. The
run deadline defaults to 120 seconds; `.with_deadline_secs(value)` overrides
it when constructing the plugin. The script waits for the glTF scene and
180 frames for first-run GPU pipeline
compilation. `capture::screenshot::screenshot_start` requests Bevy's asynchronous
primary-window screenshot, writes
`art/visuals/screenshots/facility_gallery.png`, rejects a uniform/blank image,
then acknowledges the successful write. The example waits for
`screenshot_written_at(path)` and `loop_written_at(path)` predicates in its
steps, so autopilot has no
capture-specific dependency or completion gate. Capture reports write and
encode failures through `AppExit::error()`. Recursive scene-dependency failures
are reported explicitly.

`capture::loops::LoopCapturePlugin::new(30)` fixes the simulation timestep
to 1/30 second. It records one numbered PNG readback per rendered frame when
the example starts its loop. `loop_start` accepts
an output name (default `<name>.webm` in the current directory) or an explicit
`.webm` path; the gallery passes `art/visuals/screenshots/facility_gallery.webm`.
`loop_end` drains readbacks and encodes VP9 with ffmpeg.
The example walks the camera forward for 60 scripted frames. Video fails
nonzero if ffmpeg is absent or encoding fails. There is no frame cap;
`loop_end` stops frame requests.
A successful repeat run overwrites an existing video at the same path;
the previous file remains until ffmpeg starts encoding. Unique staging directories live in the system temp
directory; successful capture removes only its own staging directory.
Only the loop encoder requires ffmpeg. The example records both captures during an autopilot run.

Optional audio hook: `capture::loops::loop_audio_path(world)` exposes a unique
staging path during an open loop. A game-specific adapter may write stereo
44,100 Hz little-endian `f32` PCM there; if present, ffmpeg muxes it as Opus.
Without that file the video is silent. There is no native game-audio loopback
or automatic Bevy audio interception in this implementation; a future
application adapter must produce the PCM. Empty or incomplete stereo samples
fail encode explicitly.

Proof (2026-10-06):

```sh
nix develop -c cargo test -p capture -p autopilot
nix develop -c cargo check --example facility_gallery
nix develop -c cargo clippy -p capture -p autopilot --tests -- -D warnings
nix develop -c cargo fmt --all -- --check
nix develop -c xvfb-run -a cargo run --example facility_gallery
nix develop -c ffprobe -v error -count_frames -select_streams v:0 -show_entries stream=nb_read_frames:format=duration -of default=noprint_wrappers=1 art/visuals/screenshots/facility_gallery.webm
```

Earlier captures exited 0. The recaptured PNG is 1280x720 with 18,303 colors
and shows the illuminated corridor. WebM is 1280x720, 30 fps, 62 frames,
2.066 seconds, with no audio track (no PCM adapter connected).
