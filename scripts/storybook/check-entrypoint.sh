#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

recipe="$(just --dry-run storybook 2>&1)"
smoke_recipe="$(just --dry-run storybook-window-smoke 2>&1)"
host_check_recipe="$(just --dry-run storybook-check 2>&1)"
linux_checks="${repo_root}/scripts/screenshot/linux/run-checks.sh"
linux_checks_contents="$(<"$linux_checks")"

case "$recipe" in
  *"cargo run --release --locked -p kcu-storybook"*) ;;
  *)
    echo "error: just storybook must launch the release KCU Storybook" >&2
    exit 1
    ;;
esac

case "$smoke_recipe" in
  *"bash scripts/screenshot/linux/run-matrix.sh --storybook-smoke"*) ;;
  *)
    echo "error: Storybook smoke must run in the isolated Linux/Xvfb container" >&2
    exit 1
    ;;
esac

case "$host_check_recipe" in
  *"cargo test -p kcu-storybook --locked"*) ;;
  *)
    echo "error: storybook-check must run the Storybook package tests on the host" >&2
    exit 1
    ;;
esac

case "$host_check_recipe" in
  *"storybook-window-smoke"*)
    echo "error: storybook-check must not launch the native smoke on the host" >&2
    exit 1
    ;;
esac

case "$linux_checks_contents" in
  *'if [[ "$mode" == "storybook-smoke" ]]; then'*'exec bash scripts/screenshot/linux/run-xvfb.sh'*'cargo test --locked -p katana-chat-ui --lib'*'python3 scripts/screenshot/linux/matrix.py'*'--output /artifacts'*'bash scripts/screenshot/linux/run-xvfb.sh'*'cargo run --release --locked -p kcu-storybook -- --smoke'*) ;;
  *)
    echo "error: the default Linux matrix must render the Storybook smoke after existing checks" >&2
    exit 1
    ;;
esac
