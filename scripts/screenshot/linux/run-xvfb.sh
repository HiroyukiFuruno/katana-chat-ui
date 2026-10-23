#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "usage: run-xvfb.sh <command> [arguments...]" >&2
  exit 2
fi

runtime_dir=$(mktemp -d)
display_file="$runtime_dir/display"
xvfb_pid=""
cleanup() {
  if [[ -n "$xvfb_pid" ]]; then
    kill "$xvfb_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT

export XDG_RUNTIME_DIR="$runtime_dir"
export KCU_X11_HEADLESS=1
export LIBGL_ALWAYS_SOFTWARE=1
unset WAYLAND_DISPLAY
Xvfb -displayfd 3 -screen 0 4096x4096x24 -nolisten tcp 3>"$display_file" &
xvfb_pid=$!
deadline=$((SECONDS + 30))
while :; do
  if ! kill -0 "$xvfb_pid" 2>/dev/null; then
    echo "Xvfb exited before its display became available" >&2
    exit 1
  fi
  if [[ -s "$display_file" ]]; then
    read -r display_number < "$display_file"
    if [[ "$display_number" =~ ^[0-9]+$ ]]; then
      export DISPLAY=":$display_number"
      if xdpyinfo -display "$DISPLAY" >/dev/null 2>&1; then
        break
      fi
    fi
  fi
  if (( SECONDS >= deadline )); then
    echo "Xvfb did not expose a working display" >&2
    exit 1
  fi
  sleep 0.1
done

bash "$(dirname "$0")/run-window-manager.sh" "$@"
