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
cargo build --locked
set +e
env -u DISPLAY timeout 60s target/debug/the-specimen-bevy --norender
status=$?
set -e
if [[ "$status" -ne 0 && "$status" -ne 124 ]]; then
  echo "headless smoke run failed with status $status" >&2
  exit "$status"
fi
if [[ "$mode" == --full ]]; then
  cargo clippy --workspace --all-targets --locked -- -D warnings
fi
