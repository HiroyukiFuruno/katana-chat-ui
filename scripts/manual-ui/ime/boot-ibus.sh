#!/usr/bin/env bash
set -euo pipefail

# ユーザーの入力ソースと分離した、container内だけの日本語IMEを起動する。
mkdir -p "$HOME/.config/mozc"
cat > "$HOME/.config/mozc/ibus_config.textproto" <<'CONFIG'
engines {
  name: "mozc-jp"
  longname: "Mozc"
  layout: "us"
  rank: 80
}
active_on_launch: True
CONFIG
cp "$HOME/.config/mozc/ibus_config.textproto" /artifacts/ibus_config.textproto
ibus write-cache
gsettings set org.freedesktop.ibus.general preload-engines "['mozc-jp']"
gsettings set org.freedesktop.ibus.general use-global-engine true
ibus-daemon --replace --daemonize --xim
deadline=$((SECONDS + 30))
while ! ibus list-engine 2>/dev/null | grep mozc-jp >/dev/null; do
  if (( SECONDS >= deadline )); then
    echo 'Mozc did not become available on the private IBus session' >&2
    exit 1
  fi
  sleep 0.1
done
/usr/lib/ibus-mozc/ibus-engine-mozc --ibus > /artifacts/mozc-engine.log 2>&1 &
ibus engine mozc-jp || true
while [[ "$(ibus engine 2>/dev/null)" != mozc-jp ]]; do
  if (( SECONDS >= deadline )); then
    echo 'Mozc engine activation did not complete' >&2
    exit 1
  fi
  sleep 0.1
done
ibus address > /artifacts/ibus-address.txt
ibus list-engine > /artifacts/ibus-engines.txt
exec /usr/bin/python3 "${KCU_IME_DRIVER:-/workspace/scripts/manual-ui/ime/probe.py}"
