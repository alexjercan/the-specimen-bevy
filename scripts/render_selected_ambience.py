#!/usr/bin/env python3
import array
import pathlib
import subprocess
import sys
import wave

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = ROOT / "art/sounds/sources/freesound/amb"
OUTPUT = ROOT / "art/sounds/generated/amb"
RATE = 48_000


def decode(path, filters):
    result = subprocess.run(
        ["ffmpeg", "-nostdin", "-v", "error", "-i", str(path),
         "-af", filters, "-ac", "1", "-ar", str(RATE), "-f", "f32le", "pipe:1"],
        capture_output=True, check=True,
    )
    samples = array.array("f")
    samples.frombytes(result.stdout)
    if sys.byteorder != "little":
        samples.byteswap()
    return samples


def loop_excerpt(samples, start, duration, overlap):
    start = round(start * RATE)
    size = round(duration * RATE)
    fade = round(overlap * RATE)
    window = samples[start:start + size + fade]
    if len(window) != size + fade or not 0 < fade < size:
        raise ValueError("source is too short for the requested loop")
    result = array.array("f", window[fade:size])
    for index in range(fade):
        blend = index / (fade - 1)
        result.append(window[size + index] * (1 - blend) + window[index] * blend)
    return result


def one_shot(samples, start, duration):
    start = round(start * RATE)
    size = round(duration * RATE)
    result = array.array("f", samples[start:start + size])
    if len(result) != size:
        raise ValueError("source is too short for the requested one-shot")
    fade_in = round(0.08 * RATE)
    fade_out = round(0.3 * RATE)
    for index in range(fade_in):
        result[index] *= index / fade_in
    for index in range(fade_out):
        result[-index - 1] *= index / fade_out
    return result


def save(path, samples):
    path.parent.mkdir(parents=True, exist_ok=True)
    pcm = array.array("h", (max(-32768, min(32767, round(value * 32767))) for value in samples))
    if sys.byteorder != "little":
        pcm.byteswap()
    with wave.open(str(path), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(RATE)
        output.writeframes(pcm.tobytes())


def main():
    furnace = decode(SOURCE / "furnace/furnace-iankath.ogg", "highpass=f=40,lowpass=f=2500,volume=0.75")
    faucet = decode(SOURCE / "water/faucet-willstepp.ogg", "highpass=f=120,lowpass=f=4500,volume=0.7")
    wind = decode(SOURCE / "vent/wind-dblover.ogg", "highpass=f=75,lowpass=f=1700,volume=0.55")
    save(OUTPUT / "furnace/burning.wav", loop_excerpt(furnace, 8, 24, 1.5))
    save(OUTPUT / "water/faucet.wav", one_shot(faucet, 8, 5))
    save(OUTPUT / "vent/wind.wav", loop_excerpt(wind, 12, 18, 1.5))


if __name__ == "__main__":
    main()
