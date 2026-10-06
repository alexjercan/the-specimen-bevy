#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

asset=concrete_floor_worn_001
dir="art/visuals/sources/polyhaven/$asset"
base="https://dl.polyhaven.org/file/ph-assets/Textures/jpg/1k/$asset"
mkdir -p "$dir"

while read -r sum name; do
  path="$dir/$name"
  if [[ ! -f "$path" ]]; then
    curl -fsSL -o "$path.part" "$base/$name"
    mv "$path.part" "$path"
  fi
  echo "$sum  $path" | sha256sum --check --quiet
done <<EOF
6e40c0fc908f4d66431836f5abe203f3a4eb064e88824378a91a46a26e2f464c ${asset}_diff_1k.jpg
523bd94a1be5dd2f11c2ae2b7d7c77445cf5bd9e41604db515a7c8364dcba067 ${asset}_nor_gl_1k.jpg
b21e63637d04b969894bd40a45df185d77671ed74b8955cd6e403c34ed465043 ${asset}_arm_1k.jpg
EOF
echo "textures ok: $dir"
