#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

if ! command -v cargo &>/dev/null; then
  echo "ERROR: cargo not found" >&2
  exit 1
fi

echo "[kcu-screenshot] building runner..."
exec "${REPO_ROOT}/scripts/maintenance/cargo-guard" run -- cargo run --release --locked --manifest-path "${SCRIPT_DIR}/Cargo.toml" --quiet -- "$@"
