# Music

`art/audio/generated/` holds synthesized music loops; `assets/audio/` holds
runtime copies by cue, separate from the sound-effect tree under
`art/sounds/` and `assets/sounds/`.

`python3 scripts/generate-menu-music.py` synthesizes the main menu's dark
drone loop to `generated/menu/ambient-loop.wav`, encodes it to
`generated/menu/ambient-loop.ogg` with `ffmpeg`, and promotes a byte-identical
copy to `assets/audio/menu/ambient-loop.ogg`. The loop is a project-authored
synthesis (harmonic sine partials plus a filtered noise bed, 16 s with a
1.5 s crossfaded tail so `PlaybackMode::Loop` has no seam); no third-party
source is involved. Rerun the script after editing it to regenerate and
repromote.
