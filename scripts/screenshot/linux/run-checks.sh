#!/usr/bin/env bash
set -euo pipefail

mode="${1:-matrix}"
if [[ $# -gt 1 || ( "$mode" != "matrix" && "$mode" != "storybook-smoke" ) ]]; then
  echo "usage: run-checks.sh [matrix|storybook-smoke]" >&2
  exit 2
fi

if [[ "$mode" == "storybook-smoke" ]]; then
  exec bash scripts/screenshot/linux/run-xvfb.sh \
    cargo run --release --locked -p kcu-storybook -- --smoke
fi

cargo test --locked -p katana-chat-ui --lib \
  kuc::output::tests::raster::output_action_label_pixels_stay_inside_the_button \
  -- --exact

bash scripts/screenshot/linux/run-xvfb.sh \
  python3 scripts/screenshot/linux/matrix.py \
    --baseline scripts/screenshot/linux/baselines/aarch64.json \
    --requests scripts/screenshot/examples \
    --output /artifacts

bash scripts/screenshot/linux/run-xvfb.sh \
  cargo run --release --locked -p kcu-storybook -- --smoke
