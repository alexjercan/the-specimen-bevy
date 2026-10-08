import array
import math
import pathlib
import sys
import tempfile
import unittest
import wave

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import render_selected_ambience as render


class SelectedAmbienceTests(unittest.TestCase):
    def test_loop_excerpt_is_deterministic_and_wraps_smoothly(self):
        samples = array.array("f", (
            0.4 * math.sin(2 * math.pi * 100 * index / render.RATE)
            for index in range(3 * render.RATE)
        ))
        first = render.loop_excerpt(samples, 0.5, 1.5, 0.2)
        self.assertEqual(first, render.loop_excerpt(samples, 0.5, 1.5, 0.2))
        self.assertEqual(len(first), round(1.5 * render.RATE))
        self.assertLess(abs(first[0] - first[-1]), 0.01)

    def test_burst_fades_and_writes_mono_pcm(self):
        source = array.array("f", [0.25] * (2 * render.RATE))
        burst = render.one_shot(source, 0.5, 1)
        self.assertEqual(burst[0], 0)
        self.assertEqual(burst[-1], 0)
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "burst.wav"
            render.save(path, burst)
            with wave.open(str(path)) as audio:
                self.assertEqual(audio.getnchannels(), 1)
                self.assertEqual(audio.getframerate(), render.RATE)
                self.assertEqual(audio.getnframes(), render.RATE)

    def test_short_sources_are_rejected(self):
        samples = array.array("f", [0] * render.RATE)
        with self.assertRaises(ValueError):
            render.loop_excerpt(samples, 0, 2, 0.2)
        with self.assertRaises(ValueError):
            render.one_shot(samples, 0, 2)


if __name__ == "__main__":
    unittest.main()
