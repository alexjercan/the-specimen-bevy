#!/usr/bin/env python3
import math
import pathlib
import struct
import wave

RATE = 48_000
ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "art/sounds/generated"
TAU = 2 * math.pi


def exp_decay(t, start, decay):
    return math.exp(-(t - start) / decay) if t >= start else 0.0


def tone(t, start, frequency, decay, phase=0.0):
    return exp_decay(t, start, decay) * math.sin(TAU * frequency * (t - start) + phase)


def soft_blip(t, start, frequency, decay):
    return tone(t, start, frequency, decay) + 0.15 * tone(t, start, frequency * 3.0, decay * 0.6)


def render(name, duration, signal, peak):
    count = round(duration * RATE)
    samples = [signal(i / RATE) for i in range(count)]
    measured = max((abs(sample) for sample in samples), default=0.0)
    if measured > 0:
        samples = [max(-1.0, min(1.0, sample * peak / measured)) for sample in samples]
    path = OUT / f"{name}.wav"
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(RATE)
        audio.writeframes(struct.pack(f"<{count}h", *(round(sample * 32767) for sample in samples)))
    return path


def ui_fuse_slot(index):
    frequency = 560 + index * 110

    def signal(t):
        return soft_blip(t, 0.0, frequency, 0.045)

    return render(f"ui/fuse/slot-{index}", 0.28, signal, peak=0.14)


def ui_fuse_complete():

    def signal(t):
        return (
            soft_blip(t, 0.0, 523.25, 0.09)
            + 0.8 * soft_blip(t, 0.075, 659.25, 0.16)
        )

    return render("ui/fuse/complete", 1.0, signal, peak=0.16)


def main():
    paths = [
        ui_fuse_slot(1),
        ui_fuse_slot(2),
        ui_fuse_slot(3),
        ui_fuse_complete(),
    ]
    for path in paths:
        print(path.relative_to(ROOT))


if __name__ == "__main__":
    main()
