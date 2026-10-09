#!/usr/bin/env python3
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = ROOT / "art/sounds/sources/freesound/device/flashbang/throw-cloth-avreliy.ogg"
OUTPUT = ROOT / "art/sounds/generated/device/flashbang"
CUTS = {"throw": (0.0, 0.40), "pickup": (0.85, 1.28236)}


def generate(output=OUTPUT):
    output.mkdir(parents=True, exist_ok=True)
    paths = []
    for name, (start, end) in CUTS.items():
        path = output / f"{name}.wav"
        duration = end - start
        subprocess.run(
            ["ffmpeg", "-nostdin", "-v", "error", "-y", "-i", str(SOURCE),
             "-af", f"atrim=start={start}:end={end},asetpts=PTS-STARTPTS,afade=t=in:st=0:d=0.005,afade=t=out:st={duration - 0.035}:d=0.035",
             "-ac", "1", "-ar", "48000", "-c:a", "pcm_s16le", str(path)],
            check=True,
        )
        paths.append(path)
    return paths


if __name__ == "__main__":
    for path in generate():
        print(path.relative_to(ROOT))
