#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

source_dir="art/visuals/generated/halloween"
destination="assets/halloween"
python3 scripts/check_facility_modules.py "$source_dir" >/dev/null
mkdir -p "$destination"
cp "$source_dir"/*.glb "$source_dir/modules.manifest.json" "$destination/"
python3 scripts/check_facility_modules.py "$destination" >/dev/null
cmp "$source_dir/modules.manifest.json" "$destination/modules.manifest.json"
echo "Promoted and verified halloween modules in $destination"
