#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v cargo &>/dev/null; then
  echo "ERROR: cargo not found" >&2
  exit 1
fi

echo "[kcu-screenshot] building runner..."
exec cargo run --release --locked --manifest-path "${SCRIPT_DIR}/Cargo.toml" --quiet -- "$@"
