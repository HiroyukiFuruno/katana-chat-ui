check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

ast-lint:
    @echo "Running AST-based custom lint checks..."
    # Check for misuse of unwrap/expect in production code
    cargo clippy --workspace --all-targets -- -D clippy::disallowed_methods
    # Check for misuse of unwrap_or_default in production code
    @! rg -r "unwrap_or_default" crates/*/src/*.rs
    # Check for missing HTTP status check in reqwest calls
    # Improved: Detects .send() followed by any chain that reaches .json() without error_for_status()
    @! rg -z --pcre2 "\.send\(\)(?:(?!\.error_for_status\(\)).)*?\.json\(\)" crates/*/src/*.rs
    # Check for missing context usage in execute
    @grep -q "request.context" crates/katana-acp-client/src/ollama.rs

test: check ast-lint
