#!/usr/bin/env python3
import math
import pathlib
import random
import struct
import wave

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "art/sounds/generated/candidates/device/flashbang/burst-original.wav"
RATE = 48000
DURATION = 2.8


def generate(output=OUTPUT):
    rng = random.Random(44721)
    samples = []
    low = 0.0
    smear = 0.0
    phase = 0.0
    impact_phase = 0.0
    for index in range(round(RATE * DURATION)):
        t = index / RATE
        noise = rng.uniform(-1.0, 1.0)
        low += 0.022 * (noise - low)
        smear += 0.003 * (noise - smear)
        snap = noise * math.exp(-t * 140) * 0.8
        impact_phase += 2 * math.pi * (95 + 85 * math.exp(-t * 65)) / RATE
        thump = math.sin(impact_phase) * math.exp(-t * 32) * 1.6
        thump += low * math.exp(-t * 38) * 3.2
        body = low * math.exp(-t * 8) * 2.2
        air = (noise - low) * math.exp(-t * 10) * 0.27
        tail = smear * math.exp(-t * 1.9) * 1.3
        phase += 2 * math.pi * (3150 - 360 * min(t / 0.8, 1.0)) / RATE
        ring = math.sin(phase) * (1 - math.exp(-t * 18)) * math.exp(-t * 1.25) * 0.12
        sample = snap + thump + body + air + tail + ring
        samples.append(math.tanh(sample * 1.5) * 0.75)
    peak = max(abs(sample) for sample in samples)
    output = pathlib.Path(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(output), "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(RATE)
        audio.writeframes(b"".join(struct.pack("<h", round(sample * 29000 / peak)) for sample in samples))
    return output


if __name__ == "__main__":
    print(generate())
