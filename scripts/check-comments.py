#!/usr/bin/env python3
import pathlib
import re
import sys

root = pathlib.Path(__file__).resolve().parent.parent
allowed = re.compile(r"(?:NOTE:|XXX:|WTF:|TODO\(horror_game_bevy\):)")
extensions = {".rs", ".sh", ".py", ".nix", ".toml", ".yml", ".yaml", ".wgsl"}
errors = []

for path in root.rglob("*"):
    if not path.is_file() or path.suffix not in extensions:
        continue
    relative = path.relative_to(root)
    if any(part in {".git", "target", "art", "assets", "tasks", "dist"} for part in relative.parts):
        continue
    for number, line in enumerate(path.read_text().splitlines(), 1):
        stripped = line.lstrip()
        if stripped.startswith("#!") or stripped.startswith("#["):
            continue
        marker = None
        quote = None
        index = 0
        while index < len(line):
            char = line[index]
            if char == "\\" and quote:
                index += 2
                continue
            if quote:
                if char == quote:
                    quote = None
            elif char in ('"', "'") and path.suffix != ".rs":
                quote = char
            elif char == '"' and path.suffix == ".rs":
                quote = char
            elif path.suffix == ".rs" and line[index:index + 2] == "//":
                marker = line[index + 2:]
                break
            elif path.suffix not in {".rs", ".py"} and char == "#":
                marker = line[index + 1:]
                break
            elif path.suffix == ".py" and char == "#":
                marker = line[index + 1:]
                break
            index += 1
        if marker is None:
            continue
        match = allowed.match(marker.strip())
        if not match:
            errors.append(f"{relative}:{number}: unmarked comment or closed TODO")

if errors:
    print("\n".join(errors), file=sys.stderr)
    sys.exit(1)
