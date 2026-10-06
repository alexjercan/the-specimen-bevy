#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

mode="${1:---full}"
case "$mode" in
  --smoke|--full) ;;
  *) echo "usage: ./tests.sh [--smoke|--full]" >&2; exit 2 ;;
esac

cargo fmt --all -- --check
python3 scripts/check-comments.py
cargo test --workspace --locked
cargo check --locked --all-targets
env -u DISPLAY timeout 60s cargo run --locked -- --norender
if [[ "$mode" == --full ]]; then
  cargo clippy --workspace --all-targets --locked -- -D warnings
fi
