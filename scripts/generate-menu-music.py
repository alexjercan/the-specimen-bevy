#!/usr/bin/env python3
import math
import pathlib
import random
import struct
import subprocess
import wave

RATE = 48_000
DURATION = 16.0
CROSSFADE = 1.5
BASE_HZ = round(55.0 * DURATION) / DURATION
ROOT = pathlib.Path(__file__).resolve().parent.parent
GENERATED_WAV = ROOT / "art/audio/generated/menu/ambient-loop.wav"
GENERATED_OGG = ROOT / "art/audio/generated/menu/ambient-loop.ogg"
PROMOTED_OGG = ROOT / "assets/audio/menu/ambient-loop.ogg"


class Drift:
    def __init__(self, seed, cutoff):
        self.rng = random.Random(seed)
        self.value = 0.0
        self.cutoff = cutoff

    def sample(self):
        self.value += self.cutoff * (self.rng.uniform(-1.0, 1.0) - self.value)
        return self.value


def drone(t):
    # NOTE: every partial is an integer number of cycles over DURATION, so the drone alone is already phase-continuous at the loop point.
    partials = [(1, 0.5), (3, 0.22), (7, 0.14), (11, 0.08)]
    return sum(amplitude * math.sin(2 * math.pi * BASE_HZ * k * t) for k, amplitude in partials)


def render():
    count = round((DURATION + CROSSFADE) * RATE)
    noise = Drift(seed=11, cutoff=0.0006)
    samples = []
    for i in range(count):
        t = i / RATE
        swell = 0.6 + 0.4 * math.sin(2 * math.pi * t / DURATION)
        samples.append(0.5 * drone(t) * swell + 0.12 * noise.sample())
    # NOTE: the noise bed is not periodic over DURATION, so crossfade the rendered tail into the head to keep PlaybackMode::Loop click-free.
    fade_samples = round(CROSSFADE * RATE)
    tail = samples[-fade_samples:]
    head = samples[:fade_samples]
    mixed_head = [
        tail[i] * (1.0 - i / fade_samples) + head[i] * (i / fade_samples) for i in range(fade_samples)
    ]
    looped = mixed_head + samples[fade_samples:-fade_samples]
    peak = max(abs(sample) for sample in looped)
    target = 0.2
    looped = [max(-1.0, min(1.0, sample * target / peak)) for sample in looped]
    GENERATED_WAV.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(GENERATED_WAV), "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(RATE)
        audio.writeframes(
            struct.pack(f"<{len(looped)}h", *(round(sample * 32767) for sample in looped))
        )
    return GENERATED_WAV


def encode_ogg(wav_path):
    GENERATED_OGG.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "ffmpeg", "-nostdin", "-v", "error", "-y",
            "-i", str(wav_path),
            "-c:a", "libvorbis", "-qscale:a", "4",
            str(GENERATED_OGG),
        ],
        check=True,
    )
    return GENERATED_OGG


def promote(ogg_path):
    PROMOTED_OGG.parent.mkdir(parents=True, exist_ok=True)
    PROMOTED_OGG.write_bytes(ogg_path.read_bytes())
    return PROMOTED_OGG


def main():
    wav_path = render()
    ogg_path = encode_ogg(wav_path)
    promoted_path = promote(ogg_path)
    print(f"wrote {wav_path}")
    print(f"wrote {ogg_path}")
    print(f"promoted {promoted_path}")


if __name__ == "__main__":
    main()
