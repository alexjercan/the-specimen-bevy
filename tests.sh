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
python3 -m unittest playtest.tests.test_specimen
python3 -m unittest scripts.perf.test_frametime
scripts/check-architecture.sh
cargo test --workspace --locked
cargo check --locked --all-targets
cargo build --locked
env -u DISPLAY timeout 120s "${CARGO_TARGET_DIR:-target}/debug/the-specimen-bevy" --headless --seed 1
if [[ "$mode" == --full ]]; then
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo clippy --workspace --all-targets --locked --features debug,halloween -- -D warnings
  cargo test --workspace --locked --features debug,halloween
  VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-${BEVY_SOFTWARE_VK_ICD:-}}" xvfb-run -a cargo test --locked --test e2e -- --rendered
  VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-${BEVY_SOFTWARE_VK_ICD:-}}" xvfb-run -a cargo test --locked --test e2e -- --credits
fi
