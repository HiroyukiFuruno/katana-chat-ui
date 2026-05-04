check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

ast-lint:
    @echo "Running AST-based custom lint checks..."
    # Check for misuse of unwrap_or_default in production code
    @! grep -r "unwrap_or_default" crates/*/src/*.rs
    # Check for missing HTTP status check in reqwest calls
    # (Checking for .send() followed by .await and then .json() without error_for_status() in between)
    @! rg -z --pcre2 "\.send\(\)\s*\.await(\s*\.map_err\(.*?\))?\?\s*\.json\(\)" crates/*/src/*.rs
    # Check for missing context usage in execute
    @grep -q "request.context" crates/katana-acp-client/src/ollama.rs

test: check ast-lint
