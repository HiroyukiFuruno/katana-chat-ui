# katana-chat-ui Rust Coding Rules

> [!IMPORTANT]
> When this file is updated, update `docs/coding-rules.ja.md` at the same time.

This document transfers the reusable `katana` coding rules to `katana-chat-ui`.
`katana-chat-ui` ships UI components, so the same quality bar applies to manual verification code and small host examples.

## 1. Structure And Responsibility

- Prefer `struct` plus `impl` for public APIs.
- Public free functions are prohibited.
- Private helpers are allowed, but move growing responsibilities into types.
- Do not mix multiple responsibilities into one type, one file, or one function.

## 2. Size Limits

- Keep one file near 150 lines and never above 200 lines.
- Keep one function at or below 30 lines.
- Split by responsibility such as state, view, transform, or validation. Do not split mechanically by line count.

## 3. Nesting And Error First

- Keep nesting at 3 levels maximum, ideally 2.
- Do not wrap success paths with `if let Ok(...)`.
- Prefer `?`, `let ... else`, and early returns for failure paths.

## 4. Type Safety

- Do not use `unwrap`, `expect`, or `unwrap_or_default`.
- Do not leave `todo!`, `unimplemented!`, or `dbg!`.
- Do not use `Box<dyn std::any::Any>`, `HashMap<String, serde_json::Value>`, or unclear `serde_json::Value`.
- Do not wrap required values in `Option`.

## 5. Comments

- Comments are only for the reason, not for restating code.
- Plain `//` comments are prohibited.
- Use forms such as `/* WHY: ... */`, `/// SAFETY: ...`, or `/// TODO: ...` so the purpose is visible first.

## 6. Magic Numbers

- Numeric literals other than `0`, `1`, `2`, `100`, and `-1` must be named constants.
- UI width, height, spacing, and thresholds are not exceptions.

## 7. Process Spawning

- Do not call `std::process::Command::new` directly.
- If process spawning becomes necessary, isolate it behind a dedicated boundary type.

## 8. Dependency Update Entrypoints

After pulling upstream changes, use this entrypoint before moving to verification when dependencies must be refreshed:

```bash
just update
```

- `just update-safe` updates `Cargo.lock` within the SemVer ranges already declared in `Cargo.toml`.
- `just update` runs `cargo upgrade -i` to accept incompatible dependency updates, then runs `cargo update`.
- The same entrypoint updates lock files for `tools/e2e-host-app` and `tools/manual-host-*` so locked verification hosts keep working.
- When `cargo-upgrade` is unavailable, the command fails with installation guidance. It must not silently skip the upgrade step.

## 9. Check Entrypoints

```bash
just fmt-check
just lint
just ast-lint
```

Treat these commands as design quality gates, not as checks to game.
Do not loosen checks without asking the user first.

## 10. lefthook

- Run `lefthook install` to enable local hooks.
- `pre-commit` runs `just fmt-check`, `just lint`, and `just ast-lint` in order.
- `pre-push` and `pre-pr` run `just release-verify`.
- Do not bypass hooks with `--no-verify`.
