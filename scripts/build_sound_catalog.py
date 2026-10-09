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
    "art/sounds/sources/freesound/flashlight/click-ralph0o7.ogg",
    "art/sounds/sources/opengameart/self/breathing-tired-mikeask.wav",
    "art/sounds/sources/opengameart/amb/boiler/switch-off-cleytonkauffman.wav",
    *{f"art/sounds/sources/opengameart/step/subway/subway-step-{letter}.ogg" for letter in "abc"},
    *{f"art/sounds/generated/door/{name}.wav" for name in (
        "unlatch", "swing-open", "shut", "locked-rattle",
    )},
    *{f"art/sounds/generated/hiding/{name}.wav" for name in (
        "locker/open", "locker/close", "table/enter", "table/leave",
    )},
    *{f"art/sounds/generated/amb/{name}.wav" for name in (
        "roomtone/plain", "boiler/rumble", "roomtone/conduit", "boiler/tick",
        "vent/hvac", "tank/hum", "light/cool-buzz", "pressure/low",
        "boiler/power-down", "boiler/reset", "boiler/restart",
    )},
    *{f"art/sounds/generated/ui/fuse/{cue}.wav" for cue in (
        "slot-1", "slot-2", "slot-3", "complete",
    )},
    *{f"art/sounds/generated/ui/{cue}.wav" for cue in (
        "back", "confirm", "denied", "focus", "hover", "pause", "press", "resume"
    )},
    *{f"art/sounds/generated/amb/{name}.wav" for name in (
        "furnace/burning", "water/faucet", "vent/wind",
    )},
    "art/sounds/sources/freesound/monster/step/metal-footsteps-gristi.ogg",
    *{f"art/sounds/generated/monster/{name}.wav" for name in (
        "detected/growl-jofae", "attack/roar-jofae", "chase/heartbeat-under-the-hood",
        *(f"step/crawl-{letter}" for letter in "abcdef"),
    )},
}
MONSTER_CANDIDATES = {
    "step/metal-footsteps-gristi": ("gristi", 562195, "Monster presence ambience during patrol; not a timed footstep."),
    "detected/growl-jofae": ("Jofae", 366837, "First growl cut before the pause; loaded for later detection behavior."),
    "attack/roar-jofae": ("Jofae", 366837, "Second roar after the pause; loaded for later attack behavior."),
    "chase/heartbeat-under-the-hood": ("under_the_hood", 455440, "Eight-second crossfaded loop loaded for later chase behavior."),
    **{f"step/crawl-{letter}": ("Ultra-Edward", 795872, "Isolated crawl impact used as a timed patrol step.") for letter in "abcdef"},
}
REVIEW_FILES = set()
MONSTER_SOURCES = {
    f"art/sounds/sources/freesound/monster/{name}.ogg" for name in (
        "idle/growls-lori-mortimer", "attack/growl-roar-jofae",
        "detected/spooky-sting-nomiqbomi", "chase/heartbeat-under-the-hood",
        "relief/sigh-marilenatip", "vent/crawl-ultra-edward",
        "vent/ventilation-rattle-heckfricker", "claw/metal-scratch-rubberduck9999",
        "claw/wood-scratching-wigglesworth",
    )
}
SOURCE_FILES = {
    *{f"art/sounds/sources/freesound/amb/{name}.ogg" for name in (
        "water/faucet-willstepp", "furnace/furnace-iankath", "vent/wind-dblover",
    )},
    "art/sounds/sources/freesound/door/locked/rattling-locked-door-shelbyshark.ogg",
    "art/sounds/sources/freesound/door/locked/doorknob-rattle-drfahrts.ogg",
}
CATEGORIES = {
    "step": "A. Player self (bus self)",
    "door": "B. Doors (bus world)",
    "fuse": "C. Objective, pickup and panel (bus world and ui)",
    "panel": "C. Objective, pickup and panel (bus world and ui)",
    "amb": "D. Facility ambience (bus ambience)",
    "ui": "E. UI and front end (bus ui)",
    "hiding": "G. Hiding (bus world)",
    "flashlight": "A. Player self (bus self)",
    "self": "A. Player self (bus self)",
    "monster": "F. Monster and threat cues",
}
STEP_CREDIT = ('GboxMikeFozzy, "Footsteps" (CC0 1.0)', "https://opengameart.org/content/footsteps-0")
METAL_WOOD_CREDIT = ('rubberduck, "100 CC0 metal and wood SFX" (CC0 1.0)', "https://opengameart.org/content/100-cc0-metal-and-wood-sfx")
RECORDED_PATHS = {
    path: STEP_CREDIT if "/opengameart/step/" in path else METAL_WOOD_CREDIT
    for path in APPROVED_FILES
    if "/opengameart/step/" in path or "/generated/door/" in path or "/generated/hiding/" in path
}
for creator, sound_id, edit in (
    ("willstepp", 188293, "water/faucet"),
    ("iankath", 173991, "furnace/burning"),
    ("DBlover", 405601, "vent/wind"),
):
    RECORDED_PATHS[f"art/sounds/generated/amb/{edit}.wav"] = (
        f"{creator} (Freesound preview, page-labeled CC0 1.0; edited by this project)",
        f"https://freesound.org/people/{creator}/sounds/{sound_id}/",
    )
RECORDED_PATHS["art/sounds/sources/opengameart/self/breathing-tired-mikeask.wav"] = (
    'mikeask, "Breathing Tired" (OpenGameArt page-labeled CC0 1.0; original download)',
    "https://opengameart.org/content/breathing-tired",
)
RECORDED_PATHS["art/sounds/sources/opengameart/amb/boiler/switch-off-cleytonkauffman.wav"] = (
    "CleytonKauffman, SFX - Circuit breaker (OpenGameArt page-labeled CC0; source ownership unverified)",
    "https://opengameart.org/content/sfx-circuit-breaker",
)
RECORDED_PATHS["art/sounds/generated/door/locked-rattle.wav"] = (
    "DrFahrts (Freesound preview, page-labeled CC0 1.0; edited by this project)",
    "https://freesound.org/people/DrFahrts/sounds/727791/",
)
for name, (creator, sound_id, _) in MONSTER_CANDIDATES.items():
    origin = "sources/freesound" if name == "step/metal-footsteps-gristi" else "generated"
    extension = "ogg" if origin == "sources/freesound" else "wav"
    edit = "" if origin == "sources/freesound" else "; edited by this project"
    RECORDED_PATHS[f"art/sounds/{origin}/monster/{name}.{extension}"] = (
        f"{creator} (Freesound low-quality preview, item page displays CC0 1.0; provenance unverified{edit})",
        f"https://freesound.org/people/{creator}/sounds/{sound_id}/",
    )
for filename, creator, sound_id in (
    ("click-ralph0o7", "Ralph0o7", 690300),
    ("thumb-switch-lunardrive", "Lunardrive", 48979),
    ("spring-switch-eskildnp", "EskildNP", 855455),
):
    RECORDED_PATHS[f"art/sounds/sources/freesound/flashlight/{filename}.ogg"] = (
        f"{creator} (Freesound low-quality preview, page-labeled CC0 1.0)",
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
    if part and part[0] in ("generated", "sources"):
        part = part[1:]
    if part and part[0] in ("opengameart", "freesound", "candidates"):
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
            part = sound_parts(path)
            href = quote(pathlib.Path(os.path.relpath(path, output.parent)).as_posix())
            name = "/".join((*relative.parts[2:-1], path.stem)).replace("_", " ")
            searchable = f"{category} {name} {relative.as_posix()}".lower()
            credit = RECORDED_PATHS.get(relative.as_posix())
            source = (f'<small>Source: <a href="{html.escape(credit[1], quote=True)}">{html.escape(credit[0])}</a></small>'
                      if credit else '<small>Original project-generated sound</small>')
            review = relative.as_posix() in REVIEW_FILES
            note = MONSTER_CANDIDATES.get("/".join(part[1:]).rsplit(".", 1)[0]) if part and part[0] == "monster" else None
            cards.append(
                f'<article class="{"review" if review else "approved"}" data-search="{html.escape(searchable, quote=True)}">'
                f'<strong>{html.escape(name)}</strong><small>{html.escape(relative.as_posix())}</small>{source}'
                f'{"<small>For review - not in game</small>" if review else ""}'
                f'{"<small>Role: " + html.escape(note[2]) + "</small>" if note else ""}'
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
