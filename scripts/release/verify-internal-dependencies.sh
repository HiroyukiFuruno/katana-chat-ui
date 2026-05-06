#!/usr/bin/env bash
set -euo pipefail

version="$(bash "$(dirname "$0")/verify-version.sh" "${1:-}" | awk -F= '$1 == "version_bare" { print $2 }')"

dependency_line="$(grep '^katana-acp-client = ' crates/katana-chat-ui/Cargo.toml)"
if [[ "${dependency_line}" != *"version = \"${version}\""* ]]; then
  echo "katana-chat-ui must depend on katana-acp-client version ${version}" >&2
  exit 1
fi

floem_dependency_line="$(grep '^katana-chat-ui = ' crates/katana-chat-ui-floem/Cargo.toml)"
if [[ "${floem_dependency_line}" != *"version = \"${version}\""* ]]; then
  echo "katana-chat-ui-floem must depend on katana-chat-ui version ${version}" >&2
  exit 1
fi

echo "internal dependency versions match ${version}"
