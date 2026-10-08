#!/usr/bin/env python3
import math
import pathlib
import struct
import wave
from functools import lru_cache

from generate_sounds import RATE, ROOT, TAU, noise_table, periodic_noise

OUT = ROOT / "art/sounds/generated/amb"


@lru_cache(maxsize=None)
def cached_noise(seed, bins):
    return noise_table(seed, bins)


def noise(t, seed, bins, period):
    table = cached_noise(seed, bins)
    position = (t % period) * bins / period
    index = int(position)
    fraction = position - index
    blend = fraction * fraction * (3 - 2 * fraction)
    return table[index] + (table[(index + 1) % bins] - table[index]) * blend


def render(name, duration, signal, loop=False):
    count = round(duration * RATE)
    samples = []
    for index in range(count):
        time = index / RATE
        edge = 1.0 if loop else min(1.0, time / 0.015, (duration - time) / 0.12)
        samples.append(signal(time) * max(0.0, edge))
    peak = max(abs(sample) for sample in samples)
    gain = (0.07 if loop else 0.12) / peak if peak else 0.0
    path = OUT / (name + ".wav")
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(RATE)
        audio.writeframes(struct.pack(f"<{count}h", *(round(sample * gain * 32767) for sample in samples)))
    return path


def roomtone_conduit():
    return render("roomtone/conduit", 6, lambda t:
        0.20 * periodic_noise(t, 801, 131)
        + 0.045 * math.sin(TAU * 60 * t) * (1 + 0.15 * math.sin(TAU * t / 6)), True)


def boiler_tick():
    return render("boiler/tick", 1.2, lambda t:
        sum(weight * math.exp(-max(0, t - start) * 6) * math.sin(TAU * freq * max(0, t - start))
            for start, freq, weight in ((0.04, 360, 0.6), (0.04, 671, 0.21), (0.13, 420, 0.23)))
        * (1 if t >= 0.04 else 0))


def vent_hvac():
    return render("vent/hvac", 6, lambda t:
        (0.16 * periodic_noise(t, 802, 1423) + 0.08 * periodic_noise(t, 803, 197))
        * (1 + 0.18 * math.sin(TAU * 26 * t))
        + 0.018 * math.sin(TAU * 78 * t), True)


def light_buzz_low():
    return render("light/cool-buzz", 6, lambda t:
        (0.052 * math.sin(TAU * 72 * t) + 0.008 * math.sin(TAU * 144 * t))
        * (1 + 0.13 * math.sin(TAU * t / 6))
        + 0.002 * periodic_noise(t, 804, 899), True)


def tank_hum():
    def bubble(t, start, pitch):
        elapsed = t - start
        if not 0 <= elapsed < 0.09:
            return 0.0
        return math.sin(TAU * (pitch * elapsed + 200 * elapsed * elapsed)) * math.sin(math.pi * elapsed / 0.09) ** 2

    return render("tank/hum", 6, lambda t:
        0.11 * periodic_noise(t, 806, 119)
        + 0.045 * math.sin(TAU * 84 * t)
        + sum(0.038 * bubble(t, start, pitch) for start, pitch in
              ((0.7, 360), (1.9, 620), (2.8, 460), (4.2, 550), (5.1, 420))), True)


def low_pressure():
    duration = 10.9
    freq_a = 490 / duration
    freq_b = 512 / duration

    def signal(t):
        return (0.5 * math.sin(TAU * freq_a * t) + 0.5 * math.sin(TAU * freq_b * t)
                + 0.1 * noise(t, 860, 59, duration))

    return render("pressure/low", duration, signal, True)


def main():
    for make in (roomtone_conduit, boiler_tick, vent_hvac, light_buzz_low, tank_hum, low_pressure):
        print(make().relative_to(ROOT))


if __name__ == "__main__":
    main()
