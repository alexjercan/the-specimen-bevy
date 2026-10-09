import pathlib
import sys
import tempfile
import unittest
import wave

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import generate_flashbang_burst as burst


class FlashbangBurstTests(unittest.TestCase):
    def test_original_render_is_deterministic_mono_pcm(self):
        with tempfile.TemporaryDirectory() as folder:
            first = burst.generate(pathlib.Path(folder) / "first.wav")
            second = burst.generate(pathlib.Path(folder) / "second.wav")
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with wave.open(str(first), "rb") as audio:
                self.assertEqual(audio.getnchannels(), 1)
                self.assertEqual(audio.getsampwidth(), 2)
                self.assertEqual(audio.getframerate(), 48000)
                self.assertEqual(audio.getnframes(), 134400)


if __name__ == "__main__":
    unittest.main()
