#!/usr/bin/env python3
import math
import pathlib
import random
from functools import lru_cache
import struct
import wave

from generate_sounds import RATE, ROOT, TAU, noise_table, periodic_noise, resonator

OUT = ROOT / "art/sounds/review/amb"


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


def drip(index, freq, decay, sweep, splash_seed, splash_amount, duration=0.32):
    def signal(t):
        body = 0.55 * resonator(t, 0.0, freq, decay, sweep)
        thump = 0.22 * resonator(t, 0.0, max(55, freq * 0.07), decay * 0.55)
        splash = splash_amount * noise(t, splash_seed, 191, duration) * math.exp(-t / (decay * 0.8))
        return body + thump + splash

    return render(f"drip/{index:02d}", duration, signal)


def pipe_flow():
    duration = 14.7
    bubbles = ((1.4, 420), (3.9, 520), (6.1, 360), (8.8, 610), (11.6, 470))

    def bubble(t, start, pitch):
        elapsed = t - start
        if not 0 <= elapsed < 0.1:
            return 0.0
        return math.sin(TAU * (pitch * elapsed + 180 * elapsed * elapsed)) * math.sin(math.pi * elapsed / 0.1) ** 2

    return render("pipe/flow", duration, lambda t:
        0.11 * noise(t, 820, 331, duration) + 0.05 * noise(t, 821, 1109, duration)
        + sum(0.05 * bubble(t, start, pitch) for start, pitch in bubbles), True)


def vent_rattle():
    ticks = ((0.0, 1180, 0.03), (0.08, 980, 0.028), (0.15, 1340, 0.022), (0.24, 1040, 0.03), (0.33, 1220, 0.02))
    return render("vent/rattle/01", 0.55, lambda t:
        sum(0.5 * resonator(t, start, freq, decay) for start, freq, decay in ticks))


def boiler_fire():
    duration = 9.3
    rng = random.Random(830)
    starts = sorted(rng.uniform(0.4, duration - 0.4) for _ in range(round(duration * 6)))
    clicks = list(zip(starts, noise_table(833, len(starts))))

    def signal(t):
        bed = 0.045 * noise(t, 831, 97, duration) * (1 + 0.2 * noise(t, 832, 23, duration))
        crackle = sum(0.09 * weight * math.exp(-(t - start) * 70)
                      for start, weight in clicks if t >= start)
        return bed + crackle

    return render("boiler/fire", duration, signal, True)


def alarm_pulse():
    duration = math.pi
    cycles = 440
    freq = cycles / duration

    def signal(t):
        factor = 0.55 + 0.45 * math.sin(2 * t)
        tone = 0.5 * math.sin(TAU * freq * t) + 0.18 * math.sin(TAU * freq * 1.5 * t)
        return tone * factor

    return render("alarm/pulse", duration, signal, True)


def lab_console():
    duration = 7.9
    cycles = round(36 * duration)
    freq = cycles / duration

    def signal(t):
        return (0.07 * noise(t, 840, 211, duration) + 0.015 * noise(t, 841, 23, duration)
                + 0.012 * math.sin(TAU * freq * t))

    return render("lab/console", duration, signal, True)


def lab_console_beep():
    return render("lab/console/beep-01", 0.16, lambda t:
        0.6 * resonator(t, 0.01, 520, 0.045) + 0.22 * resonator(t, 0.01, 781, 0.03))


def outside_wind():
    duration = 18.7
    gust_cycles = 3
    gust_freq = gust_cycles / duration

    def signal(t):
        gust = 1.0 + 0.3 * math.sin(TAU * gust_freq * t)
        return (0.1 * noise(t, 850, 131, duration) + 0.045 * noise(t, 851, 719, duration)) * gust

    return render("outside/wind", duration, signal, True)


def low_pressure():
    duration = 10.9
    freq_a = 490 / duration
    freq_b = 512 / duration

    def signal(t):
        return (0.5 * math.sin(TAU * freq_a * t) + 0.5 * math.sin(TAU * freq_b * t)
                + 0.1 * noise(t, 860, 59, duration))

    return render("low/pressure", duration, signal, True)


def main():
    for make in (roomtone_conduit, boiler_tick, vent_hvac, light_buzz,
                 light_buzz_low, light_flicker, tank_hum, distant_settle,
                 lambda: drip(1, 1820, 0.045, -900, 870, 0.09),
                 lambda: drip(2, 980, 0.085, -280, 871, 0.07),
                 lambda: drip(3, 2420, 0.03, -1500, 872, 0.05),
                 lambda: drip(4, 1320, 0.065, -520, 873, 0.12),
                 pipe_flow, vent_rattle, boiler_fire, alarm_pulse,
                 lab_console, lab_console_beep, outside_wind, low_pressure):
        print(make().relative_to(ROOT))


if __name__ == "__main__":
    main()
