#!/usr/bin/env python3
import math
import pathlib
import random
import struct
import wave

RATE = 48_000
ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "art/sounds/generated/candidates/device"


def render(duration, seed, tones=(), clicks=(), burst=None):
    rng = random.Random(seed)
    count = round(duration * RATE)
    samples = []
    for index in range(count):
        t = index / RATE
        value = 0.0
        for start, length, frequency, volume in tones:
            age = t - start
            if 0 <= age < length:
                envelope = min(1.0, age / 0.005) * min(1.0, (length - age) / 0.025)
                value += volume * envelope * math.sin(2 * math.pi * frequency * age)
        for start, volume in clicks:
            age = t - start
            if 0 <= age < 0.05:
                value += volume * math.exp(-age * 105) * rng.uniform(-1.0, 1.0)
        if burst is not None:
            start, length, volume = burst
            age = t - start
            if 0 <= age < length:
                envelope = min(1.0, age / 0.012) * math.exp(-age * 9)
                value += volume * envelope * (
                    0.6 * rng.uniform(-1.0, 1.0)
                    + 0.4 * math.sin(2 * math.pi * (115 - 65 * age) * age)
                )
        samples.append(value)
    peak = max((abs(sample) for sample in samples), default=0.0)
    gain = min(1.0, 0.45 / peak) if peak else 1.0
    return struct.pack(f"<{count}h", *(round(max(-1.0, min(1.0, sample * gain)) * 32767) for sample in samples))


def flashbang_pickup():
    rng = random.Random(17)
    count = round(0.32 * RATE)
    samples = []
    for index in range(count):
        t = index / RATE
        value = 0.0
        for start, amplitude in ((0.015, 0.28), (0.095, 0.34)):
            age = t - start
            if 0 <= age < 0.055:
                value += amplitude * math.exp(-age * 90) * rng.uniform(-1.0, 1.0)
            if 0 <= age < 0.18:
                value += 0.09 * math.exp(-age * 28) * (
                    math.sin(2 * math.pi * 510 * age)
                    + 0.35 * math.sin(2 * math.pi * 1280 * age)
                )
        samples.append(max(-1.0, min(1.0, value)))
    return struct.pack(f"<{count}h", *(round(sample * 32767) for sample in samples))


def generate(output=OUT):
    cues = {
        "detector/pickup": (0.24, 11, ((0.035, 0.08, 540, 0.14), (0.13, 0.075, 810, 0.12)), ((0.0, 0.13),), None),
        "detector/nearby": (0.38, 13, ((0.02, 0.10, 630, 0.10), (0.20, 0.095, 710, 0.08)), (), None),
        "flashbang/pickup": (0.32, 17, (), (), None),
        "flashbang/throw": (0.34, 19, (), ((0.01, 0.18), (0.07, 0.10)), None),
        "flashbang/burst": (1.0, 23, ((0.09, 0.6, 840, 0.025),), (), (0.03, 0.72, 0.33)),
    }
    paths = []
    for name, (duration, seed, tones, clicks, burst) in cues.items():
        path = output / f"{name}.wav"
        path.parent.mkdir(parents=True, exist_ok=True)
        with wave.open(str(path), "wb") as audio:
            audio.setnchannels(1)
            audio.setsampwidth(2)
            audio.setframerate(RATE)
            audio.writeframes(flashbang_pickup() if name == "flashbang/pickup" else render(duration, seed, tones, clicks, burst))
        paths.append(path)
    return paths


if __name__ == "__main__":
    for path in generate():
        print(path.relative_to(ROOT))
