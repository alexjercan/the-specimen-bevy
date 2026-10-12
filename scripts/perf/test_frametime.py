import contextlib
import importlib.util
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent / "frametime.py"
spec = importlib.util.spec_from_file_location("frametime", SCRIPT)
frametime = importlib.util.module_from_spec(spec)
spec.loader.exec_module(frametime)


def span(ph, name, ts, tid=1):
    return {"ph": ph, "pid": 1, "tid": tid, "ts": ts, "name": name, "cat": "bevy"}


def frame(start, systems):
    """Build one ``update`` frame with nested ``system`` spans.

    ``systems`` is a list of ``(label, begin, end, children)`` with times
    relative to ``start``. Children nest inside their parent system.
    """
    events = [span("B", "update: ", start)]
    end = start

    def emit(label, begin, finish, children):
        nonlocal end
        name = f'system: \x1b[3mname\x1b[0m\x1b[2m=\x1b[0m"{label}"'
        events.append(span("B", name, start + begin))
        for child in children:
            emit(*child)
        events.append(span("E", name, start + finish))
        end = max(end, start + finish)

    for system in systems:
        emit(*system)
    events.append(span("E", "update: ", end + 100))
    return events


def fixture():
    events = [
        {"ph": "M", "pid": 1, "tid": 1, "name": "thread_name", "args": {"name": "main"}},
        span("B", "bevy_app", 0),
    ]
    starts = [0, 10_000, 20_000, 30_000, 40_000]
    for index, start in enumerate(starts):
        slow = 8_000 if index == 4 else 1_000
        events += frame(
            start,
            [
                ("game::move", 0, slow, [("game::physics", 100, 500, [])]),
                ("game::audio", slow + 100, slow + 300, []),
            ],
        )
    return events


class ParseTests(unittest.TestCase):
    def test_span_name_strips_ansi_and_fields(self):
        kind, fields = frametime.span_name('system: \x1b[3mname\x1b[0m="a::b"')
        self.assertEqual(kind, "system")
        self.assertEqual(frametime.system_label(fields), "a::b")
        self.assertEqual(frametime.span_name("update: "), ("update", ""))
        self.assertEqual(frametime.span_name("present_frames"), ("present_frames", ""))

    def test_frame_times_and_percentiles(self):
        summary = frametime.summarize(fixture())
        self.assertEqual(sorted(summary.frames), [1400, 1400, 1400, 1400, 8400])
        frames = sorted(summary.frames)
        self.assertEqual(frametime.percentile(frames, 50), 1400)
        self.assertEqual(frametime.percentile(frames, 99), 8400)

    def test_self_time_excludes_children(self):
        summary = frametime.summarize(fixture())
        self.assertEqual(summary.system_self["game::physics"], 400 * 5)
        self.assertEqual(summary.system_self["game::move"], (1000 - 400) * 4 + (8000 - 400))
        self.assertEqual(summary.system_calls["game::audio"], 5)
        top = [name for name, _ in frametime.top_systems(summary)]
        self.assertEqual(top, ["game::move", "game::physics", "game::audio"])

    def test_missing_system_spans_prints_the_log_hint(self):
        events = [span("B", "update: ", 0), span("E", "update: ", 1000)]
        text = frametime.report(frametime.summarize(events))
        self.assertIn("RUST_LOG=bevy_ecs=info", text)

    def test_headless_trace_refuses_gpu_numbers(self):
        text = frametime.report(frametime.summarize(fixture()))
        self.assertIn("GPU numbers need a rendered run", text)
        self.assertIn("frame time p99: 8.400 ms", text)

    def test_rendered_trace_still_has_no_gpu_numbers(self):
        events = fixture() + [
            span("B", "present_frames", 50_000, tid=2),
            span("E", "present_frames", 50_500, tid=2),
        ]
        text = frametime.report(frametime.summarize(events))
        self.assertIn("GPU numbers need Tracy on a rendered run", text)

    def test_complete_events_and_truncated_file(self):
        events = [
            {"ph": "X", "tid": 1, "ts": 0, "dur": 2000, "name": "update"},
            {"ph": "X", "tid": 1, "ts": 100, "dur": 500, "name": 'system: name="a"'},
        ]
        text = json.dumps(events)
        truncated = text[: text.rfind("}") + 1] + ",\n"
        summary = frametime.summarize(frametime.load_events(truncated))
        self.assertEqual(summary.frames, [2000])
        self.assertEqual(summary.system_self["a"], 500)

    def test_main_deletes_the_trace(self):
        with tempfile.TemporaryDirectory() as directory:
            trace = Path(directory) / "trace-1.json"
            trace.write_text(json.dumps(fixture()))
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(frametime.main([str(trace)]), 0)
                self.assertFalse(trace.exists())
                trace.write_text(json.dumps(fixture()))
                self.assertEqual(frametime.main([str(trace), "--keep"]), 0)
            self.assertTrue(trace.exists())


if __name__ == "__main__":
    sys.exit(unittest.main())
