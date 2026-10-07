#!/usr/bin/env python3
import math
import pathlib
import random
import struct
import wave
from functools import lru_cache

RATE = 48_000
ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "art/sounds/generated"
TAU = 2 * math.pi


def envelope(t, duration, attack=0.002, release=0.06):
    return min(1.0, t / attack, max(0.0, (duration - t) / release))


class Texture:
    def __init__(self, seed):
        self.rng = random.Random(seed)
        self.low = 0.0
        self.mid = 0.0

    def sample(self):
        white = self.rng.uniform(-1.0, 1.0)
        self.low += 0.018 * (white - self.low)
        self.mid += 0.17 * (white - self.mid)
        return self.low, self.mid - self.low, white - self.mid


def hit(t, start, decay):
    return math.exp(-(t - start) / decay) if t >= start else 0.0


def stroke(t, start, end):
    if not start <= t < end:
        return 0.0
    phase = (t - start) / (end - start)
    return math.sin(math.pi * phase) ** 2


def render(name, duration, signal, loop=False):
    count = round(duration * RATE)
    samples = []
    for i in range(count):
        t = i / RATE
        value = signal(t)
        if not loop:
            value *= envelope(t, duration)
        samples.append(value)
    peak = max(abs(sample) for sample in samples)
    target = 0.10 if loop else (0.12 if name.startswith("ui.") else 0.30)
    if peak > 0:
        samples = [max(-1.0, min(1.0, sample * target / peak)) for sample in samples]
    path = OUT / (name.replace(".", "/") + ".wav")
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(RATE)
        audio.writeframes(struct.pack(f"<{count}h", *(round(value * 32767) for value in samples)))
    return path


def resonator(t, start, frequency, decay, sweep=0.0):
    elapsed = t - start
    if elapsed < 0:
        return 0.0
    phase = TAU * (frequency * elapsed + sweep * decay * (1 - math.exp(-elapsed / decay)))
    return math.exp(-elapsed / decay) * math.sin(phase)


def impact(t, start, frequency, decay):
    return (resonator(t, start, frequency, decay, frequency * 0.23)
            + 0.26 * resonator(t, start, frequency * 1.73, decay * 0.47))


def pickup():
    texture = Texture(401)

    def signal(t):
        low, _, _ = texture.sample()
        lift = 0.35 * impact(t, 0.035, 235, 0.038)
        ring = 0.38 * resonator(t, 0.065, 940, 0.15)
        ring += 0.12 * resonator(t, 0.065, 1510, 0.09)
        return lift + ring + 0.04 * low * stroke(t, 0.02, 0.09)

    return render("fuse.pickup.01", 0.35, signal)


def panel():
    return render("panel.install.01", 0.34, lambda t: (
        1.0 * impact(t, 0.04, 125, 0.08)
        + 0.38 * impact(t, 0.13, 205, 0.035)
    ))


def ui_hover():
    return render("ui.hover.01", 0.09, lambda t: (
        0.7 * resonator(t, 0.008, 510, 0.023, 65)
        + 0.13 * resonator(t, 0.008, 870, 0.014)
    ))


def ui_press():
    return render("ui.press.01", 0.14, lambda t: (
        0.9 * impact(t, 0.012, 255, 0.037)
        + 0.36 * impact(t, 0.075, 190, 0.023)
    ))


def ui_denied():
    def signal(t):
        first = resonator(t, 0.015, 420, 0.10, -45)
        second = resonator(t, 0.14, 295, 0.16, -35)
        clash = 0.16 * resonator(t, 0.14, 312, 0.095)
        return 0.55 * first + 0.85 * second + clash

    return render("ui.denied.01", 0.43, signal)


def ui_action(name, seed, shape):
    texture = Texture(seed)

    def signal(t):
        low, grit, _ = texture.sample()
        return sum(weight * impact(t, start, frequency, 0.025) for start, frequency, weight in shape) + (
            0.045 * grit + 0.07 * low
        ) * hit(t, 0.015, 0.02)

    return render(name, max(start for start, _, _ in shape) + 0.13, signal)


@lru_cache(maxsize=8)
def noise_table(seed, bins):
    rng = random.Random(seed)
    return [rng.uniform(-1, 1) for _ in range(bins)]


def periodic_noise(t, seed, bins):
    table = noise_table(seed, bins)
    position = t * bins / 6.0
    index = int(position)
    fraction = position - index
    blend = fraction * fraction * (3 - 2 * fraction)
    return table[index] + (table[(index + 1) % bins] - table[index]) * blend


def roomtone():
    return render("amb.roomtone", 6.0, lambda t: (
        0.14 * periodic_noise(t, 601, 159) + 0.07 * periodic_noise(t, 602, 1279)
    ), loop=True)


def boiler():
    return render("amb.boiler.rumble", 6.0, lambda t: (
        0.23 * periodic_noise(t, 701, 97) + 0.08 * periodic_noise(t, 702, 719)
        + 0.06 * math.sin(TAU * 48 * t) * (0.8 + 0.2 * periodic_noise(t, 703, 37))
    ), loop=True)


def main():
    paths = [pickup(), panel(),
             ui_hover(), ui_press(),
             ui_action("ui.focus.01", 611, [(0.012, 185, 0.6)]),
             ui_action("ui.back.01", 612, [(0.012, 200, 0.6), (0.07, 145, 0.4)]),
             ui_action("ui.confirm.01", 613, [(0.012, 220, 0.5), (0.08, 275, 0.45)]),
             ui_denied(),
             ui_action("ui.pause.01", 615, [(0.012, 170, 0.75)]),
             ui_action("ui.resume.01", 616, [(0.012, 190, 0.7), (0.075, 240, 0.3)]),
             roomtone(), boiler()]
    for path in paths:
        print(path.relative_to(ROOT))


if __name__ == "__main__":
    main()
