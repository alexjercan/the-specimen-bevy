#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

out="art/visuals/generated"
./scripts/fetch-facility-textures.sh
PYTHONHASHSEED=0 blender --background --factory-startup --python-exit-code 1 \
  --python scripts/generate_facility.py -- \
  --root . --out "$out/facility.glb" --manifest "$out/facility.manifest.json"
