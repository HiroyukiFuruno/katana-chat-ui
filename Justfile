set shell := ["bash", "-uc"]

REPO_ROOT := justfile_directory()
RTK := env_var_or_default("RTK", `command -v rtk 2> /dev/null || true`)
RTK_CMD := if RTK == "" { "" } else { RTK + " " }
JOBS := env_var_or_default("JOBS", "2")
export RUSTFLAGS := env_var_or_default("RUSTFLAGS", "-D warnings")
CARGO := env_var_or_default("CARGO", "cargo")
GUARDED_CARGO := "\"" + REPO_ROOT + "/scripts/maintenance/cargo-guard\""
VERSION := env_var_or_default("VERSION", `awk -F '"' '/^version = / { print $2; exit }' Cargo.toml`)
COVERAGE_MIN_LINES := env_var_or_default("COVERAGE_MIN_LINES", "77")
E2E_HOST_MANIFEST := "tools/e2e-host-app/Cargo.toml"
HOST_SUPPORT_MANIFEST := "tools/host-support/Cargo.toml"
MANUAL_EGUI_MANIFEST := "tools/manual-host-egui/Cargo.toml"
MANUAL_FLOEM_MANIFEST := "tools/manual-host-floem/Cargo.toml"
MANUAL_GPUI_MANIFEST := "tools/manual-host-gpui/Cargo.toml"
SCREENSHOT_MANIFEST := "scripts/screenshot/Cargo.toml"
SCREENSHOT_REQUEST := "scripts/screenshot/examples/standard-chat.json"
SCREENSHOT_REQUEST_DIR := "scripts/screenshot/examples"

[private]
default: help

help:
    @just --list --unsorted

init:
    @echo "Installing cargo-llvm-cov..."
    @{{RTK_CMD}}{{CARGO}} install cargo-llvm-cov
    @echo "Installing cargo-sweep..."
    @{{RTK_CMD}}{{CARGO}} install cargo-sweep
    @echo "Installing lefthook..."
    @command -v brew >/dev/null 2>&1 && brew install lefthook || echo "lefthook を手動でインストールしてください: https://github.com/evilmartians/lefthook"
    lefthook install

sweep:
    @{{RTK_CMD}}{{GUARDED_CARGO}} cleanup -- {{CARGO}} sweep --time 7

clean:
    {{RTK_CMD}}{{GUARDED_CARGO}} cleanup -- {{CARGO}} clean

update-safe:
    {{RTK_CMD}}{{CARGO}} update
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{HOST_SUPPORT_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{E2E_HOST_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{MANUAL_EGUI_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{MANUAL_FLOEM_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{MANUAL_GPUI_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{SCREENSHOT_MANIFEST}}

update:
    @{{CARGO}} upgrade --help >/dev/null 2>&1 || { echo "error: just update には cargo-edit の cargo-upgrade が必要です。"; echo "install: cargo install cargo-edit"; exit 1; }
    {{RTK_CMD}}{{CARGO}} upgrade -i
    {{RTK_CMD}}{{CARGO}} update
    {{RTK_CMD}}{{CARGO}} upgrade -i --manifest-path {{HOST_SUPPORT_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{HOST_SUPPORT_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} upgrade -i --manifest-path {{E2E_HOST_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{E2E_HOST_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} upgrade -i --manifest-path {{MANUAL_EGUI_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{MANUAL_EGUI_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} upgrade -i --manifest-path {{MANUAL_FLOEM_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{MANUAL_FLOEM_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} upgrade -i --manifest-path {{MANUAL_GPUI_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{MANUAL_GPUI_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} upgrade -i --manifest-path {{SCREENSHOT_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} update --manifest-path {{SCREENSHOT_MANIFEST}}

outdated:
    {{RTK_CMD}}{{CARGO}} outdated --workspace
    {{RTK_CMD}}{{CARGO}} outdated --manifest-path {{HOST_SUPPORT_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} outdated --manifest-path {{E2E_HOST_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} outdated --manifest-path {{MANUAL_EGUI_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} outdated --manifest-path {{MANUAL_FLOEM_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} outdated --manifest-path {{MANUAL_GPUI_MANIFEST}}
    {{RTK_CMD}}{{CARGO}} outdated --manifest-path {{SCREENSHOT_MANIFEST}}

check: check-macos check-native-linux-aarch64

# macOSで採取した画像基準を含む検査を同じOSへまとめる。
check-macos: fmt-check lint unit-test ast-lint build-cache-guard-check repository-maintenance-check host-support-check manual-ui-check storybook-check host-e2e harness-screenshot-check release-source-check

release-source-check:
    {{RTK_CMD}}bash scripts/release/test-verify-release-source.sh
    {{RTK_CMD}}python3 scripts/release/test-release-source-fixture-routing.py

# native画像基準はLinux/aarch64コンテナで検証する。
check-native-linux-aarch64: harness-native-linux-matrix-check

fmt-check:
    cargo fmt --all -- --check
    cargo fmt --manifest-path {{HOST_SUPPORT_MANIFEST}} --all -- --check
    cargo fmt --manifest-path {{E2E_HOST_MANIFEST}} --all -- --check
    cargo fmt --manifest-path {{MANUAL_EGUI_MANIFEST}} --all -- --check
    cargo fmt --manifest-path {{MANUAL_FLOEM_MANIFEST}} --all -- --check
    cargo fmt --manifest-path {{MANUAL_GPUI_MANIFEST}} --all -- --check
    cargo fmt --manifest-path {{SCREENSHOT_MANIFEST}} --all -- --check

fmt:
    cargo fmt --all
    cargo fmt --manifest-path {{HOST_SUPPORT_MANIFEST}} --all
    cargo fmt --manifest-path {{E2E_HOST_MANIFEST}} --all
    cargo fmt --manifest-path {{MANUAL_EGUI_MANIFEST}} --all
    cargo fmt --manifest-path {{MANUAL_FLOEM_MANIFEST}} --all
    cargo fmt --manifest-path {{MANUAL_GPUI_MANIFEST}} --all
    cargo fmt --manifest-path {{SCREENSHOT_MANIFEST}} --all

lint:
    {{GUARDED_CARGO}} run -- {{CARGO}} clippy -j {{JOBS}} --workspace --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity

lint-fix:
    {{RTK_CMD}}{{CARGO}} clippy -j {{JOBS}} --workspace --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity

ast-lint:
    @echo "Running AST-based custom lint checks..."
    {{GUARDED_CARGO}} run -- {{CARGO}} test -j {{JOBS}} -p kcu-linter --test ast_linter -- --nocapture

build-cache-guard-check:
    {{RTK_CMD}}python3 scripts/maintenance/test-cargo-guard.py

repository-maintenance-check:
    {{RTK_CMD}}python3 -B -m unittest scripts.hooks.test_issue_gate scripts.release.test_release_cleanup
    {{RTK_CMD}}python3 -B -m unittest discover -s scripts/security -p 'test_*.py'

docker-cache-preview:
    {{RTK_CMD}}python3 scripts/maintenance/docker-cache.py

docker-cache-remove volume:
    {{RTK_CMD}}python3 scripts/maintenance/docker-cache.py --remove {{quote(volume)}}

docker-cache-check:
    {{RTK_CMD}}python3 scripts/maintenance/test-docker-cache.py

container-tls-check:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{REPO_ROOT}}"
    output="tmp/verification/container-tls-$(date -u +%Y%m%dT%H%M%SZ)-$$"
    mkdir -p "${output}-build"
    if docker build --platform linux/arm64 --file scripts/manual-ui/ime/Dockerfile \
        --tag kcu-ime-linux-arm64:local scripts/manual-ui/ime 2>&1 | tee "${output}-build/ime-build.log"; then
        printf '0\n' > "${output}-build/ime-build.exit"
    else
        build_exit=$?
        printf '%s\n' "$build_exit" > "${output}-build/ime-build.exit"
        exit "$build_exit"
    fi
    {{RTK_CMD}}python3 scripts/security/check-images.py --output-dir "$output" \
        kcu-screenshot-linux-arm64:local kcu-ime-linux-arm64:local

host-support-check: local-provider-launcher-check
    {{GUARDED_CARGO}} run -- {{CARGO}} clippy --manifest-path {{HOST_SUPPORT_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{GUARDED_CARGO}} run -- {{CARGO}} test --manifest-path {{HOST_SUPPORT_MANIFEST}} --all-targets --locked

local-provider-launcher-check:
    {{RTK_CMD}}python3 scripts/manual-ui/test_local_provider.py

host-e2e:
    {{GUARDED_CARGO}} run -- {{CARGO}} clippy --manifest-path {{E2E_HOST_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{GUARDED_CARGO}} run -- {{CARGO}} test --manifest-path {{E2E_HOST_MANIFEST}} --all-targets --locked

harness-screenshot request=SCREENSHOT_REQUEST output="target/harness-screenshots":
    bash scripts/screenshot/run.sh --request {{request}} --output {{output}}

headless-adapter-check:
    {{GUARDED_CARGO}} run -- {{CARGO}} test -j {{JOBS}} --locked -p katana-chat-ui-egui -p katana-chat-ui-floem -p katana-chat-ui-gpui --lib

harness-screenshot-matrix-check: headless-adapter-check
    {{GUARDED_CARGO}} run -- {{CARGO}} clippy --manifest-path {{SCREENSHOT_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{GUARDED_CARGO}} run -- {{CARGO}} test --manifest-path {{SCREENSHOT_MANIFEST}} --all-targets --locked
    for request in {{SCREENSHOT_REQUEST_DIR}}/*.json; do bash scripts/screenshot/run.sh --request "$request" --output target/harness-screenshots || exit $?; done

harness-screenshot-check: harness-screenshot-matrix-check

harness-native-linux-matrix-check:
    bash scripts/screenshot/linux/run-matrix.sh
    just docker-cache-check
    just container-tls-check

harness-native-screenshot vendor="floem" output="target/native-harness-screenshots": harness-screenshot-matrix-check
    @echo "harness-native-screenshot is headless by default. Use harness-visible-native-screenshot only when visible native windows are acceptable."

harness-native-screenshot-check output="target/native-harness-screenshots": harness-screenshot-matrix-check
    bash scripts/screenshot/linux/run-matrix.sh "{{output}}"

harness-visible-native-screenshot vendor="floem" output="target/native-harness-screenshots":
    @if [[ "${KCU_ALLOW_VISIBLE_WINDOWS:-}" != "1" ]]; then echo "error: this opens visible native windows. Set KCU_ALLOW_VISIBLE_WINDOWS=1 to run it."; exit 2; fi
    bash scripts/screenshot/run.sh --native-host {{vendor}} --output {{output}}

harness-visible-native-screenshot-check output="target/native-harness-screenshots":
    @if [[ "${KCU_ALLOW_VISIBLE_WINDOWS:-}" != "1" ]]; then echo "error: this opens visible native windows. Set KCU_ALLOW_VISIBLE_WINDOWS=1 to run it."; exit 2; fi
    @if [[ "$(uname -s)" != "Darwin" ]]; then echo "native screenshot check is macOS-only; skipped"; exit 0; fi
    {{GUARDED_CARGO}} run -- {{CARGO}} clippy --manifest-path {{SCREENSHOT_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{GUARDED_CARGO}} run -- {{CARGO}} test --manifest-path {{SCREENSHOT_MANIFEST}} --all-targets --locked
    for vendor in egui floem gpui; do bash scripts/screenshot/run.sh --native-host "$vendor" --output {{output}} || exit $?; done

harness-up vendor="floem":
    @case "{{vendor}}" in \
        egui) {{GUARDED_CARGO}} run -- {{CARGO}} run --manifest-path {{MANUAL_EGUI_MANIFEST}} --locked ;; \
        floem) {{GUARDED_CARGO}} run -- {{CARGO}} run --manifest-path {{MANUAL_FLOEM_MANIFEST}} --locked ;; \
        gpui) {{GUARDED_CARGO}} run -- {{CARGO}} run --manifest-path {{MANUAL_GPUI_MANIFEST}} --locked ;; \
        *) echo "error: unknown harness vendor '{{vendor}}'. expected: egui, floem, gpui"; exit 2 ;; \
    esac

manual-ui-egui:
    @just harness-up egui

manual-ui-floem:
    @just harness-up floem

manual-ui-gpui:
    @just harness-up gpui

# 明示選択した場合だけ検証用のClaude/Ollama設定をmanual hostへ引き渡す。
manual-ui-local-claude MODE="egui":
    {{RTK_CMD}}bash scripts/manual-ui/local-provider.sh manual-ui "{{MODE}}"

# 実provider probeへ同じ隔離Claude/Ollama設定を明示的に引き渡す。
provider-probe-local-claude:
    {{RTK_CMD}}bash scripts/manual-ui/local-provider.sh probe

manual-ui-check-egui:
    {{GUARDED_CARGO}} run -- {{CARGO}} clippy --manifest-path {{MANUAL_EGUI_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{GUARDED_CARGO}} run -- {{CARGO}} check --manifest-path {{MANUAL_EGUI_MANIFEST}} --locked
    {{GUARDED_CARGO}} run -- {{CARGO}} test --manifest-path {{MANUAL_EGUI_MANIFEST}} --all-targets --locked

manual-ui-check-floem:
    {{GUARDED_CARGO}} run -- {{CARGO}} clippy --manifest-path {{MANUAL_FLOEM_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{GUARDED_CARGO}} run -- {{CARGO}} check --manifest-path {{MANUAL_FLOEM_MANIFEST}} --locked
    {{GUARDED_CARGO}} run -- {{CARGO}} test --manifest-path {{MANUAL_FLOEM_MANIFEST}} --all-targets --locked

manual-ui-check-gpui:
    {{GUARDED_CARGO}} run -- {{CARGO}} clippy --manifest-path {{MANUAL_GPUI_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{GUARDED_CARGO}} run -- {{CARGO}} check --manifest-path {{MANUAL_GPUI_MANIFEST}} --locked
    {{GUARDED_CARGO}} run -- {{CARGO}} test --manifest-path {{MANUAL_GPUI_MANIFEST}} --all-targets --locked

manual-ui: harness-up

manual-ui-check: manual-ui-check-egui manual-ui-check-floem manual-ui-check-gpui

# 開発者がKUC共通UIを実際に操作して確認する、provider-freeのStorybook。
storybook:
    {{RTK_CMD}}{{GUARDED_CARGO}} run -- {{CARGO}} run --release --locked -p kcu-storybook

# Linux/Xvfb上でStorybookの実windowを描画して終了する。
storybook-window-smoke:
    {{RTK_CMD}}bash scripts/screenshot/linux/run-matrix.sh --storybook-smoke

storybook-entrypoint-check:
    {{RTK_CMD}}bash scripts/storybook/check-entrypoint.sh

storybook-check: storybook-entrypoint-check
    {{RTK_CMD}}{{GUARDED_CARGO}} run -- {{CARGO}} test -p kcu-storybook --locked

unit-test:
    {{GUARDED_CARGO}} run -- {{CARGO}} test --workspace --all-targets --all-features

# Run coverage as a required full-check gate
coverage:
    {{GUARDED_CARGO}} run -- {{CARGO}} llvm-cov --workspace --all-features --locked --summary-only --fail-under-lines {{COVERAGE_MIN_LINES}}

# macOS側の品質検査後に公開パッケージを検証する。
release-verify-macos: check-macos coverage
    bash scripts/release/verify-version.sh "{{VERSION}}"
    bash scripts/release/verify-internal-dependencies.sh "{{VERSION}}"
    {{GUARDED_CARGO}} run -- {{CARGO}} package -p katana-acp-client --locked --allow-dirty
    {{GUARDED_CARGO}} run -- {{CARGO}} package -p katana-chat-ui --locked --allow-dirty --list >/dev/null
    {{GUARDED_CARGO}} run -- {{CARGO}} package -p katana-chat-ui-egui --locked --allow-dirty --list >/dev/null
    {{GUARDED_CARGO}} run -- {{CARGO}} package -p katana-chat-ui-floem --locked --allow-dirty --list >/dev/null
    {{GUARDED_CARGO}} run -- {{CARGO}} package -p katana-chat-ui-gpui --locked --allow-dirty --list >/dev/null
    {{GUARDED_CARGO}} run -- {{CARGO}} publish -p katana-acp-client --dry-run --locked --allow-dirty

# native画像検査をCIの専用runnerから呼び出せるようにする。
release-verify-native-linux-aarch64: check-native-linux-aarch64

# ローカル入口でもCIの両OSの検査を必須にする。
release-verify: release-verify-macos release-verify-native-linux-aarch64

# Verify release branch readiness before merging
release-check: release-verify
    bash scripts/release/assert-crates-not-published.sh "{{VERSION}}"

# 公開済み版の再利用をマージ前のmacOS側ジョブで拒否する。
release-check-macos: release-verify-macos
    bash scripts/release/assert-crates-not-published.sh "{{VERSION}}"

test: check
