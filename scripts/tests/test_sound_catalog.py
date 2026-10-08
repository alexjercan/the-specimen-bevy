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
                paths = [synth.ui_hover(), synth.ui_press(), synth.ui_denied(),
                         synth.ui_action("ui.back", 612, [(0.012, 200, 0.6), (0.07, 145, 0.4)]),
                         synth.roomtone()]
                for path in paths:
                    with wave.open(str(path)) as audio:
                        self.assertEqual(audio.getframerate(), 48_000)
                        self.assertEqual(audio.getnchannels(), 1)
                        self.assertGreater(audio.getnframes(), 0)
                    self.assertIn("<svg", catalog.waveform(path))
                for path, regenerate in ((paths[0], synth.ui_hover),
                                         (paths[1], synth.ui_press),
                                         (paths[2], synth.ui_denied)):
                    original = path.read_bytes()
                    regenerate()
                    self.assertEqual(original, path.read_bytes())
            finally:
                synth.OUT = old_out

    def test_approved_source_step_waveforms(self):
        for number in (1, 2, 4):
            source = catalog.ROOT / f"art/sounds/sources/opengameart/step/subway/subway-step-{dict(zip((1, 2, 4), 'abc'))[number]}.ogg"
            self.assertIn("<svg", catalog.waveform(source))

    def test_ambience_review_generation(self):
        with tempfile.TemporaryDirectory() as folder:
            old_out = ambience.OUT
            ambience.OUT = pathlib.Path(folder) / "art/sounds/generated/amb"
            try:
                makers = (ambience.roomtone_conduit, ambience.boiler_tick,
                          ambience.vent_hvac, ambience.light_buzz_low,
                          ambience.tank_hum, ambience.low_pressure)
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
            {f"art/sounds/sources/opengameart/step/subway/subway-step-{letter}.ogg" for letter in "abc"},
        )
        self.assertEqual(len([path for path in selected if "/door/" in path]), 4)
        self.assertFalse(catalog.REVIEW_FILES)
        self.assertNotIn("art/sounds/generated/candidates/door/locked/rattling-locked-door-shelbyshark.wav", selected)
        self.assertEqual(len([path for path in selected if "Freesound preview" in catalog.RECORDED_PATHS.get(path, ("", ""))[0]]), 4)
        self.assertTrue(catalog.SOURCE_FILES.isdisjoint(selected))
        hiding = {path for path in selected if "/hiding/" in path}
        self.assertEqual(len(hiding), 4)
        for path in hiding:
            self.assertIn("<svg", catalog.waveform(catalog.ROOT / path))
        all_audio = {path.relative_to(catalog.ROOT).as_posix()
                     for path in (catalog.ROOT / "art/sounds").rglob("*")
                     if path.is_file() and path.suffix.lower() in catalog.EXTENSIONS}
        self.assertEqual(all_audio, catalog.APPROVED_FILES | catalog.SOURCE_FILES | {
            "art/sounds/generated/candidates/door/locked/rattling-locked-door-shelbyshark.wav",
        })

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
                if "/opengameart/step/" in path:
                    self.assertIn("GboxMikeFozzy", card)
                    self.assertNotIn("rubberduck", card)
                elif path.endswith("/door/locked-rattle.wav"):
                    self.assertIn("DrFahrts", card)
                    self.assertIn("Freesound preview", card)
                    self.assertNotIn("rubberduck", card)
                elif "/generated/door/" in path or "/generated/hiding/" in path:
                    self.assertIn("rubberduck", card)
                    self.assertNotIn("GboxMikeFozzy", card)
                elif "Freesound preview" in catalog.RECORDED_PATHS.get(path, ("", ""))[0]:
                    self.assertIn("Freesound preview", card)
                    self.assertIn("freesound.org/people/", card)
                else:
                    self.assertIn("Original project-generated sound", card)
                    self.assertNotIn("GboxMikeFozzy", card)
                    self.assertNotIn("rubberduck", card)
            for path in catalog.REVIEW_FILES:
                matching = [card for card in cards if f"<small>{path}</small>" in card]
                self.assertEqual(len(matching), 1, path)
                self.assertIn('class="review"', matching[0])
                self.assertIn("For review - not in game", matching[0])
                self.assertIn("Freesound preview", matching[0])
                self.assertTrue(any(name in matching[0] for name in ("shelbyshark", "DrFahrts")))
                self.assertNotIn("Original project-generated sound", matching[0])
                self.assertNotIn("GboxMikeFozzy", matching[0])
            self.assertIn("<h2>D. Facility ambience (bus ambience)</h2>", page)
            self.assertIn("<h2>G. Hiding (bus world)</h2>", page)
            self.assertIn("<h2>B. Doors (bus world)</h2>", page)
            self.assertIn("<h2>E. UI and front end (bus ui)</h2>", page)
            self.assertEqual(page.count('class="review"'), len(catalog.REVIEW_FILES))
            self.assertEqual(page.count('Freesound preview'), 4)
            for name in ("furnace-iankath", "faucet-willstepp", "wind-dblover"):
                self.assertNotIn(f"<strong>{name}</strong>", page)

    def test_runtime_sounds_are_present_and_edited_sources_are_retained(self):
        assets = (catalog.ROOT / "crates/assets/src/lib.rs").read_text()
        paths = re.findall(r'#\[asset\(path = "(sounds/[^\"]+)"\)\]', assets)
        self.assertTrue(paths)
        for path in paths:
            self.assertTrue((catalog.ROOT / "assets" / path).is_file(), path)
        for path in catalog.APPROVED_FILES:
            art = catalog.ROOT / path
            if "art/sounds/generated/" in path:
                runtime = catalog.ROOT / path.replace("art/sounds/generated/", "assets/sounds/")
            elif "art/sounds/sources/opengameart/" in path:
                runtime = catalog.ROOT / path.replace("art/sounds/sources/opengameart/", "assets/sounds/")
            else:
                self.fail(f"unhandled source origin: {path}")
            self.assertEqual(art.read_bytes(), runtime.read_bytes(), path)
        for cue, source in (
            ("furnace/burning.wav", "furnace/furnace-iankath.ogg"),
            ("water/faucet.wav", "water/faucet-willstepp.ogg"),
            ("vent/wind.wav", "vent/wind-dblover.ogg"),
        ):
            edit = catalog.ROOT / "art/sounds/generated/amb" / cue
            runtime = catalog.ROOT / "assets/sounds/amb" / cue
            original = catalog.ROOT / "art/sounds/sources/freesound/amb" / source
            self.assertEqual(edit.read_bytes(), runtime.read_bytes())
            self.assertTrue(original.is_file(), source)

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
