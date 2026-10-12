#!/usr/bin/env python3
import argparse
import json
import os
import queue
import subprocess
import sys
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = 1
# NOTE: the game's VideoProfile deadlines; the margin matches the e2e record run.
ENCODE_DEADLINES = {"preview": 120, "final": 600}
ENCODE_MARGIN = 180


def positive(value):
    number = int(value)
    if number <= 0:
        raise argparse.ArgumentTypeError("must be positive")
    return number


def parser(description):
    cli = argparse.ArgumentParser(description=description)
    cli.add_argument("--model", required=True)
    cli.add_argument("--provider", default="openai-codex")
    cli.add_argument("--thinking", default="medium", choices=("off", "minimal", "low", "medium", "high", "xhigh", "max"))
    prompts = cli.add_mutually_exclusive_group()
    prompts.add_argument("--prompt")
    prompts.add_argument("--prompt-file", type=Path)
    cli.add_argument("--ticks", type=positive, default=3600)
    cli.add_argument("--seconds", type=positive, default=600)
    cli.add_argument("--step-timeout", type=positive, default=90)
    cli.add_argument("--boot-timeout", type=positive, default=300)
    cli.add_argument("--encode-timeout", type=positive)
    cli.add_argument("--max-step", type=positive, default=120)
    cli.add_argument("--seed", type=int)
    cli.add_argument("--render", action="store_true")
    cli.add_argument("--noscreen", action="store_true")
    cli.add_argument("--record", type=Path)
    cli.add_argument("--video-profile", default="preview", choices=("preview", "final"))
    cli.add_argument("--game", type=Path)
    cli.add_argument("--pi", default="pi")
    cli.add_argument("--output", type=Path, default=ROOT / "target" / "playtest")
    return cli


class Progress:
    def __init__(self, path):
        self.stream = path.open("w")
        self.start = time.monotonic()

    def write(self, message):
        line = f"[playtest +{time.monotonic() - self.start:.1f}s] {message}"
        print(line, file=sys.stderr, flush=True)
        print(line, file=self.stream, flush=True)

    def close(self):
        self.stream.close()


class Lines:
    def __init__(self, stream, source, log, process=None):
        self.source = source
        self.log = log
        self.process = process
        self.messages = queue.Queue()
        self.thread = threading.Thread(target=self._read, args=(stream,), daemon=True)
        self.thread.start()

    def _read(self, stream):
        for line in stream:
            try:
                self.messages.put(json.loads(line))
            except (ValueError, UnicodeError) as error:
                self.messages.put({"error": f"invalid JSON line: {error}"})
        self.messages.put(None)

    def get(self, deadline):
        try:
            value = self.messages.get(timeout=max(0, deadline - time.monotonic()))
        except queue.Empty as error:
            raise TimeoutError("subprocess response timed out") from error
        if value is None:
            status = self.process.poll() if self.process else None
            if status is not None and status < 0:
                raise RuntimeError(f"{self.source} stopped by signal {-status}; see {self.log}")
            raise RuntimeError(f"{self.source} closed stdout; see {self.log}")
        return value


def send(process, message, source, log):
    try:
        process.stdin.write(json.dumps(message, separators=(",", ":")) + "\n")
        process.stdin.flush()
    except BrokenPipeError as error:
        status = process.poll()
        raise RuntimeError(
            f"{source} closed stdin while sending {message['type'] if source == 'pi' else 'tick'}"
            f" (exit status: {status if status is not None else 'pending'}); see {log}"
        ) from error


def rpc(process, lines, message, deadline):
    send(process, message, "pi", lines.log)
    while True:
        response = lines.get(deadline)
        if response.get("type") == "response" and response.get("id") == message["id"]:
            if not response.get("success"):
                raise RuntimeError(f"pi rejected {message['type']}: {response.get('error')}")
            return response.get("data")


def decide(process, lines, prompt, deadline, index):
    request = f"prompt-{index}"
    send(process, {"id": request, "type": "prompt", "message": prompt}, "pi", lines.log)
    accepted = False
    settled = False
    while not (accepted and settled):
        event = lines.get(deadline)
        if event.get("type") == "response" and event.get("id") == request:
            if not event.get("success"):
                raise RuntimeError(f"pi rejected prompt: {event.get('error')}")
            accepted = True
        elif event.get("type") == "message_end" and event.get("message", {}).get("role") == "assistant":
            reason = event["message"].get("stopReason")
            if reason in ("error", "aborted"):
                raise RuntimeError(f"pi stopped with {reason}; see pi.log")
        elif event.get("type") == "agent_settled":
            settled = True
    result = rpc(process, lines, {"id": f"answer-{index}", "type": "get_last_assistant_text"}, deadline)
    text = result.get("text") if result else None
    if not text:
        raise ValueError("pi returned no action")
    try:
        return json.loads(text)
    except ValueError as error:
        raise ValueError(f"pi action is not valid JSON ({error}): {text[:200]!r}...") from error


def game_reply(process, lines, deadline, expected=None):
    reply = lines.get(deadline)
    if "error" in reply:
        raise RuntimeError(f"game stdio error: {reply['error']}")
    if reply.get("version") != VERSION:
        raise RuntimeError(f"unsupported game protocol version: {reply.get('version')}, expected {VERSION}")
    if type(reply.get("tick")) is not int or (expected is not None and reply["tick"] != expected):
        raise RuntimeError(f"unexpected game tick: {reply.get('tick')}, expected {expected}")
    return reply


def close_child(process, timeout=5):
    try:
        if process.stdin and not process.stdin.closed:
            try:
                process.stdin.close()
            except BrokenPipeError:
                pass
        try:
            return process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.terminate()
            try:
                return process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                return process.wait(timeout=5)
    finally:
        if process.stdout:
            process.stdout.close()


def encode_timeout(args):
    if args.encode_timeout is not None:
        return args.encode_timeout
    return ENCODE_DEADLINES[args.video_profile] + ENCODE_MARGIN


def versioned(wire):
    return {"version": VERSION, **wire}


def game_command(args, executable=None):
    game_args = ([str(args.game)] if args.game else ["cargo", "run", "--", "--stdio"])
    if args.game:
        game_args.append("--stdio")
    if not (args.render or args.noscreen):
        game_args.append("--headless")
    if args.seed is not None:
        game_args.extend(("--seed", str(args.seed)))
    if args.record:
        game_args.extend(("--record", str(args.record.resolve()), "--video-profile", args.video_profile))
    return ["xvfb-run", "-a", *game_args] if args.noscreen else game_args


def play(args, game_adapter):
    if args.record and not (args.render or args.noscreen):
        raise ValueError("--record requires --render or --noscreen")
    if args.seed is not None and not 0 <= args.seed < 2**64:
        raise ValueError("seed must be a u64")
    args.output.mkdir(parents=True, exist_ok=True)
    game_log = (args.output / "game.log").open("w")
    pi_log = (args.output / "pi.log").open("w")
    progress = Progress(args.output / "progress.log")
    game_args = game_command(args)
    game = None
    agent = None
    try:
        environment = os.environ.copy()
        environment.setdefault("BEVY_ASSET_ROOT", str(ROOT))
        progress.write(f"starting game; waiting for ready tick 0 (log: {args.output / 'game.log'})")
        game = subprocess.Popen(game_args, cwd=ROOT, env=environment, stdin=subprocess.PIPE,
                                stdout=subprocess.PIPE, stderr=game_log, text=True, bufsize=1,
                                start_new_session=True)
        game_lines = Lines(game.stdout, "game", args.output / "game.log", game)
        initial = game_reply(game, game_lines, time.monotonic() + args.boot_timeout, 0)
        progress.write(f"game ready: {game_adapter.progress_snapshot(initial)}; map received")
        agent = subprocess.Popen([args.pi, "--mode", "rpc", "--no-session", "--provider", args.provider,
                                  "--model", args.model, "--thinking", args.thinking,
                                  "--no-tools", "--no-extensions",
                                  "--no-skills", "--no-prompt-templates", "--no-context-files"],
                                 cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                 stderr=pi_log, text=True, bufsize=1, start_new_session=True)
        agent_lines = Lines(agent.stdout, "pi", args.output / "pi.log", agent)
        rpc(agent, agent_lines, {"id": "startup", "type": "get_state"}, time.monotonic() + args.boot_timeout)
        progress.write(f"pi ready: {args.provider}/{args.model}; trace: {args.output / 'trace.jsonl'}")
        if args.prompt is not None:
            instructions = args.prompt
        elif args.prompt_file:
            instructions = args.prompt_file.read_text()
        else:
            instructions = game_adapter.INSTRUCTIONS
        rules = game_adapter.rules(args.max_step)
        end = time.monotonic() + args.seconds
        snapshot = initial
        timed_out = False
        with (args.output / "trace.jsonl").open("w") as trace:
            trace.write(json.dumps({"snapshot": initial}) + "\n")
            index = 0
            held = set()
            while snapshot["tick"] < args.ticks and not game_adapter.terminal(snapshot):
                deadline = min(end, time.monotonic() + args.step_timeout)
                if deadline <= time.monotonic():
                    timed_out = True
                    break
                prompt = (instructions + "\n" + rules + "\nSnapshot: " + json.dumps(snapshot)
                          if index == 0 else "Snapshot: " + json.dumps(snapshot) + "\n" + rules)
                decision_start = time.monotonic()
                try:
                    for attempt in range(3):
                        try:
                            action = decide(agent, agent_lines, prompt, deadline, f"{index}-{attempt}")
                            wire = game_adapter.command(action, snapshot["tick"], args.ticks, args.max_step)
                            break
                        except ValueError as error:
                            if attempt == 2:
                                raise ValueError(f"pi returned three invalid actions at tick "
                                                 f"{snapshot['tick']}: {error}") from error
                            progress.write(f"invalid pi action at tick {snapshot['tick']}; "
                                           f"retrying ({attempt + 1}/2): {error}")
                            prompt = (f"Your last action was invalid: {str(error)[:120]}. "
                                      "Return ONLY one valid JSON object matching the action format.\n"
                                      f"Snapshot: {json.dumps(snapshot)}\n{rules}")
                except TimeoutError:
                    if time.monotonic() < end:
                        raise
                    timed_out = True
                    break
                decision_time = time.monotonic() - decision_start
                intent = action.get("intent")
                progress.write(game_adapter.progress_decision(snapshot, wire, intent, decision_time, held))
                send(game, versioned(wire), "game", game_lines.log)
                snapshot = game_reply(game, game_lines, time.monotonic() + args.step_timeout, wire["tick"])
                trace.write(json.dumps({"action": wire, "intent": intent,
                                        "decision_time_s": decision_time, "snapshot": snapshot}) + "\n")
                trace.flush()
                index += 1
        status = ("won" if snapshot.get("won") else "game_over" if snapshot.get("game_over")
                  else "tick_limit" if snapshot["tick"] >= args.ticks else "timeout" if timed_out
                  else "stopped")
        reason = (f"wall-clock limit of {args.seconds}s reached before tick {args.ticks}; "
                  "increase --seconds to allow more model decisions" if status == "timeout" else None)
        progress.write(f"playtest {status}: {reason + '; ' if reason else ''}"
                       f"{game_adapter.progress_snapshot(snapshot)}; closing game and pi")
        return {**snapshot, "status": status, "reason": reason}
    finally:
        progress.write("stopping children; waiting for recording finalization" if args.record else "stopping children")
        game_status = close_child(game, timeout=encode_timeout(args) if args.record else 5) if game else None
        if agent:
            close_child(agent)
        game_log.close()
        pi_log.close()
        progress.close()
        if sys.exc_info()[0] is None:
            if game_status not in (None, 0):
                raise RuntimeError(f"game exited with status {game_status}; see {args.output / 'game.log'}")
            if args.record:
                video = args.record.resolve()
                if not video.is_file() or video.stat().st_size == 0:
                    raise RuntimeError(f"game exited without a video at {video}; see {args.output / 'game.log'}")


def main(args, game_adapter):
    try:
        result = play(args, game_adapter)
        print(json.dumps({"status": result["status"], "reason": result["reason"],
                          "tick": result["tick"], "won": result.get("won", False),
                          "game_over": result.get("game_over", False), "output": str(args.output)}))
    except KeyboardInterrupt:
        print("playtest interrupted; children stopped (see output/progress.log and trace.jsonl)", file=sys.stderr)
        raise SystemExit(130) from None
    except (OSError, RuntimeError, ValueError, TimeoutError) as error:
        print(f"playtest failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
