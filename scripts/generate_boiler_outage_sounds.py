#!/usr/bin/env python3
import math
import pathlib
import struct
import wave

from generate_sounds import RATE, ROOT, TAU, impact, periodic_noise, resonator

OUT = ROOT / "art/sounds/generated/candidates/amb/boiler"


def breaker_trip(t):
    return (0.9 * impact(t, 0.01, 950, 0.012)
            + 0.55 * impact(t, 0.01, 190, 0.03)
            + 0.18 * resonator(t, 0.013, 2600, 0.006))


def power_down(t):
    f0 = 140.0
    tau_f = 0.55
    phase = TAU * f0 * tau_f * (1 - math.exp(-t / tau_f))
    amp = math.exp(-t / 0.9)
    hum = math.sin(phase) + 0.35 * math.sin(2 * phase)
    whir = periodic_noise(t, 451, 211) * amp
    return amp * 0.6 * hum + 0.25 * whir


def reset(t):
    return (0.85 * impact(t, 0.012, 150, 0.028)
            + 0.5 * impact(t, 0.012, 310, 0.02)
            + 0.7 * impact(t, 0.19, 165, 0.026)
            + 0.3 * impact(t, 0.19, 340, 0.018))


def restart(t):
    ticks = sum(impact(t, start, 2200, 0.004) for start in (0.05, 0.14, 0.23, 0.32, 0.41, 0.5))
    whoosh = 0.0
    whoosh_start = 0.55
    if t >= whoosh_start:
        elapsed = t - whoosh_start
        rise = min(1.0, elapsed / 0.5)
        settle = math.exp(-max(0.0, elapsed - 0.9) / 0.35)
        level = settle if elapsed >= 0.9 else 1.0
        whoosh = rise * level * (
            0.5 * periodic_noise(t, 452, 97)
            + 0.25 * math.sin(TAU * 54 * t) * (0.8 + 0.2 * periodic_noise(t, 453, 37))
        )
    return 0.5 * ticks + whoosh


CUES = (
    ("breaker-trip", 0.22, breaker_trip, 0.30),
    ("power-down", 1.4, power_down, 0.22),
    ("reset", 0.42, reset, 0.30),
    ("restart", 2.3, restart, 0.26),
)


def render(name, duration, signal, peak, output):
    count = round(duration * RATE)
    samples = [signal(index / RATE) for index in range(count)]
    fade = min(round(0.01 * RATE), count)
    for index in range(fade):
        samples[index] *= index / fade
        samples[count - 1 - index] *= index / fade
    measured = max((abs(sample) for sample in samples), default=0.0)
    if measured > 0:
        samples = [max(-1.0, min(1.0, sample * peak / measured)) for sample in samples]
    path = output / f"{name}.wav"
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(RATE)
        audio.writeframes(struct.pack(f"<{count}h", *(round(sample * 32767) for sample in samples)))
    return path


def generate(output=OUT):
    return [render(name, duration, signal, peak, output) for name, duration, signal, peak in CUES]


def main():
    for path in generate():
        print(path.relative_to(ROOT))


if __name__ == "__main__":
    main()
