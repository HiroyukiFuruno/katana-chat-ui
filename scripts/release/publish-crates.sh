#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/packages.sh"

version="$(bash "$(dirname "$0")/verify-version.sh" "${1:-}" | awk -F= '$1 == "version_bare" { print $2 }')"

if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
  echo "CARGO_REGISTRY_TOKEN is required." >&2
  exit 1
fi

publish_if_needed() {
  local package="$1"
  if cargo info "${package}@${version}" --registry crates-io >/dev/null 2>&1; then
    echo "${package} ${version} already published; skipping."
    return
  fi
  cargo publish -p "${package}" --locked --token "${CARGO_REGISTRY_TOKEN}"
}

wait_for_crate() {
  local package="$1"
  for _ in {1..30}; do
    if cargo info "${package}@${version}" --registry crates-io >/dev/null 2>&1; then
      return
    fi
    sleep 10
  done
  echo "${package} ${version} did not become visible on crates.io in time." >&2
  exit 1
}

for package in "${release_packages[@]}"; do
  publish_if_needed "${package}"
  wait_for_crate "${package}"
done
