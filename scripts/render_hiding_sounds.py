#!/usr/bin/env python3
import hashlib
import pathlib
import subprocess
import tempfile
import urllib.request
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "art/sounds/generated/hiding"
SOURCE_URL = "https://opengameart.org/sites/default/files/100-CC0-wood-metal-SFX.zip"
SOURCE_SHA256 = "be6eba63b03409ac0c77787a956b1503a7c186403d04aef9725c52644a4b7878"

CUES = (
    ("locker/open.wav", ("metal_open_01.ogg",), "highpass=f=85,lowpass=f=4100,volume=0.38,afade=t=in:st=0:d=0.012,afade=t=out:st=0.94:d=0.16"),
    ("locker/close.wav", ("metal_close_01.ogg",), "highpass=f=85,lowpass=f=4100,volume=0.44,afade=t=in:st=0:d=0.012,afade=t=out:st=0.73:d=0.13"),
    ("table/enter.wav", ("wood_squeak_01.ogg", "wood_misc_01.ogg"), "[0:a]highpass=f=115,lowpass=f=3200,volume=0.42[a];[1:a]highpass=f=120,lowpass=f=2800,volume=0.22,adelay=110:all=1[b];[a][b]amix=inputs=2:duration=longest:normalize=0[out]"),
    ("table/leave.wav", ("wood_misc_02.ogg", "wood_close_01.ogg"), "[0:a]highpass=f=115,lowpass=f=3000,volume=0.32[a];[1:a]highpass=f=100,lowpass=f=3000,volume=0.26,adelay=170:all=1[b];[a][b]amix=inputs=2:duration=longest:normalize=0[out]"),
)


def render(output=OUTPUT):
    request = urllib.request.Request(SOURCE_URL, headers={"User-Agent": "horror-game-bevy/audio-review"})
    with tempfile.TemporaryDirectory() as folder:
        folder = pathlib.Path(folder)
        archive_path = folder / "source.zip"
        with urllib.request.urlopen(request, timeout=30) as response:
            archive_path.write_bytes(response.read())
        if hashlib.sha256(archive_path.read_bytes()).hexdigest() != SOURCE_SHA256:
            raise ValueError("door/wood source archive SHA-256 mismatch")
        with zipfile.ZipFile(archive_path) as archive:
            names = {name for _, sources, _ in CUES for name in sources}
            for name in names:
                (folder / name).write_bytes(archive.read(name))
        for name, sources, filters in CUES:
            destination = output / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            command = ["ffmpeg", "-nostdin", "-v", "error", "-y"]
            for source in sources:
                command.extend(("-i", str(folder / source)))
            if len(sources) == 1:
                command.extend(("-af", filters))
            else:
                command.extend(("-filter_complex", filters, "-map", "[out]"))
            command.extend(("-ac", "1", "-ar", "48000", "-c:a", "pcm_s16le", str(destination)))
            subprocess.run(command, check=True, timeout=30)
            print(destination)


if __name__ == "__main__":
    render()
