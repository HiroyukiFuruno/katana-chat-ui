#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
config_file="${KCU_LOCAL_PROVIDER_ENV:-${repo_root}/.codex/local-provider.env}"
action="${1:-}"

if [[ ! -r "$config_file" ]]; then
  echo "error: local Claude/Ollama config is missing or unreadable: $config_file" >&2
  echo "Create .codex/local-provider.env with the approved local settings before using this opt-in launcher." >&2
  exit 1
fi

case "$action" in
  manual-ui)
    if [[ $# -ne 2 ]]; then
      echo "usage: local-provider.sh manual-ui [egui|floem|gpui]" >&2
      exit 2
    fi
    host="$2"
    case "$host" in
      egui|floem|gpui) ;;
      *)
        echo "error: unsupported manual host '$host'; expected egui, floem, or gpui" >&2
        exit 2
        ;;
    esac
    ;;
  probe)
    if [[ $# -ne 1 ]]; then
      echo "usage: local-provider.sh probe" >&2
      exit 2
    fi
    ;;
  *)
    echo "usage: local-provider.sh [manual-ui [egui|floem|gpui]|probe]" >&2
    exit 2
    ;;
esac

# このファイルは明示的なlocal launcherからだけ読み、子processへ値を渡す。
set -a
source "$config_file"
set +a

for name in KCU_CLAUDE_OLLAMA_MODEL KCU_OLLAMA_ENDPOINT CLAUDE_CONFIG_DIR; do
  if [[ -z "${!name:-}" ]]; then
    echo "error: $name must be set in $config_file" >&2
    exit 1
  fi
done

case "$KCU_OLLAMA_ENDPOINT" in
  http://127.0.0.1:11434|http://localhost:11434) ;;
  *)
    echo "error: local Claude launcher only permits the local Ollama endpoint" >&2
    exit 1
    ;;
esac

settings_file="${CLAUDE_CONFIG_DIR}/settings.json"
if [[ ! -r "$settings_file" ]] || ! python3 - "$settings_file" <<'PY'
import json
import sys

try:
    with open(sys.argv[1], encoding="utf-8") as settings_file:
        settings = json.load(settings_file)
except (OSError, json.JSONDecodeError):
    raise SystemExit(1)

if not isinstance(settings, dict):
    raise SystemExit(1)
permissions = settings.get("permissions")
raise SystemExit(0 if isinstance(permissions, dict) and permissions.get("defaultMode") == "default" else 1)
PY
then
  echo "error: Claude config must contain permissions.defaultMode=default: $settings_file" >&2
  exit 1
fi

case "$action" in
  manual-ui)
    exec just "manual-ui-${host}"
    ;;
  probe)
    exec cargo run --locked \
      --manifest-path "${repo_root}/tools/host-support/Cargo.toml" \
      --bin acp_provider_probe -- claude-code
    ;;
esac
