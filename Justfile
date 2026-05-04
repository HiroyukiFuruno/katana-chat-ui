check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

ast-lint:
    @echo "Running AST-based custom lint checks..."
    # Check for misuse of unwrap_or_default in production code
    @! grep -r "unwrap_or_default" crates/*/src/*.rs
    # Check for missing HTTP status check in reqwest calls
    # (This is a naive check for .send().await without status/error check)
    @! grep -r "\.send().await;" crates/*/src/*.rs
    # Check for missing context usage in execute
    @grep -q "request.context" crates/katana-acp-client/src/ollama.rs

test: check ast-lint
