set shell := ["bash", "-uc"]

JOBS := env_var_or_default("JOBS", "2")
export RUSTFLAGS := env_var_or_default("RUSTFLAGS", "-D warnings")
CARGO := env_var_or_default("CARGO", "cargo")
VERSION := env_var_or_default("VERSION", `awk -F '"' '/^version = / { print $2; exit }' Cargo.toml`)
COVERAGE_MIN_LINES := env_var_or_default("COVERAGE_MIN_LINES", "77")
E2E_HOST_MANIFEST := "tools/e2e-host-app/Cargo.toml"
MANUAL_EGUI_MANIFEST := "tools/manual-host-egui/Cargo.toml"
MANUAL_FLOEM_MANIFEST := "tools/manual-host-floem/Cargo.toml"
MANUAL_GPUI_MANIFEST := "tools/manual-host-gpui/Cargo.toml"
SCREENSHOT_MANIFEST := "scripts/screenshot/Cargo.toml"
SCREENSHOT_REQUEST := "scripts/screenshot/examples/standard-chat.json"
SCREENSHOT_REQUEST_DIR := "scripts/screenshot/examples"

update-safe:
    {{CARGO}} update
    {{CARGO}} update --manifest-path {{E2E_HOST_MANIFEST}}
    {{CARGO}} update --manifest-path {{MANUAL_EGUI_MANIFEST}}
    {{CARGO}} update --manifest-path {{MANUAL_FLOEM_MANIFEST}}
    {{CARGO}} update --manifest-path {{MANUAL_GPUI_MANIFEST}}
    {{CARGO}} update --manifest-path {{SCREENSHOT_MANIFEST}}

update:
    @{{CARGO}} upgrade --help >/dev/null 2>&1 || { echo "error: just update には cargo-edit の cargo-upgrade が必要です。"; echo "install: cargo install cargo-edit"; exit 1; }
    {{CARGO}} upgrade -i
    {{CARGO}} update
    {{CARGO}} upgrade -i --manifest-path {{E2E_HOST_MANIFEST}}
    {{CARGO}} update --manifest-path {{E2E_HOST_MANIFEST}}
    {{CARGO}} upgrade -i --manifest-path {{MANUAL_EGUI_MANIFEST}}
    {{CARGO}} update --manifest-path {{MANUAL_EGUI_MANIFEST}}
    {{CARGO}} upgrade -i --manifest-path {{MANUAL_FLOEM_MANIFEST}}
    {{CARGO}} update --manifest-path {{MANUAL_FLOEM_MANIFEST}}
    {{CARGO}} upgrade -i --manifest-path {{MANUAL_GPUI_MANIFEST}}
    {{CARGO}} update --manifest-path {{MANUAL_GPUI_MANIFEST}}
    {{CARGO}} upgrade -i --manifest-path {{SCREENSHOT_MANIFEST}}
    {{CARGO}} update --manifest-path {{SCREENSHOT_MANIFEST}}

check: fmt-check lint unit-test ast-lint manual-ui-check host-e2e harness-screenshot-check

fmt-check:
    cargo fmt --all -- --check
    cargo fmt --manifest-path {{E2E_HOST_MANIFEST}} --all -- --check
    cargo fmt --manifest-path {{MANUAL_EGUI_MANIFEST}} --all -- --check
    cargo fmt --manifest-path {{MANUAL_FLOEM_MANIFEST}} --all -- --check
    cargo fmt --manifest-path {{MANUAL_GPUI_MANIFEST}} --all -- --check
    cargo fmt --manifest-path {{SCREENSHOT_MANIFEST}} --all -- --check

fmt:
    cargo fmt --all
    cargo fmt --manifest-path {{E2E_HOST_MANIFEST}} --all
    cargo fmt --manifest-path {{MANUAL_EGUI_MANIFEST}} --all
    cargo fmt --manifest-path {{MANUAL_FLOEM_MANIFEST}} --all
    cargo fmt --manifest-path {{MANUAL_GPUI_MANIFEST}} --all
    cargo fmt --manifest-path {{SCREENSHOT_MANIFEST}} --all

lint:
    {{CARGO}} clippy -j {{JOBS}} --workspace --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity

ast-lint:
    @echo "Running AST-based custom lint checks..."
    {{CARGO}} test -j {{JOBS}} -p kcu-linter ast_linter -- --nocapture

host-e2e:
    {{CARGO}} clippy --manifest-path {{E2E_HOST_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{CARGO}} test --manifest-path {{E2E_HOST_MANIFEST}} --all-targets --locked

harness-screenshot request=SCREENSHOT_REQUEST output="target/harness-screenshots":
    bash scripts/screenshot/run.sh --request {{request}} --output {{output}}

harness-screenshot-matrix-check:
    {{CARGO}} clippy --manifest-path {{SCREENSHOT_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{CARGO}} test --manifest-path {{SCREENSHOT_MANIFEST}} --all-targets --locked
    for request in {{SCREENSHOT_REQUEST_DIR}}/*.json; do bash scripts/screenshot/run.sh --request "$request" --output target/harness-screenshots; done

harness-screenshot-check: harness-screenshot-matrix-check

harness-native-screenshot vendor="floem" output="target/native-harness-screenshots": harness-screenshot-matrix-check
    @echo "harness-native-screenshot is headless by default. Use harness-visible-native-screenshot only when visible native windows are acceptable."

harness-native-screenshot-check output="target/native-harness-screenshots": harness-screenshot-matrix-check
    @echo "harness-native-screenshot-check is headless by default. Use harness-visible-native-screenshot-check only when visible native windows are acceptable."

harness-visible-native-screenshot vendor="floem" output="target/native-harness-screenshots":
    @if [[ "${KCU_ALLOW_VISIBLE_WINDOWS:-}" != "1" ]]; then echo "error: this opens visible native windows. Set KCU_ALLOW_VISIBLE_WINDOWS=1 to run it."; exit 2; fi
    bash scripts/screenshot/run.sh --native-host {{vendor}} --output {{output}}

harness-visible-native-screenshot-check output="target/native-harness-screenshots":
    @if [[ "${KCU_ALLOW_VISIBLE_WINDOWS:-}" != "1" ]]; then echo "error: this opens visible native windows. Set KCU_ALLOW_VISIBLE_WINDOWS=1 to run it."; exit 2; fi
    @if [[ "$(uname -s)" != "Darwin" ]]; then echo "native screenshot check is macOS-only; skipped"; exit 0; fi
    {{CARGO}} clippy --manifest-path {{SCREENSHOT_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{CARGO}} test --manifest-path {{SCREENSHOT_MANIFEST}} --all-targets --locked
    for vendor in egui floem gpui; do bash scripts/screenshot/run.sh --native-host "$vendor" --output {{output}}; done

harness-up vendor="floem":
    @case "{{vendor}}" in \
        egui) {{CARGO}} run --manifest-path {{MANUAL_EGUI_MANIFEST}} --locked ;; \
        floem) {{CARGO}} run --manifest-path {{MANUAL_FLOEM_MANIFEST}} --locked ;; \
        gpui) {{CARGO}} run --manifest-path {{MANUAL_GPUI_MANIFEST}} --locked ;; \
        *) echo "error: unknown harness vendor '{{vendor}}'. expected: egui, floem, gpui"; exit 2 ;; \
    esac

manual-ui-egui:
    @just harness-up egui

manual-ui-floem:
    @just harness-up floem

manual-ui-gpui:
    @just harness-up gpui

manual-ui-check-egui:
    {{CARGO}} clippy --manifest-path {{MANUAL_EGUI_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{CARGO}} check --manifest-path {{MANUAL_EGUI_MANIFEST}} --locked
    {{CARGO}} test --manifest-path {{MANUAL_EGUI_MANIFEST}} --all-targets --locked

manual-ui-check-floem:
    {{CARGO}} clippy --manifest-path {{MANUAL_FLOEM_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{CARGO}} check --manifest-path {{MANUAL_FLOEM_MANIFEST}} --locked
    {{CARGO}} test --manifest-path {{MANUAL_FLOEM_MANIFEST}} --all-targets --locked

manual-ui-check-gpui:
    {{CARGO}} clippy --manifest-path {{MANUAL_GPUI_MANIFEST}} --all-targets --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports -D clippy::too_many_lines -D clippy::cognitive_complexity
    {{CARGO}} check --manifest-path {{MANUAL_GPUI_MANIFEST}} --locked
    {{CARGO}} test --manifest-path {{MANUAL_GPUI_MANIFEST}} --all-targets --locked

manual-ui: harness-up

manual-ui-check: manual-ui-check-egui manual-ui-check-floem manual-ui-check-gpui

unit-test:
    {{CARGO}} test --workspace --all-targets --all-features

# Run coverage as a required full-check gate
coverage:
    {{CARGO}} llvm-cov --workspace --all-features --locked --summary-only --fail-under-lines {{COVERAGE_MIN_LINES}}

# Verify package metadata and dry-run the first publishable crate
release-verify: check coverage
    bash scripts/release/verify-version.sh "{{VERSION}}"
    bash scripts/release/verify-internal-dependencies.sh "{{VERSION}}"
    {{CARGO}} package -p katana-acp-client --locked --allow-dirty
    {{CARGO}} package -p katana-chat-ui --locked --allow-dirty --list >/dev/null
    {{CARGO}} package -p katana-chat-ui-floem --locked --allow-dirty --list >/dev/null
    {{CARGO}} publish -p katana-acp-client --dry-run --locked --allow-dirty

# Verify release branch readiness before merging
release-check: release-verify
    bash scripts/release/assert-crates-not-published.sh "{{VERSION}}"

test: check
