#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
mkdir -p target/probe

for example in facility_probe facility_overhead_probe; do
    cargo run --release --example "$example" -- \
        --probe-label "$example" \
        --probe-out "target/probe/$example.json" \
        "$@"
done
