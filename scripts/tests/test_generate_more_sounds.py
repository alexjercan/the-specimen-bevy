import pathlib
import sys
import tempfile
import unittest
import wave

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import generate_more_sounds as synth


class GenerateMoreSoundsTests(unittest.TestCase):
    def test_approved_fuse_cues_are_valid_and_reproducible(self):
        with tempfile.TemporaryDirectory() as folder:
            old_out = synth.OUT
            synth.OUT = pathlib.Path(folder) / "generated"
            try:
                generators = [
                    lambda: synth.ui_fuse_slot(1),
                    lambda: synth.ui_fuse_slot(2),
                    lambda: synth.ui_fuse_slot(3),
                    synth.ui_fuse_complete,
                ]
                for generate in generators:
                    path = generate()
                    with wave.open(str(path)) as audio:
                        self.assertEqual(audio.getframerate(), 48_000)
                        self.assertEqual(audio.getnchannels(), 1)
                        self.assertEqual(audio.getsampwidth(), 2)
                        self.assertGreater(audio.getnframes(), 0)
                    original = path.read_bytes()
                    regenerated = generate()
                    self.assertEqual(path, regenerated)
                    self.assertEqual(original, path.read_bytes())
            finally:
                synth.OUT = old_out

    def test_fuse_cue_paths_are_distinct(self):
        with tempfile.TemporaryDirectory() as folder:
            old_out = synth.OUT
            synth.OUT = pathlib.Path(folder) / "generated"
            try:
                paths = [
                    synth.ui_fuse_slot(1),
                    synth.ui_fuse_slot(2),
                    synth.ui_fuse_slot(3),
                    synth.ui_fuse_complete(),
                ]
                self.assertEqual(len(paths), len(set(paths)))
            finally:
                synth.OUT = old_out


if __name__ == "__main__":
    unittest.main()
