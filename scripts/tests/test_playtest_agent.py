import contextlib
import importlib.util
import io
import os
import signal
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "playtest_agent.py"
spec = importlib.util.spec_from_file_location("playtest_agent", SCRIPT)
playtest = importlib.util.module_from_spec(spec)
spec.loader.exec_module(playtest)


class ProtocolTests(unittest.TestCase):
    def test_terminal_snapshot_includes_game_over(self):
        self.assertFalse(playtest.terminal({"won": False, "game_over": False}))
        self.assertTrue(playtest.terminal({"won": True, "game_over": False}))
        self.assertTrue(playtest.terminal({"won": False, "game_over": True}))

    def test_persistent_controls_and_tick_cap(self):
        self.assertEqual(
            playtest.command({"frames": 10, "input": {"w": True}}, 8, 12, 120),
            {"tick": 12, "input": {"w": True}},
        )
        self.assertEqual(playtest.command({"frames": 1}, 12, 13, 120), {"tick": 13, "input": {}})

    def test_flashbang_control_and_device_progress(self):
        self.assertEqual(playtest.command({"frames": 1, "input": {"flashbang": True}}, 0, 10, 120),
                         {"tick": 1, "input": {"flashbang": True}})
        self.assertEqual(playtest.command({"frames": 1, "input": {"flashbang": False}}, 1, 10, 120),
                         {"tick": 2, "input": {"flashbang": False}})
        held = set()
        self.assertEqual(playtest.progress_input({"flashbang": True}, held), "press flashbang")
        self.assertEqual(playtest.progress_input({}, held), "keep held controls")
        self.assertEqual(playtest.progress_input({"flashbang": False}, held), "release flashbang")
        snapshot = {"tick": 1, "player": {"position": [0, 1.6, 0], "flashbangs": 1,
                     "flash_remaining": 4.5, "has_detector": True,
                     "detector": {"distance_m": 12.3, "bearing_deg": -20}}, "visible": [], "heard": []}
        self.assertIn("flashbangs=1 flash_remaining=4.5s detector=12.3m/-20deg",
                      playtest.progress_snapshot(snapshot))
        snapshot["player"]["detector"] = None
        self.assertIn("detector=no signal", playtest.progress_snapshot(snapshot))

    def test_rejects_bad_actions(self):
        bad = (
            {}, {"frames": True}, {"frames": 0}, {"frames": 121},
            {"frames": 1, "input": {"f": 1}},
            {"frames": 1, "input": {"look": [float("nan"), 0]}},
            {"frames": 1, "input": {"map": True}},
            {"frames": 1, "command": "quit"},
            {"frames": 1, "intent": 123},
            {"frames": 1, "intent": "a" * 241},
        )
        for action in bad:
            with self.subTest(action=action), self.assertRaises(ValueError):
                playtest.command(action, 0, 120, 120)

    def test_progress_pairs_observation_with_next_action(self):
        snapshot = {"tick": 255, "player": {"position": [-0.874, 1.6, -18.12]},
                    "visible": [{"kind": "door", "distance_m": 1.98,
                                 "bearing_deg": [-11.3, -14.6], "open": False}], "heard": []}
        wire = playtest.command({"frames": 1, "intent": "Try the maintenance door", "input": {
            "w": False, "f": True, "look": [-30, 0],
        }}, 255, 3600, 120)
        line = playtest.progress_decision(snapshot, wire, "Try the maintenance door", 1.2)
        self.assertIn("[tick 255] position=[-0.874, 1.6, -18.12]", line)
        self.assertIn("nearest_door=2.0m/-11deg/open=False", line)
        self.assertIn("-> release W, press F, look=[-30, 0]", line)
        self.assertTrue(line.endswith("intent=Try the maintenance door; decision_time=1.2s; wait 1 tick"))
        longer = playtest.progress_decision(
            snapshot, {"tick": 375, "input": {"w": True}}, "Explore ahead", 2.4,
        )
        self.assertTrue(longer.endswith("decision_time=2.4s; wait 120 ticks"))
        held = set()
        self.assertEqual(playtest.progress_input({"w": True}, held), "press W")
        self.assertEqual(playtest.progress_input({"w": True}, held), "hold W")
        self.assertEqual(playtest.progress_input({}, held), "keep held controls")
        self.assertEqual(playtest.progress_input({"w": False}, held), "release W")
        self.assertEqual(held, set())
        self.assertEqual(playtest.command({"frames": 1, "intent": "move"}, 0, 2, 120),
                         {"tick": 1, "input": {}})

    def test_closed_stream_identifies_its_process_log(self):
        lines = playtest.Lines(io.StringIO(""), "game", "game.log")
        with self.assertRaisesRegex(RuntimeError, "game closed stdout; see game.log"):
            lines.get(__import__("time").monotonic() + 1)

    def test_broken_pipe_identifies_pi_and_log(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            agent = root / "pi"
            agent.write_text("#!/usr/bin/env python3\nimport sys\nsys.stdin.close()\n")
            agent.chmod(0o755)
            process = subprocess.Popen([str(agent)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, text=True)
            process.wait(timeout=5)
            with self.assertRaisesRegex(RuntimeError, r"pi closed stdin while sending prompt.*pi.log"):
                playtest.send(process, {"type": "prompt", "message": "hello"}, "pi", root / "pi.log")
            with contextlib.suppress(BrokenPipeError):
                process.stdin.close()
            process.stdout.close()
            process.stderr.close()

    def test_inline_prompt_and_file_are_exclusive(self):
        options = playtest.parser().parse_args(["--model", "luna", "--prompt", "Find three fuses"])
        self.assertEqual(options.prompt, "Find three fuses")
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            playtest.parser().parse_args([
                "--model", "luna", "--prompt", "Find three fuses", "--prompt-file", "prompt.txt",
            ])

    def test_cannot_record_headless(self):
        options = playtest.parser().parse_args(["--model", "test", "--record", "movie.webm"])
        with self.assertRaisesRegex(ValueError, "requires --render"):
            playtest.play(options)

    def test_noscreen_uses_xvfb_and_mutes_rendered_game(self):
        options = playtest.parser().parse_args([
            "--model", "test", "--game", "/tmp/game", "--noscreen",
            "--record", "movie.webm", "--seed", "42",
        ])
        args = playtest.game_command(options)
        self.assertEqual(args[:4], ["xvfb-run", "-a", "/tmp/game", "--transport"])
        self.assertIn("--mute", args)
        self.assertNotIn("--norender", args)
        self.assertEqual(args[args.index("--seed") + 1], "42")
        normal = playtest.parser().parse_args(["--model", "test", "--game", "/tmp/game"])
        self.assertIn("--norender", playtest.game_command(normal))
        self.assertNotIn("--mute", playtest.game_command(normal))

    def test_fake_game_and_pi_exchange_initial_map_and_dynamic_ticks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            game = root / "game"
            game.write_text("""#!/usr/bin/env python3
import json, os, sys
assert os.path.isdir(os.path.join(os.environ['BEVY_ASSET_ROOT'], 'assets'))
print(json.dumps({'tick':0,'map':{'rooms':[]},'won':False}), flush=True)
for line in sys.stdin:
    data = json.loads(line)
    print(json.dumps({'tick':data['tick'],'won':data['tick'] >= 2}), flush=True)
if '--record' in sys.argv:
    open(sys.argv[sys.argv.index('--record') + 1], 'wb').write(b'fake video')
""")
            game.chmod(0o755)
            agent = root / "pi"
            agent.write_text("""#!/usr/bin/env python3
import json, sys
if sys.argv[sys.argv.index('--provider') + 1] != 'openai-codex':
    sys.exit(2)
if sys.argv[sys.argv.index('--model') + 1] != 'gpt-6-luna':
    sys.exit(2)
for line in sys.stdin:
    request = json.loads(line)
    print(json.dumps({'type':'response','id':request['id'],'success':True,'data':{'text':'{"intent":"Explore ahead","frames":1,"input":{"w":true}}'}}), flush=True)
    if request['type'] == 'prompt':
        assert '"flashbang":true/false' in request['message']
        assert 'has_detector' in request['message']
        print(json.dumps({'type':'agent_settled'}), flush=True)
""")
            agent.chmod(0o755)
            options = playtest.parser().parse_args([
                "--model", "gpt-6-luna", "--provider", "openai-codex",
                "--game", str(game), "--pi", str(agent),
                "--output", str(root / "output"), "--ticks", "3", "--seconds", "5",
                "--render", "--record", str(root / "run.webm"),
            ])
            output = io.StringIO()
            with contextlib.redirect_stderr(output):
                result = playtest.play(options)
            self.assertEqual(result["tick"], 2)
            progress = (root / "output" / "progress.log").read_text()
            self.assertEqual(progress, output.getvalue())
            self.assertIn("game ready: tick=0", progress)
            self.assertIn("[tick 0] won=False game_over=False visible=0 heard=0 -> press W", progress)
            self.assertIn("intent=Explore ahead; decision_time=", progress)
            self.assertIn("[tick 1] won=False game_over=False visible=0 heard=0 -> hold W", progress)
            self.assertIn("wait 1 tick", progress)
            self.assertNotIn("waiting for pi", progress)
            self.assertIn("playtest finished: tick=2", progress)
            self.assertTrue(result["won"])
            trace = [__import__("json").loads(line) for line in (root / "output" / "trace.jsonl").read_text().splitlines()]
            self.assertEqual(len(trace), 3)
            self.assertIn("map", trace[0]["snapshot"])
            self.assertNotIn("map", trace[1]["snapshot"])
            self.assertEqual(trace[1]["action"], {"tick": 1, "input": {"w": True}})
            self.assertEqual(trace[1]["intent"], "Explore ahead")
            self.assertGreaterEqual(trace[1]["decision_time_s"], 0)
            self.assertEqual((root / "run.webm").read_bytes(), b"fake video")

    def test_interruption_preserves_trace_and_closes_children(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            game = root / "game"
            game.write_text("""#!/usr/bin/env python3
import json, sys
print(json.dumps({'tick':0,'map':{},'won':False}), flush=True)
for line in sys.stdin:
    print(json.dumps({'tick':json.loads(line)['tick'],'won':False}), flush=True)
""")
            game.chmod(0o755)
            agent = root / "pi"
            agent.write_text("""#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    request = json.loads(line)
    print(json.dumps({'type':'response','id':request['id'],'success':True,'data':{}}), flush=True)
""")
            agent.chmod(0o755)
            options = playtest.parser().parse_args([
                "--model", "test", "--game", str(game), "--pi", str(agent),
                "--output", str(root / "output"), "--ticks", "2",
            ])
            with patch.object(playtest, "decide", side_effect=KeyboardInterrupt), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(KeyboardInterrupt):
                    playtest.play(options)
            self.assertEqual(len((root / "output" / "trace.jsonl").read_text().splitlines()), 1)
            self.assertIn("stopping children", (root / "output" / "progress.log").read_text())

    def test_sigint_finalizes_recording_without_signaling_game(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            game = root / "game"
            game.write_text("""#!/usr/bin/env python3
import json, sys
print(json.dumps({'tick':0,'won':False}), flush=True)
for line in sys.stdin:
    print(json.dumps({'tick':json.loads(line)['tick'],'won':False}), flush=True)
if '--record' in sys.argv:
    open(sys.argv[sys.argv.index('--record') + 1], 'wb').write(b'finished video')
""")
            game.chmod(0o755)
            agent = root / "pi"
            agent.write_text("""#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    request = json.loads(line)
    if request['type'] == 'get_state':
        print(json.dumps({'type':'response','id':request['id'],'success':True,'data':{}}), flush=True)
""")
            agent.chmod(0o755)
            recording = root / "recording.webm"
            output = root / "output"
            process = subprocess.Popen([
                "python3", str(SCRIPT), "--model", "test", "--game", str(game),
                "--pi", str(agent), "--output", str(output), "--render", "--record", str(recording),
            ], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                deadline = time.monotonic() + 5
                while time.monotonic() < deadline:
                    if (output / "progress.log").exists() and "pi ready:" in (output / "progress.log").read_text():
                        break
                    time.sleep(0.02)
                else:
                    self.fail("runner did not reach its first decision")
                os.kill(process.pid, signal.SIGINT)
                _, stderr = process.communicate(timeout=10)
                self.assertEqual(process.returncode, 130, stderr)
                self.assertIn("playtest interrupted", stderr)
                self.assertEqual(recording.read_bytes(), b"finished video")
                self.assertEqual(len((output / "trace.jsonl").read_text().splitlines()), 1)
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate(timeout=5)

    def test_missing_recording_is_a_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            game = root / "game"
            game.write_text("""#!/usr/bin/env python3
import json
print(json.dumps({'tick':0,'won':True}), flush=True)
""")
            game.chmod(0o755)
            options = playtest.parser().parse_args([
                "--model", "gpt-6-luna", "--game", str(game),
                "--output", str(root / "output"), "--render", "--record", str(root / "missing.webm"),
            ])
            with self.assertRaisesRegex(RuntimeError, "without a video"):
                playtest.play(options)


if __name__ == "__main__":
    unittest.main()
