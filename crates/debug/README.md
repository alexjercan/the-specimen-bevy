# debug

Opt-in developer tools for `horror_game_bevy`. The game links this crate only
when you enable the `debug` Cargo feature. Default builds do not compile it.

## Usage

```bash
cargo run --features debug
```

- An FPS counter shows in the top-right corner of the window.
- Press `F12` to show or hide the inspector panel.
- The inspector panel shows the world: entities, components, resources, and
  assets. You can edit values in place.
- Use the `Wireframe` checkbox in the panel to draw all meshes as wireframes.

The inspector and the wireframe start off. The `--norender` mode does not add
the debug tools.

Wireframes work on native builds only. On wasm (WebGL or WebGPU) the
`Wireframe` checkbox has no effect.

## Use in other apps

Add `DebugPlugin` after `DefaultPlugins`:

```rust
app.add_plugins(DefaultPlugins).add_plugins(debug::DebugPlugin);
```

`DebugPlugin` adds the egui inspector only when `WinitPlugin` is present. It
adds the wireframe pass only when `PbrPlugin` is present. Headless apps get the
FPS text and the `DebugSettings` resource only.

Examples do not add `DebugPlugin`, so their screenshots do not show the
overlay.

## Tests

```bash
cargo test -p debug
```
