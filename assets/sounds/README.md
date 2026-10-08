# Shipped sound cues

These 27 files are byte-identical copies of approved clips in
`art/sounds/`. Recorded footstep and door creators are credited explicitly in
`credits/CREDITS.md`. See `art/sounds/README.md` for source URLs, license
claims, source hashes, and editing history.

- `step/subway/{01,02,04}.ogg`: recorded by GboxMikeFozzy; the OpenGameArt
  [Footsteps](https://opengameart.org/content/footsteps-0) item displays CC0 1.0.
  Tile surface and shoe type were not independently verified.
- `door/{unlatch,swing/open,shut}/01.wav`: edits of rubberduck's recordings;
  the OpenGameArt [100 CC0 metal and wood SFX](https://opengameart.org/content/100-cc0-metal-and-wood-sfx)
  item displays CC0 1.0. Original OGG inputs were removed at the user's
  request, so these edits cannot be rebuilt without downloading the source.
- `hiding/{locker,table}/`: edited recordings from rubberduck's CC0-labeled
  metal/wood pack. The four approved WAVs were copied from `art/sounds/review/hiding/`.
- `amb/`, `fuse/`, `panel/`, and `ui/`: original synthesized project cues;
  project MIT license. The five added ambience cues are `amb/roomtone/conduit.wav`,
  `amb/boiler/tick/01.wav`, `amb/vent/hvac/01.wav`, `amb/tank/hum.wav`, and
  `amb/light/buzz/cool-low.wav`. Rebuild these five with
  `scripts/generate_ambience_review.py`; the earlier cues use
  `scripts/generate_sounds.py`. The low-quality recorded flicker preview and
  new drip candidates remain review-only and are not shipped.

License claims for recordings are based on the source item pages, not
independent authorship verification. Preserve this provenance with shipped
copies. Playback levels and spatial behavior need in-game review.
