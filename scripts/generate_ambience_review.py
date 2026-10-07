#!/usr/bin/env python3
import math
import pathlib
import struct
import wave

from generate_sounds import RATE, ROOT, TAU, periodic_noise

OUT = ROOT / "art/sounds/review/amb"


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
    return render("boiler/tick/01", 1.2, lambda t:
        sum(weight * math.exp(-max(0, t - start) * 6) * math.sin(TAU * freq * max(0, t - start))
            for start, freq, weight in ((0.04, 360, 0.6), (0.04, 671, 0.21), (0.13, 420, 0.23)))
        * (1 if t >= 0.04 else 0))


def vent_hvac():
    return render("vent/hvac/01", 6, lambda t:
        (0.16 * periodic_noise(t, 802, 1423) + 0.08 * periodic_noise(t, 803, 197))
        * (1 + 0.18 * math.sin(TAU * 26 * t))
        + 0.018 * math.sin(TAU * 78 * t), True)


def light_buzz():
    return render("light/buzz/cool", 6, lambda t:
        (0.052 * math.sin(TAU * 120 * t) + 0.015 * math.sin(TAU * 240 * t))
        * (1 + 0.13 * math.sin(TAU * t / 6))
        + 0.006 * periodic_noise(t, 804, 899), True)


def light_buzz_low():
    return render("light/buzz/cool-low", 6, lambda t:
        (0.052 * math.sin(TAU * 72 * t) + 0.008 * math.sin(TAU * 144 * t))
        * (1 + 0.13 * math.sin(TAU * t / 6))
        + 0.002 * periodic_noise(t, 804, 899), True)


def light_flicker():
    return render("light/flicker/01", 0.32, lambda t:
        (0.7 * periodic_noise(t, 805, 9701) + 0.25 * math.sin(TAU * 930 * t))
        * sum(math.exp(-(t - start) * 48) if t >= start else 0 for start in (0.02, 0.09, 0.145)))


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


def distant_settle():
    return render("distant/settle/01", 2.0, lambda t:
        (0.6 * math.sin(TAU * (83 * t - 5 * t * t)) + 0.12 * periodic_noise(t, 807, 753))
        * (math.sin(math.pi * max(0, t - 0.15) / 1.5) ** 2 if 0.15 < t < 1.65 else 0))


def main():
    for make in (roomtone_conduit, boiler_tick, vent_hvac, light_buzz,
                 light_buzz_low, light_flicker, tank_hum, distant_settle):
        print(make().relative_to(ROOT))


if __name__ == "__main__":
    main()
