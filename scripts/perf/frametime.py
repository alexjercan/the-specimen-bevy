#!/usr/bin/env python3
"""Summarize a Bevy ``trace_chrome`` trace.

Usage: ``python3 scripts/perf/frametime.py [TRACE] [--keep]``

Produce the trace with a bounded headless run::

    RUST_LOG=bevy_ecs=info cargo run --features trace-chrome -- --headless --seed 1

The game's log filter drops ``bevy_ecs`` spans, so set
``RUST_LOG=bevy_ecs=info`` to get per system spans. Bevy writes
``trace-<unix micros>.json`` in the working directory, or the path in
``TRACE_CHROME``. Without an argument the script picks the newest
``trace-*.json`` in the working directory. The script prints frame time
percentiles and the top systems by self time, then deletes the trace unless
``--keep`` is given. A chrome trace holds CPU spans only, so the script never
prints GPU numbers.
"""

import json
import math
import os
import re
import sys
from collections import defaultdict
from pathlib import Path

FRAME_SPAN = "update"
SYSTEM_SPAN = "system"
RENDER_SPANS = {"main_render_schedule", "present_frames"}
TOP_SYSTEMS = 10
SYSTEM_SPAN_LOG = "bevy_ecs=info"

ANSI = re.compile(r"\x1b\[[0-9;]*m")
SYSTEM_NAME = re.compile(r'name="(?P<name>[^"]*)"')


def span_name(raw):
    """Return ``(kind, fields)`` for a tracing-chrome span name.

    Bevy names spans ``<kind>: <fields>`` and the fields may carry ANSI
    color codes when the fmt layer wrote them with colors on.
    """
    clean = ANSI.sub("", raw)
    kind, separator, fields = clean.partition(": ")
    if not separator:
        kind = clean.rstrip(": ")
        fields = ""
    return kind.strip(), fields.strip()


def system_label(fields):
    match = SYSTEM_NAME.search(fields)
    return match.group("name") if match else fields


def load_events(text):
    """Parse the JSON array, tolerating a trace cut off before its ``]``."""
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        trimmed = text.rstrip().rstrip(",")
        if not trimmed.endswith("]"):
            trimmed += "]"
        return json.loads(trimmed)


def timeline(events):
    """Yield ``(ts, order, kind, fields, tid)`` for B and E events in time order.

    ``X`` events become a B and an E pair. ``order`` puts E before B at an
    equal timestamp so sibling spans do not nest.
    """
    rows = []
    for index, event in enumerate(events):
        ph = event.get("ph")
        if ph not in ("B", "E", "X"):
            continue
        tid = event.get("tid", 0)
        ts = float(event.get("ts", 0.0))
        kind, fields = span_name(str(event.get("name", "")))
        if ph == "X":
            end = ts + float(event.get("dur", 0.0))
            rows.append((ts, 1, index, "B", kind, fields, tid))
            rows.append((end, 0, index, "E", kind, fields, tid))
        else:
            rows.append((ts, 1 if ph == "B" else 0, index, ph, kind, fields, tid))
    rows.sort(key=lambda row: (row[0], row[1], row[2]))
    for ts, _, _, ph, kind, fields, tid in rows:
        yield ts, ph, kind, fields, tid


class Summary:
    """Frame durations and per-system self times, both in microseconds."""

    def __init__(self):
        self.frames = []
        self.system_self = defaultdict(float)
        self.system_calls = defaultdict(int)
        self.render_spans = 0


def summarize(events):
    summary = Summary()
    stacks = defaultdict(list)
    for ts, ph, kind, fields, tid in timeline(events):
        stack = stacks[tid]
        if ph == "B":
            stack.append([kind, fields, ts, 0.0])
            continue
        if not stack:
            continue
        open_kind, open_fields, start, child_time = stack.pop()
        duration = max(ts - start, 0.0)
        if stack:
            stack[-1][3] += duration
        if open_kind == FRAME_SPAN:
            summary.frames.append(duration)
        elif open_kind == SYSTEM_SPAN:
            label = system_label(open_fields)
            summary.system_self[label] += max(duration - child_time, 0.0)
            summary.system_calls[label] += 1
        if open_kind in RENDER_SPANS:
            summary.render_spans += 1
    return summary


def percentile(values, percent):
    """Nearest-rank percentile of a sorted list."""
    if not values:
        return 0.0
    rank = math.ceil(percent / 100.0 * len(values))
    return values[max(rank, 1) - 1]


def top_systems(summary, count=TOP_SYSTEMS):
    ranked = sorted(summary.system_self.items(), key=lambda item: (-item[1], item[0]))
    return ranked[:count]


def report(summary):
    lines = []
    frames = sorted(summary.frames)
    lines.append(f"frames: {len(frames)}")
    if frames:
        for label, percent in (("p50", 50), ("p95", 95), ("p99", 99)):
            lines.append(f"frame time {label}: {percentile(frames, percent) / 1000.0:.3f} ms")
    else:
        lines.append("frame time: no 'update' spans in this trace")
    lines.append(f"top {TOP_SYSTEMS} systems by self time (total ms, calls):")
    if not summary.system_self:
        lines.append(f"  no system spans: run the game with RUST_LOG={SYSTEM_SPAN_LOG}")
    for name, self_time in top_systems(summary):
        calls = summary.system_calls[name]
        lines.append(f"  {self_time / 1000.0:10.3f} ms  {calls:6d}  {name}")
    if summary.render_spans == 0:
        lines.append("GPU: none, this is a headless trace. GPU numbers need a rendered run.")
    else:
        lines.append("GPU: a chrome trace holds CPU spans only. GPU numbers need Tracy on a rendered run.")
    return "\n".join(lines)


def newest_trace(directory):
    traces = sorted(Path(directory).glob("trace-*.json"), key=lambda path: path.stat().st_mtime)
    return traces[-1] if traces else None


def main(argv):
    keep = "--keep" in argv
    paths = [argument for argument in argv if argument != "--keep"]
    if paths:
        trace = Path(paths[0])
    else:
        trace = newest_trace(os.getcwd())
        if trace is None:
            print("no trace-*.json in the working directory", file=sys.stderr)
            return 2
    try:
        events = load_events(trace.read_text())
    except (OSError, json.JSONDecodeError) as error:
        print(f"cannot read {trace}: {error}", file=sys.stderr)
        return 2
    print(f"trace: {trace}")
    print(report(summarize(events)))
    if not keep:
        trace.unlink()
        print(f"deleted {trace}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
