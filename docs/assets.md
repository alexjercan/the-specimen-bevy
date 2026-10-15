# Assets

`art/` holds sources, research and generated candidates. `assets/` holds what
the game loads at runtime. Nothing goes from `art/` to `assets/` without a
provenance record.

## Flow

1. Fetch. A script under `scripts/` downloads a source into
   `art/<kind>/sources/` with its checksum, for example
   `scripts/fetch-facility-textures.sh`.
2. Generate. A script renders or edits it into `art/<kind>/generated/`, for
   example `scripts/generate-facility.sh` or `scripts/render_monster_sounds.py`.
3. Check. A checker validates the output, for example
   `scripts/check_facility_modules.py`.
4. Promote. A promote script copies checked output into `assets/`, for
   example `scripts/promote-facility-modules.sh`.

Edit the source or the generator, then rerun the chain. Never edit a file
under `assets/` by hand.

## Provenance

Before a file ships under `assets/`, record in `credits/ASSET-SOURCES.md`:
source URL, author, license, the changes made, and any required credit.
Licensed credits go in `credits/CREDITS.md`. License texts go in
`credits/licenses/`. Third-party crate licenses come from
`scripts/gen-licenses.sh`.

Direction notes and candidate research stay in `art/*/README.md`. The original menu music generation, OGG encoding, and promotion steps are in `art/audio/README.md`; its provenance is in `credits/ASSET-SOURCES.md`.

## Loading

`crates/assets` declares the game's `bevy_asset_loader` collections. The
only other crate with asset paths is the `halloween` decorator, which
declares its own collection of `assets/halloween/` props; it is compiled only
with the `halloween` feature. Each gameplay feature declares its own path-free
assets resource; `app/glue` fills it from the collection once loading is
done. The loading state is `Loading -> Ready | Failed`; a failed load exits
the app with an error.
