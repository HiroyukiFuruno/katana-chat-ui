check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

ast-lint:
    @echo "Running AST-based custom lint checks..."
    # Check for misuse of unwrap/expect in production code
    cargo clippy --workspace --all-targets -- -D clippy::disallowed_methods
    # Check for misuse of unwrap_or_default in production code (recursive)
    @! grep -r "unwrap_or_default" crates
    # Check for missing HTTP status check in reqwest calls (recursive, cross-line)
    # Detects .send() followed by any chain (including whitespace/newlines) that reaches .json() without error_for_status()
    @! find crates -name "*.rs" -exec grep -Pz "\.send\(\)(?:(?!\.error_for_status\(\)).)*?\.json\(\)" {} +
    # Check for missing context usage in execute
    @grep -q "request.context" crates/katana-acp-client/src/ollama.rs

test: check ast-lint
