#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

out="art/visuals/generated"
./scripts/fetch-facility-textures.sh
for target in "${@:-kit}"; do
  case "$target" in
    facility) args=(--out "$out/facility.glb" --manifest "$out/facility.manifest.json") ;;
    kit) args=(--out "$out/modules") ;;
    *) echo "usage: scripts/generate-facility.sh [kit] [facility]" >&2; exit 2 ;;
  esac
  PYTHONHASHSEED=0 blender --background --factory-startup --python-exit-code 1 \
    --python scripts/generate_facility.py -- --root . --target "$target" "${args[@]}"
done
