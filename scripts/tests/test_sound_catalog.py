import pathlib
import re
import sys
import tempfile
import unittest
import wave

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import build_sound_catalog as catalog
import generate_sounds as synth
import generate_ambience_review as ambience


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

    def test_ambience_review_generation(self):
        with tempfile.TemporaryDirectory() as folder:
            old_out = ambience.OUT
            ambience.OUT = pathlib.Path(folder) / "art/sounds/review/amb"
            try:
                makers = (ambience.roomtone_conduit, ambience.boiler_tick,
                          ambience.vent_hvac, ambience.light_buzz,
                          ambience.light_buzz_low, ambience.light_flicker,
                          ambience.tank_hum, ambience.distant_settle)
                for make in makers:
                    path = make()
                    with wave.open(str(path)) as audio:
                        self.assertEqual(audio.getframerate(), 48_000)
                        self.assertEqual(audio.getnchannels(), 1)
                        self.assertGreater(audio.getnframes(), 0)
                    self.assertIn("<svg", catalog.waveform(path))
                    original = path.read_bytes()
                    make()
                    self.assertEqual(original, path.read_bytes())
            finally:
                ambience.OUT = old_out

    def test_catalog_only_lists_approved_and_review_audio(self):
        selected = {path.relative_to(catalog.ROOT).as_posix() for path in catalog.collect()}
        self.assertEqual(selected, catalog.APPROVED_FILES | catalog.REVIEW_FILES)
        self.assertEqual(
            {path for path in selected if "/step/" in path},
            {f"art/sounds/source/step/subway/{n:02d}.ogg" for n in (1, 2, 4)},
        )
        self.assertEqual(len([path for path in selected if "/door/" in path]), 3)
        hiding = {path for path in selected if "/hiding/" in path}
        self.assertEqual(len(hiding), 4)
        for path in hiding:
            self.assertIn("<svg", catalog.waveform(catalog.ROOT / path))
        all_audio = {path.relative_to(catalog.ROOT).as_posix()
                     for path in (catalog.ROOT / "art/sounds").rglob("*")
                     if path.is_file() and path.suffix.lower() in catalog.EXTENSIONS}
        self.assertEqual(all_audio, catalog.APPROVED_FILES | catalog.REVIEW_FILES | catalog.DRAFT_FILES)
        self.assertTrue(catalog.DRAFT_FILES.isdisjoint(selected))

    def test_catalog_credits_each_file_separately(self):
        with tempfile.TemporaryDirectory() as folder:
            output = pathlib.Path(folder) / "catalog.html"
            catalog.build(catalog.collect(), output)
            page = output.read_text()
            cards = re.findall(r"<article .*?</article>", page)
            self.assertEqual(len(cards), len(catalog.APPROVED_FILES | catalog.REVIEW_FILES))
            for path in catalog.APPROVED_FILES:
                matching = [card for card in cards if f"<small>{path}</small>" in card]
                self.assertEqual(len(matching), 1, path)
                card = matching[0]
                if path.startswith("art/sounds/source/step/"):
                    self.assertIn("GboxMikeFozzy", card)
                    self.assertNotIn("rubberduck", card)
                elif "/door/" in path or "/hiding/" in path:
                    self.assertIn("rubberduck", card)
                    self.assertNotIn("GboxMikeFozzy", card)
                else:
                    self.assertIn("Original project-generated sound", card)
                    self.assertNotIn("GboxMikeFozzy", card)
                    self.assertNotIn("rubberduck", card)
            for path in catalog.REVIEW_FILES:
                matching = [card for card in cards if f"<small>{path}</small>" in card]
                self.assertEqual(len(matching), 1, path)
                self.assertIn('class="review"', matching[0])
                self.assertIn("For review - not in game", matching[0])
                if path.endswith("light/flicker/recorded-01.ogg"):
                    self.assertIn("mmaruska", matching[0])
                    self.assertIn("Freesound preview", matching[0])
                    self.assertNotIn("Original project-generated sound", matching[0])
                else:
                    self.assertIn("Original project-generated sound", matching[0])
                    self.assertNotIn("mmaruska", matching[0])
                self.assertNotIn("GboxMikeFozzy", matching[0])
            self.assertIn("<h2>D. Facility ambience (bus ambience)</h2>", page)
            self.assertIn("<h2>G. Hiding (bus world)</h2>", page)
            self.assertEqual(page.count('class="review"'), len(catalog.REVIEW_FILES))

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
            self.assertIn("<h2>B. Doors (bus world)</h2>", page)
            self.assertIn("unlatch &amp; click", page)
            self.assertIn("unlatch%20%26%20click.ogg", page)
            self.assertIn("Cannot decode waveform", page)
            self.assertIn('Original project-generated sound', page)


if __name__ == "__main__":
    unittest.main()
