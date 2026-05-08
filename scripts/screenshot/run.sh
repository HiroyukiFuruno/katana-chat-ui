#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v cargo &>/dev/null; then
  echo "ERROR: cargo not found" >&2
  exit 1
fi

echo "[kcu-screenshot] building runner..."
cargo build --release --manifest-path "${SCRIPT_DIR}/Cargo.toml" --quiet

RUNNER="${SCRIPT_DIR}/target/release/kcu-screenshot"
exec "$RUNNER" "$@"
