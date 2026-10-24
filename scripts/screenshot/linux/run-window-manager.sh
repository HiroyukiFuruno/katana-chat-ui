#!/usr/bin/env bash
set -euo pipefail

# GPUIの初期surfaceを確定するConfigureNotifyを通常のWM経路から受け取る。
openbox --sm-disable > "$XDG_RUNTIME_DIR/openbox.log" 2>&1 &
wm_pid=$!
cleanup() {
  kill "$wm_pid" 2>/dev/null || true
  wait "$wm_pid" 2>/dev/null || true
}
trap cleanup EXIT

deadline=$((SECONDS + 30))
while ! xprop -root _NET_SUPPORTING_WM_CHECK | grep -q 'window id'; do
  if ! kill -0 "$wm_pid" 2>/dev/null; then
    cat "$XDG_RUNTIME_DIR/openbox.log" >&2
    exit 1
  fi
  if (( SECONDS >= deadline )); then
    echo "Openbox did not become ready" >&2
    exit 1
  fi
  sleep 0.1
done

"$@"
