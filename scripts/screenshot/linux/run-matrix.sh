#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
mode="matrix"
if [[ ${1:-} == "--storybook-smoke" ]]; then
  mode="storybook-smoke"
  shift
fi
if [[ $# -gt 1 || ( $# -eq 1 && $1 == -* ) ]]; then
  echo "usage: run-matrix.sh [--storybook-smoke] [output-dir]" >&2
  exit 2
fi
default_output_dir="${root_dir}/target/native-harness-screenshots"
if [[ "$mode" == "storybook-smoke" ]]; then
  default_output_dir="${root_dir}/target/storybook-window-smoke"
fi
output_dir="${1:-$default_output_dir}"
image_tag="kcu-screenshot-linux-arm64:local"
if [[ "$mode" == "matrix" ]]; then
  python3 -m unittest discover -s "${root_dir}/scripts/screenshot/linux" -p 'test_*.py'
fi
mkdir -p "$output_dir"
output_dir="$(cd "$output_dir" && pwd)"

docker build --platform linux/arm64 \
  --file "${root_dir}/scripts/screenshot/linux/Dockerfile" \
  --tag "$image_tag" "${root_dir}/scripts/screenshot/linux" 2>&1 | tee "$output_dir/docker-build.log"
docker image inspect "$image_tag" > "$output_dir/docker-image.json"

docker run --rm --platform linux/arm64 \
  --mount "type=bind,src=${root_dir},dst=/workspace,readonly" \
  --mount "type=bind,src=${output_dir},dst=/artifacts" \
  --mount "type=volume,src=kcu-v010-linux-target,dst=/target" \
  --mount "type=volume,src=kcu-v010-linux-registry,dst=/usr/local/cargo/registry" \
  --workdir /workspace \
  --env CARGO_TARGET_DIR=/target \
  --env CARGO_INCREMENTAL=0 \
  --env CARGO_PROFILE_DEV_DEBUG=0 \
  --env CARGO_BUILD_JOBS=2 \
  --env 'RUSTFLAGS=-D warnings' \
  "$image_tag" \
  bash scripts/screenshot/linux/run-checks.sh "$mode"
