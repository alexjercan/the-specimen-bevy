#!/usr/bin/env python3
import array
import pathlib
import subprocess
import sys
import wave

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = ROOT / "art/sounds/sources/freesound/monster"
OUTPUT = ROOT / "art/sounds/generated/monster"
RATE = 48_000


def decode(path):
    result = subprocess.run(
        ["ffmpeg", "-nostdin", "-v", "error", "-i", str(path),
         "-ac", "1", "-ar", str(RATE), "-f", "f32le", "pipe:1"],
        capture_output=True, check=True,
    )
    samples = array.array("f")
    samples.frombytes(result.stdout)
    if sys.byteorder != "little":
        samples.byteswap()
    return samples


def excerpt(samples, start, end, fade=0.02):
    result = array.array("f", samples[round(start * RATE):round(end * RATE)])
    if len(result) != round((end - start) * RATE):
        raise ValueError("source is too short for excerpt")
    width = round(fade * RATE)
    if not 0 < 2 * width < len(result):
        raise ValueError("invalid fade length")
    for index in range(width):
        result[index] *= index / width
        result[-index - 1] *= index / width
    return result


def loop_excerpt(samples, start, duration, overlap):
    start = round(start * RATE)
    length = round(duration * RATE)
    fade = round(overlap * RATE)
    window = samples[start:start + length + fade]
    if len(window) != length + fade or not 0 < fade < length:
        raise ValueError("source is too short for loop")
    result = array.array("f", window[fade:length])
    for index in range(fade):
        blend = index / (fade - 1)
        result.append(window[length + index] * (1 - blend) + window[index] * blend)
    return result


def save(path, samples):
    path.parent.mkdir(parents=True, exist_ok=True)
    pcm = array.array("h", (max(-32768, min(32767, round(sample * 32767))) for sample in samples))
    if sys.byteorder != "little":
        pcm.byteswap()
    with wave.open(str(path), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(RATE)
        output.writeframes(pcm.tobytes())


def generate(output=OUTPUT):
    roar = decode(SOURCE / "attack/growl-roar-jofae.ogg")
    heartbeat = decode(SOURCE / "chase/heartbeat-under-the-hood.ogg")
    crawl = decode(SOURCE / "vent/crawl-ultra-edward.ogg")
    rendered = {
        "detected/growl-jofae.wav": excerpt(roar, 0.04, 2.24, 0.035),
        "attack/roar-jofae.wav": excerpt(roar, 3.12, 6.16, 0.035),
        "chase/heartbeat-under-the-hood.wav": loop_excerpt(heartbeat, 0.0, 8.0, 0.3),
        "step/crawl-a.wav": excerpt(crawl, 0.08, 1.12),
        "step/crawl-b.wav": excerpt(crawl, 1.72, 2.65),
        "step/crawl-c.wav": excerpt(crawl, 3.05, 4.18),
        "step/crawl-d.wav": excerpt(crawl, 4.92, 5.92),
        "step/crawl-e.wav": excerpt(crawl, 8.40, 9.49),
        "step/crawl-f.wav": excerpt(crawl, 10.01, 10.91),
    }
    for name, samples in rendered.items():
        save(output / name, samples)
    return [output / name for name in rendered]


if __name__ == "__main__":
    generate()
