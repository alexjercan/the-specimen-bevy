# Freesound ambience sources

These three files are low-quality Freesound OGG previews retained to reproduce the shorter edits used in the game. Their item pages display CC0 1.0. This is a source-page claim, not independent verification of authorship, recording rights, or original-file terms. Original downloads require a Freesound login and were not obtained. Credit each creator in `credits/CREDITS.md` and in the HTML catalog.

| Preview source | Creator and page | Preview URL | SHA-256 |
| --- | --- | --- | --- |
| `amb/furnace/furnace-iankath.ogg` | iankath, https://freesound.org/people/iankath/sounds/173991/ | https://cdn.freesound.org/previews/173/173991_2022935-lq.ogg | `8c728ff4d097a381ea2bac965af332b1178d6292c8f36cf9e4ad26e2532bdff7` |
| `amb/water/faucet-willstepp.ogg` | willstepp, https://freesound.org/people/willstepp/sounds/188293/ | https://cdn.freesound.org/previews/188/188293_2831691-lq.ogg | `671b5411597ec7f00a90043db616dfbc2b54a536fd46464d57bd3bc71613f2a4` |
| `amb/vent/wind-dblover.ogg` | DBlover, https://freesound.org/people/DBlover/sounds/405601/ | https://cdn.freesound.org/previews/405/405601_7846219-lq.ogg | `f957da677ce5c62b90380e9e5c97debb42b36226fb59678fa114805f7efc117e` |

`scripts/render_selected_ambience.py` produces mono 48 kHz PCM WAV edits. These edits are kept under `art/sounds/generated/amb/`, with byte-identical copies in `assets/sounds/amb/`. The furnace is a 24-second crossfaded loop; the faucet a 5-second fading burst; the wind a low-passed 18-second crossfaded loop. Source previews are not shown in the HTML catalog or loaded by the game.

| Game edit | SHA-256 |
| --- | --- |
| `furnace/burning.wav` | `770d3eceec309ca92110cd66976e40e3ff364abba425f0a59afd7e8a49c9bd20` |
| `water/faucet.wav` | `bd013de9a3643c63a9c98222598ac1d81cc2712698ded70ae28c65dfc6f527c1` |
| `vent/wind.wav` | `9f53591094dffc26269a614b5685e4b4d3ea4818dc06fbfd3057e9a974e9c26d` |
