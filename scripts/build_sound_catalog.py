#!/usr/bin/env python3
import argparse
import html
import os
import pathlib
import subprocess
import sys
import wave
from urllib.parse import quote

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "art/sounds/catalog.html"
SOURCES = (ROOT / "art/sounds", ROOT / "assets/sounds")
EXTENSIONS = {".wav", ".ogg", ".mp3", ".flac"}
APPROVED_FILES = {
    "art/sounds/source/step/subway/01.ogg",
    "art/sounds/source/step/subway/02.ogg",
    "art/sounds/source/step/subway/04.ogg",
    "art/sounds/generated/door/unlatch/01.wav",
    "art/sounds/generated/door/swing/open/01.wav",
    "art/sounds/generated/door/shut/01.wav",
    "art/sounds/generated/fuse/pickup/01.wav",
    "art/sounds/generated/panel/install/01.wav",
    "art/sounds/generated/amb/boiler/rumble.wav",
    "art/sounds/generated/amb/roomtone.wav",
    *{f"art/sounds/review/amb/{name}.wav" for name in (
        "roomtone/conduit", "boiler/tick/01", "vent/hvac/01", "tank/hum",
        "light/buzz/cool-low",
    )},
    *{f"art/sounds/review/hiding/{name}.wav" for name in (
        "locker/open", "locker/close", "table/enter", "table/leave"
    )},
    *{f"art/sounds/generated/ui/{cue}/01.wav" for cue in (
        "back", "confirm", "denied", "focus", "hover", "pause", "press", "resume"
    )},
}
REVIEW_FILES = {
    "art/sounds/review/amb/light/flicker/recorded-01.ogg",
    *{f"art/sounds/review/amb/{name}.wav" for name in (
        "drip/01", "drip/02", "drip/03", "drip/04",
        "pipe/flow", "vent/rattle/01", "boiler/fire", "alarm/pulse",
        "lab/console", "lab/console/beep-01", "outside/wind", "low/pressure",
    )},
    *{f"art/sounds/review/amb/web/{name}.ogg" for name in (
        "drip/single-metaepitome", "drip/faucet-willstepp",
        "drip/metal-wasabiwielder", "pipe/flow-pipelenisf",
        "boiler/furnace-iankath", "outside/wind-dblover",
    )},
}
DRAFT_FILES = {
    f"art/sounds/review/amb/{name}.wav" for name in (
        "light/buzz/cool", "light/flicker/01", "distant/settle/01",
    )
}
CATEGORIES = {
    "step": "A. Player self (bus self)",
    "door": "B. Doors (bus world)",
    "fuse": "C. Objective, pickup and panel (bus world and ui)",
    "panel": "C. Objective, pickup and panel (bus world and ui)",
    "amb": "D. Facility ambience (bus ambience)",
    "ui": "E. UI and front end (bus ui)",
    "hiding": "G. Hiding (bus world)",
}
STEP_CREDIT = ('GboxMikeFozzy, "Footsteps" (CC0 1.0)', "https://opengameart.org/content/footsteps-0")
METAL_WOOD_CREDIT = ('rubberduck, "100 CC0 metal and wood SFX" (CC0 1.0)', "https://opengameart.org/content/100-cc0-metal-and-wood-sfx")
RECORDED_PATHS = {
    path: STEP_CREDIT if path.startswith("art/sounds/source/step/") else METAL_WOOD_CREDIT
    for path in APPROVED_FILES
    if path.startswith(("art/sounds/source/step/", "art/sounds/generated/door/", "art/sounds/review/hiding/"))
}
RECORDED_PATHS["art/sounds/review/amb/light/flicker/recorded-01.ogg"] = (
    'mmaruska, "Lights Flicker On.wav" (page-labeled CC0 1.0, Freesound preview)',
    "https://freesound.org/people/mmaruska/sounds/232447/",
)
for name, creator, sound_id in (
    ("drip/single-metaepitome", "metaepitome", 165206),
    ("drip/faucet-willstepp", "willstepp", 188293),
    ("drip/metal-wasabiwielder", "WasabiWielder", 360735),
    ("pipe/flow-pipelenisf", "Pipelenisf", 620620),
    ("boiler/furnace-iankath", "iankath", 173991),
    ("outside/wind-dblover", "DBlover", 405601),
):
    RECORDED_PATHS[f"art/sounds/review/amb/web/{name}.ogg"] = (
        f"{creator} (page-labeled CC0 1.0, Freesound preview)",
        f"https://freesound.org/people/{creator}/sounds/{sound_id}/",
    )
STYLE = """body{background:#101416;color:#e2e5df;font:16px system-ui,sans-serif;max-width:1000px;margin:2rem auto;padding:0 1rem}h1,h2{color:#b0d9cf}section{margin:2rem 0}article{background:#1a2224;border:1px solid #354644;border-radius:8px;margin:.6rem 0;padding:.8rem 1rem}article.review{background:#302919;border-color:#c69a46}article.review small{color:#f2cf83}strong,small{display:block}small{color:#9aada9}audio{display:block;width:100%;margin:.5rem 0}svg{display:block;width:100%;height:72px;background:#111b1b;border-radius:4px}input{background:#1a2224;color:#fff;border:1px solid #6e8b84;border-radius:4px;padding:.5rem;width:min(25rem,95%)}.empty{color:#c1a886}"""


def waveform(path, columns=240):
    if path.suffix.lower() == ".ogg":
        try:
            decoded = subprocess.run(
                ["ffmpeg", "-nostdin", "-v", "error", "-i", str(path),
                 "-vn", "-ac", "1", "-ar", "48000", "-f", "s16le", "pipe:1"],
                capture_output=True, timeout=30, check=True,
            )
        except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
            return "<small>Cannot decode waveform.</small>"
        raw = decoded.stdout
        channels = 1
        frames = len(raw) // 2
    elif path.suffix.lower() == ".wav":
        try:
            with wave.open(str(path), "rb") as sound:
                if sound.getcomptype() != "NONE" or sound.getsampwidth() != 2:
                    return "<small>Waveform needs uncompressed 16-bit PCM.</small>"
                channels = sound.getnchannels()
                frames = sound.getnframes()
                raw = sound.readframes(frames)
        except (OSError, EOFError, wave.Error):
            return "<small>Cannot read waveform.</small>"
    else:
        return "<small>Waveform needs a WAV or OGG source.</small>"
    if not frames:
        return "<small>Empty file.</small>"
    import array

    samples = array.array("h")
    samples.frombytes(raw)
    if samples.itemsize != 2:
        return "<small>Unsupported PCM layout.</small>"
    if sys.byteorder != "little":
        samples.byteswap()
    peaks = []
    for x in range(columns):
        first = x * frames // columns
        last = max(first + 1, (x + 1) * frames // columns)
        bucket = samples[first * channels:last * channels]
        peaks.append(max((abs(value) for value in bucket), default=0) / 32768)
    lines = "".join(
        f'<line x1="{x}" x2="{x}" y1="{36 - 32 * peak:.2f}" y2="{36 + 32 * peak:.2f}"/>'
        for x, peak in enumerate(peaks)
    )
    return f'<svg viewBox="0 0 {columns} 72" preserveAspectRatio="none" role="img" aria-label="Audio waveform"><g stroke="#8ccfb3" stroke-width="1">{lines}</g></svg>'


def collect(sources=SOURCES):
    return sorted(
        path for source in sources if source.exists()
        for path in source.rglob("*")
        if path.is_file() and path.suffix.lower() in EXTENSIONS
        and path.relative_to(ROOT).as_posix() in APPROVED_FILES | REVIEW_FILES
    )


def sound_parts(path):
    part = path.relative_to(ROOT).parts[2:]
    if part and part[0] in ("generated", "recorded", "source", "review"):
        part = part[1:]
    return part


def build(paths, output=OUTPUT):
    groups = {}
    for path in paths:
        relative = path.relative_to(ROOT)
        part = sound_parts(path)
        category = CATEGORIES.get(part[0], part[0]) if len(part) > 1 else "misc"
        groups.setdefault(category, []).append(path)
    sections = []
    for category, sounds in sorted(groups.items()):
        cards = []
        for path in sounds:
            relative = path.relative_to(ROOT)
            href = quote(pathlib.Path(os.path.relpath(path, output.parent)).as_posix())
            name = "/".join((*relative.parts[2:-1], path.stem)).replace("_", " ")
            searchable = f"{category} {name} {relative.as_posix()}".lower()
            credit = RECORDED_PATHS.get(relative.as_posix())
            source = (f'<small>Source: <a href="{html.escape(credit[1], quote=True)}">{html.escape(credit[0])}</a></small>'
                      if credit else '<small>Original project-generated sound</small>')
            review = relative.as_posix() in REVIEW_FILES
            cards.append(
                f'<article class="{"review" if review else "approved"}" data-search="{html.escape(searchable, quote=True)}">'
                f'<strong>{html.escape(name)}</strong><small>{html.escape(relative.as_posix())}</small>{source}'
                f'{"<small>For review - not in game</small>" if review else ""}'
                f'{waveform(path)}<audio controls preload="none" src="{html.escape(href, quote=True)}"></audio></article>'
            )
        sections.append(f'<section><h2>{html.escape(category)}</h2>{"".join(cards)}</section>')
    content = "".join(sections) or '<p class="empty">No audio files found in art/sounds or assets/sounds.</p>'
    document = (
        '<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">'
        '<title>Facility sound catalog</title><style>' + STYLE + '</style><body>'
        '<h1>Facility sound catalog</h1><p>Selected sounds from art/sounds and assets/sounds. Waveforms show peak amplitude, not loudness. '
        'Approved cues have runtime copies in assets/sounds. Yellow cards are review-only sounds, not in the game. '
        'Source-page CC0 labels are recorded as displayed, not independently verified authorship. In-game mix remains provisional.</p>'
        '<label for="filter">Filter sounds</label> <input id="filter" type="search" placeholder="Category or filename">'
        + content + '<script>document.addEventListener("play",e=>{if(e.target.tagName==="AUDIO")for(const a of document.querySelectorAll("audio"))if(a!==e.target)a.pause()},true);'
        'document.querySelector("#filter").addEventListener("input",e=>{const q=e.target.value.toLowerCase();'
        'for(const card of document.querySelectorAll("article")){card.hidden=!card.dataset.search.includes(q)}'
        'for(const group of document.querySelectorAll("section")){group.hidden=![...group.querySelectorAll("article")].some(card=>!card.hidden)}})</script></body></html>'
    )
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(document, encoding="utf-8")
    print(f"{output}: {len(paths)} sounds, {len(groups)} categories")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path, default=OUTPUT)
    args = parser.parse_args()
    build(collect(), args.output.resolve())


if __name__ == "__main__":
    main()
