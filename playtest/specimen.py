#!/usr/bin/env python3
import harness
from harness import Lines, game_command, send

CONTROLS = frozenset(("w", "a", "s", "d", "shift", "f", "flashlight", "flashbang"))

INSTRUCTIONS = (
    "Play the facility fairly. Explore the map, find three fuses, install them at the panel, "
    "and escape. The initial map lists possible fuse sites, not the selected sites. "
    "Only visible observations reveal whether a site has a fuse."
)


def rules(max_step):
    return ("Respond with ONLY JSON: {\"intent\":\"one short sentence describing your next action\","
            "\"frames\":1..MAX,\"input\":{\"w\":true/false,"
            "\"a\":true/false,\"s\":true/false,\"d\":true/false,\"shift\":true/false,"
            "\"f\":true/false,\"flashlight\":true/false,\"flashbang\":true/false,"
            "\"look\":[dx,dy]}}. "
            "Omitted buttons stay held; explicitly send false to release. Look applies once. "
            "Flashlight and flashbang are logical controls mapped to configured bindings. "
            "Flashbang is single-use: press flashbang to throw, then release it before another press. "
            "Player flashbangs is remaining inventory; flash_remaining is the temporary effect in seconds. "
            "Detector is passive: has_detector indicates ownership, detector gives nearby distance_m "
            "and bearing_deg, and null means no signal; do not infer monster location from null. "
            "Keep intent to one short, user-visible sentence, not private reasoning. "
            "Move and look by choosing a bounded number of frames. "
            "To open a door, face its visible bearing, approach within interaction range, "
            "press F, and release F before pressing it again. If blocked, change approach "
            "instead of repeating the same movement. "
            "You receive initial map only once; keep track of it. MAX=" + str(max_step))


def terminal(snapshot):
    return snapshot.get("won", False) or snapshot.get("game_over", False)


def progress_snapshot(snapshot):
    player = snapshot.get("player") or {}
    position = player.get("position")
    location = f" position={position}" if position is not None else ""
    visible = snapshot.get("visible", [])
    doors = [item for item in visible if item.get("kind") == "door"]
    nearest = min(doors, key=lambda item: item.get("distance_m", float("inf")), default=None)
    door = (f" nearest_door={nearest['distance_m']:.1f}m/"
            f"{nearest['bearing_deg'][0]:+.0f}deg/open={nearest['open']}" if nearest else "")
    devices = ""
    if "flashbangs" in player:
        devices += f" flashbangs={player['flashbangs']} flash_remaining={player.get('flash_remaining', 0):.1f}s"
    if "has_detector" in player:
        reading = player.get("detector")
        devices += (f" detector={reading['distance_m']:.1f}m/{reading['bearing_deg']:+.0f}deg"
                    if reading else f" detector={'no signal' if player['has_detector'] else 'not held'}")
    return (f"tick={snapshot['tick']}{location} won={snapshot.get('won', False)}"
            f" game_over={snapshot.get('game_over', False)}"
            f" visible={len(visible)} heard={len(snapshot.get('heard', []))}{door}{devices}")


def progress_input(controls, held=None):
    held = held if held is not None else set()
    changes = []
    for key, value in controls.items():
        if key == "look":
            continue
        verb = "press" if value and key not in held else "hold" if value else "release"
        changes.append(f"{verb} {key.upper() if len(key) == 1 else key}")
        if value:
            held.add(key)
        else:
            held.discard(key)
    if "look" in controls:
        changes.append(f"look={controls['look']}")
    return ", ".join(changes) if changes else "keep held controls"


def progress_decision(snapshot, wire, intent, seconds, held=None):
    observation = progress_snapshot(snapshot).removeprefix(f"tick={snapshot['tick']}")
    summary = " ".join(intent.split()) if intent else "intent unavailable"
    frames = wire['tick'] - snapshot['tick']
    unit = "tick" if frames == 1 else "ticks"
    return (f"[tick {snapshot['tick']}]{observation} -> {progress_input(wire['input'], held)}; "
            f"intent={summary}; decision_time={seconds:.1f}s; wait {frames} {unit}")


def command(action, now, cap, max_step):
    if not isinstance(action, dict) or set(action) - {"frames", "input", "intent"}:
        raise ValueError("action must contain frames, optional input and a short intent only")
    if "intent" in action and (not isinstance(action["intent"], str) or len(action["intent"]) > 240):
        raise ValueError("intent must be a short string (up to 240 characters)")
    frames = action.get("frames")
    if type(frames) is not int or not 1 <= frames <= max_step:
        raise ValueError(f"frames must be between 1 and {max_step}")
    controls = action.get("input", {})
    if not isinstance(controls, dict) or set(controls) - (CONTROLS | {"look"}):
        raise ValueError("input contains an unknown control")
    for key in CONTROLS & controls.keys():
        if type(controls[key]) is not bool:
            raise ValueError(f"{key} must be true or false")
    if "look" in controls:
        look = controls["look"]
        if not isinstance(look, list) or len(look) != 2 or any(
            type(value) not in (int, float) or not -10000 <= value <= 10000 for value in look
        ):
            raise ValueError("look must be two finite bounded numbers")
    if now + frames > cap:
        frames = cap - now
    if frames <= 0:
        raise ValueError("tick budget exhausted")
    return {"tick": now + frames, "input": controls}


class Specimen:
    INSTRUCTIONS = INSTRUCTIONS
    rules = staticmethod(rules)
    terminal = staticmethod(terminal)
    progress_snapshot = staticmethod(progress_snapshot)
    progress_decision = staticmethod(progress_decision)
    command = staticmethod(command)


def parser():
    return harness.parser("Run a bounded pi playtest of The Specimen over stdio")


def play(args):
    return harness.play(args, Specimen)


def main():
    harness.main(parser().parse_args(), Specimen)


if __name__ == "__main__":
    main()
