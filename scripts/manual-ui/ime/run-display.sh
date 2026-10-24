#!/usr/bin/env bash
set -euo pipefail

runtime_dir=$(mktemp -d)
export XDG_RUNTIME_DIR="$runtime_dir"
export KCU_X11_HEADLESS=1
export LIBGL_ALWAYS_SOFTWARE=1
unset WAYLAND_DISPLAY
if [[ "$(locale charmap)" != UTF-8 ]]; then
  echo 'The real Japanese IME probe requires a UTF-8 locale for both daemon and clients' >&2
  exit 2
fi
locale > /artifacts/locale.txt
xvfb_pid=""
wm_pid=""
cleanup() {
  # 専用container内の自分のdisplayだけを終了し、IMEの孤児processで待ち続けない。
  for pid in "$wm_pid" "$xvfb_pid"; do
    if [[ -n "$pid" ]]; then
      kill -KILL "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
}
trap cleanup EXIT
Xvfb -displayfd 3 -screen 0 4096x4096x24 -nolisten tcp 3>"$runtime_dir/display" &
xvfb_pid=$!
deadline=$((SECONDS + 30))
while :; do
  if ! kill -0 "$xvfb_pid" 2>/dev/null; then exit 1; fi
  if [[ -s "$runtime_dir/display" ]]; then
    read -r number < "$runtime_dir/display"
    export DISPLAY=":$number"
    if xdpyinfo >/dev/null 2>&1; then break; fi
  fi
  if (( SECONDS >= deadline )); then exit 1; fi
  sleep 0.1
done
openbox --sm-disable > "$runtime_dir/openbox.log" 2>&1 &
wm_pid=$!
while ! xprop -root _NET_SUPPORTING_WM_CHECK | grep 'window id' >/dev/null; do
  if ! kill -0 "$wm_pid" 2>/dev/null || (( SECONDS >= deadline )); then exit 1; fi
  sleep 0.1
done
dbus-run-session -- bash /workspace/scripts/manual-ui/ime/boot-ibus.sh
