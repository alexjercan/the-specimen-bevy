# Halloween cosmetic update on a separate PR branch

- STATUS: OPEN
- PRIORITY: 0
- TAGS: backlog

## User facts

- Plan a Halloween update on a separate branch as a PR. Other wishlist work came first.
- Consider alternate fuse pickup models (for example candy), lit pumpkins and other sparse orange decorations, and orange glowing monster eyes.
- Include a true/false way to turn the theme on or off.
- Latest user choice (2026-10-11): activation is the compile-time Cargo feature `halloween`. The theme lives in an isolated decorator crate `game_halloween`, which adds seasonal props, the candy fuse visual, and orange monster eyes without changes to gameplay or level code. The update is cosmetic only.
- Add a dedicated gallery example like `prop_gallery`, with orbit and selection, built from reusable gallery components.
- Do not push, release, or open a PR. No commits unless needed for local review.

## Decisions

- Automode decision A (reversible; not an explicit personal user sign-off): use the candy fuse model everywhere the fuse model appears, including pickups and panel insertion. Fuse blade terminals stay recognizable. Collision, interaction, and gameplay stay unchanged.
- The original monster model is a Sketchfab asset (see `credits/`). It is not modified. The eyes use a reversible runtime overlay: an emissive orange material on the `Eyes_low_Deer_head1_0` mesh plus two additive halo spheres on the head joint.

## Agent findings

- The `halloween-update` sprout at `/home/alex/.cache/sprouts/the-specimen-bevy/halloween-update` is based on 9a960f5 (not 4d23250).
- Bevy has no bloom in this game, so the emissive eye material alone shows only as a small dot from the side. The halo spheres make the eyes read from the front and the side.
- Main menu background props are children of the menu root. Decor uses an intermediate `HalloweenDecor` entity so that `attach_prop_glow` does not act on it.

## Implementation

- Root `Cargo.toml`: feature `halloween = ["game_core/halloween", "dep:game_halloween"]`, workspace member `crates/halloween`, example `halloween_gallery` (requires the feature).
- `crates/core/src/lib.rs`: adds `game_halloween::HalloweenPlugin` only in the rendered `AppBuilder` branch. Headless and `--norender` builds skip it.
- `crates/halloween`:
  - `fuse.rs`: an `Insert<WorldAssetRoot>` observer swaps the `fuse_pickup` scene handle for `halloween/candy_fuse.glb`. Entities, components, and colliders stay the same.
  - `decor.rs`: an `Add<Prop>` observer adds 7 decorations (4 jack-o'-lanterns on steel drums, 2 small pumpkins on crates on the first floor, 1 jack-o'-lantern on the main menu crate). Jack-o'-lanterns have a shadowless orange candle point light. Decor has no `Prop`, `PropCollider`, or `PickupKind`.
  - `eyes.rs`: the emissive eye material and the halo overlay.
  - `HalloweenAssets` loads in the `GameAssetsState::Loading` state.
- `gameplay`: `MONSTER_HEAD_JOINT` is now public. No behavior change.
- Assets: `scripts/generate_halloween.py` (Blender, reuses `generate_facility.py` helpers), `scripts/generate-halloween.sh`, `scripts/promote-halloween-modules.sh`. The output is in `art/visuals/generated/halloween/` and is promoted to `assets/halloween/` (candy_fuse 664 tris, jack_o_lantern 452 tris, pumpkin_small 368 tris). Credits: `credits/ASSET-SOURCES.md`, "Halloween decoration kit".
- Examples: `examples/gallery/mod.rs` (shared gallery: exhibits, orbit, zoom, browse, optional capture). `prop_gallery` uses it with the same exhibits and zoom limits. `halloween_gallery` shows the candy fuse, the pumpkins, the decor placements, and the monster eyes. `facility_shots` has eye-level shots of the decorated drums and crate.
- Browser builds: `trunk build --features halloween` enables the theme. `index.html` is unchanged, so the default browser build stays without the theme. No Trunk build was run; `cargo check --target wasm32-unknown-unknown --features halloween` passes.

## Verification (2026-10-11)

- `cargo test --workspace --locked`: 502 passed, 0 failed. With `--features halloween`: 503 passed, 0 failed.
- Example tests (`halloween_gallery`, `prop_gallery`, `facility_shots`): pass.
- `cargo check --locked --all-targets`, with and without the feature: pass.
- `cargo check --locked -p the-specimen-bevy --target wasm32-unknown-unknown`, with and without the feature: pass.
- `env -u DISPLAY timeout 60s cargo run --locked [--features halloween] -- --norender`: runs 60 s with no panic.
- `cargo clippy`: no warnings in the changed files. Workspace `-D warnings` fails on pre-existing lints in `game_audio` and `game_ui`.
- `cargo fmt --all -- --check`: the changed files are clean. Drift exists only in files this task does not touch.
- `scripts/check_facility_modules.py assets/halloween`: 3 modules ok. The generator output is deterministic.
- `scripts/check-comments.py`: one pre-existing issue at `crates/audio/tests/unit/support.rs:4`.
- Screenshots (Xvfb, 1280x720):
  - `facility_shots`, on and off: jack-o'-lanterns sit on the drums with an orange glow. The small pumpkins sit on crates. With the feature off, the shots are the same except that the decor is absent.
  - `halloween_gallery`: the candy fuse reads as candy and as a fuse next to the base fuse. The jack-o'-lantern face glows. The monster eyes show as two orange eyes from the front and the side.

- Code review (read-only subagent): the fuse swap ordering, pickup identity, decor transforms, halo lifetime, and headless isolation are sound. It found a `prop_gallery` minimum zoom change from the gallery refactor (fixed: `Gallery::min_radius`), an unclear Trunk note (fixed above), and the candle outage behavior (recorded below).

## Notes and risks

- The main menu lantern is at the bottom-right edge of the menu view. It was not visible in the 800x600 Xvfb capture. It is not verified at 16:9.
- The candy fuse swap in a live game run is verified by unit tests and the gallery, not by an in-game screenshot.
- Decor candles stay lit during power outages. They do not follow `LightIntensity` and add small orange light pools (2.5 m range) in the dark. This includes Insane mode, where power is out from the start.
- The orange eye halos make the monster slightly easier to see in the dark. This is a fairness change to accept or tune (`HALO_RADIUS`, `HALO_STRENGTH` in `crates/halloween/src/eyes.rs`).
- The jack-o'-lantern skin looks slightly olive under the gallery lights.
- Trunk copies `assets/halloween/` into browser builds even with the feature off (about 100 KB).

## Delivery

- Implemented in the sprout, not on master. Not committed, pushed, or opened as a PR.
- Next: user review of the screenshots and the risks above, then commit and open a PR only when the user asks.

## Done when

- The approved cosmetic scope and switch are implemented and reviewed, required checks pass, and a PR from the Halloween branch is ready for review. No release or deployment is implied.
