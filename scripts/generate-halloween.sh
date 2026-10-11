#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

out="art/visuals/generated/halloween"
PYTHONHASHSEED=0 blender --background --factory-startup --python-exit-code 1 \
  --python scripts/generate_halloween.py -- --root . --out "$out"
