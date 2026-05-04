check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

ast-lint:
    @echo "Running AST-based custom lint checks..."
    # Check for misuse of unwrap/expect in production code
    cargo clippy --workspace --all-targets -- -D clippy::disallowed_methods
    # Check for misuse of unwrap_or_default in production code (recursive)
    @! rg -r "unwrap_or_default" crates
    # Check for missing HTTP status check in reqwest calls (recursive, cross-line)
    # Detects .send() followed by any chain (including whitespace/newlines) that reaches .json() without error_for_status()
    @! rg -U -z --pcre2 "\.send\(\)\s*(?:\.await\??\s*)?(?:\.(?!error_for_status\(\))\w+\(.*\)\s*)*\.json\(\)" crates
    # Check for missing context usage in execute
    @rg -q "request.context" crates/katana-acp-client/src/ollama.rs

test: check ast-lint
