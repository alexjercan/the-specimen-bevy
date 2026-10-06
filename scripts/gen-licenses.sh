#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p credits
cargo about generate about.hbs -o credits/THIRD-PARTY-LICENSES.txt
