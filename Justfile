check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

ast-lint:
    @echo "Running AST-based custom lint checks..."
    # Check for misuse of unwrap/expect in production code
    cargo clippy --workspace --all-targets -- -D clippy::disallowed_methods
    # Check for missing HTTP status check in reqwest calls (fallback for complex chains)
    @! rg -z --pcre2 "\.send\(\)\s*\.await(\s*\.map_err\(.*?\))?\?\s*\.json\(\)" crates/*/src/*.rs
    # Check for missing context usage in execute
    @grep -q "request.context" crates/katana-acp-client/src/ollama.rs

test: check ast-lint
