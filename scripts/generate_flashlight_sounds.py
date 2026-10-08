#!/usr/bin/env python3
import math
import pathlib
import random
import struct
import wave

RATE = 48_000
ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "art/sounds/generated/candidates/flashlight"


def click(duration, events, seed, peak=0.22):
    rng = random.Random(seed)
    count = round(duration * RATE)
    samples = [0.0] * count
    for start, weight, body_hz, decay in events:
        low = 0.0
        previous = 0.0
        origin = round(start * RATE)
        for index in range(origin, min(count, origin + round(0.072 * RATE))):
            elapsed = (index - origin) / RATE
            noise = rng.uniform(-1.0, 1.0)
            low += 0.16 * (noise - low)
            texture = low - previous
            previous = low
            attack = min(1.0, elapsed / 0.0012)
            grit = texture * math.exp(-elapsed / 0.008)
            body = math.sin(2 * math.pi * body_hz * elapsed) * math.exp(-elapsed / decay)
            samples[index] += weight * attack * (0.85 * grit + 0.2 * body)
    maximum = max((abs(sample) for sample in samples), default=0.0)
    if maximum:
        gain = peak / maximum
        samples = [sample * gain for sample in samples]
    fade = min(240, count)
    for index in range(fade):
        samples[count - fade + index] *= (fade - index - 1) / fade
    return struct.pack(f"<{count}h", *(round(sample * 32767) for sample in samples))


def generate(output=OUT):
    cues = {
        "switch-on": (0.16, [(0.008, 1.0, 720, 0.014), (0.035, 0.35, 370, 0.018)], 137),
        "switch-off": (0.15, [(0.008, 0.75, 440, 0.012), (0.024, 0.62, 250, 0.021)], 211),
        "battery-empty": (0.27, [(0.008, 0.8, 240, 0.016), (0.105, 0.38, 190, 0.013)], 307),
    }
    paths = []
    for name, (duration, events, seed) in cues.items():
        path = output / f"{name}.wav"
        path.parent.mkdir(parents=True, exist_ok=True)
        with wave.open(str(path), "wb") as audio:
            audio.setnchannels(1)
            audio.setsampwidth(2)
            audio.setframerate(RATE)
            audio.writeframes(click(duration, events, seed))
        paths.append(path)
    return paths


if __name__ == "__main__":
    for path in generate():
        print(path.relative_to(ROOT))
