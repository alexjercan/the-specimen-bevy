import pathlib
import sys
import tempfile
import unittest
import wave

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import build_sound_catalog as catalog
import generate_sounds as synth


class SoundCatalogTests(unittest.TestCase):
    def test_generation_and_waveforms(self):
        with tempfile.TemporaryDirectory() as folder:
            old_out = synth.OUT
            synth.OUT = pathlib.Path(folder) / "art/sounds/generated"
            try:
                paths = [synth.pickup(), synth.panel(), synth.ui_hover(),
                         synth.ui_press(), synth.ui_denied(),
                         synth.ui_action("ui.back.01", 612, [(0.012, 200, 0.6), (0.07, 145, 0.4)]),
                         synth.roomtone()]
                for path in paths:
                    with wave.open(str(path)) as audio:
                        self.assertEqual(audio.getframerate(), 48_000)
                        self.assertEqual(audio.getnchannels(), 1)
                        self.assertGreater(audio.getnframes(), 0)
                    self.assertIn("<svg", catalog.waveform(path))
                for path, regenerate in ((paths[0], synth.pickup),
                                         (paths[1], synth.panel),
                                         (paths[2], synth.ui_hover),
                                         (paths[3], synth.ui_press),
                                         (paths[4], synth.ui_denied)):
                    original = path.read_bytes()
                    regenerate()
                    self.assertEqual(original, path.read_bytes())
            finally:
                synth.OUT = old_out

    def test_approved_source_step_waveforms(self):
        for number in (1, 2, 4):
            source = catalog.ROOT / f"art/sounds/source/step/subway/{number:02d}.ogg"
            self.assertIn("<svg", catalog.waveform(source))

    def test_catalog_only_lists_approved_audio(self):
        selected = {path.relative_to(catalog.ROOT).as_posix() for path in catalog.collect()}
        self.assertEqual(selected, catalog.APPROVED_FILES)
        self.assertEqual(
            {path for path in selected if "/step/" in path},
            {f"art/sounds/source/step/subway/{n:02d}.ogg" for n in (1, 2, 4)},
        )
        self.assertEqual(len([path for path in selected if "/door/" in path]), 3)
        all_audio = {path.relative_to(catalog.ROOT).as_posix()
                     for path in (catalog.ROOT / "art/sounds").rglob("*")
                     if path.is_file() and path.suffix.lower() in catalog.EXTENSIONS}
        self.assertEqual(all_audio, catalog.APPROVED_FILES)

    def test_catalog_groups_and_escapes(self):
        with tempfile.TemporaryDirectory() as folder:
            source = pathlib.Path(folder) / "assets/sounds/door"
            source.mkdir(parents=True)
            clip = source / "unlatch & click.ogg"
            clip.write_bytes(b"placeholder")
            output = pathlib.Path(folder) / "art/sounds/catalog.html"
            old_root = catalog.ROOT
            catalog.ROOT = pathlib.Path(folder)
            try:
                catalog.build([clip], output)
            finally:
                catalog.ROOT = old_root
            page = output.read_text()
            self.assertIn("<h2>Doors</h2>", page)
            self.assertIn("unlatch &amp; click", page)
            self.assertIn("unlatch%20%26%20click.ogg", page)
            self.assertIn("Cannot decode waveform", page)


if __name__ == "__main__":
    unittest.main()
