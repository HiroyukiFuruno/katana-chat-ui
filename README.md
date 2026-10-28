<h1 align="center">katana-chat-ui</h1>

<p align="center">
  Host-agnostic AI chat UI foundation with provider-neutral integration contracts.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/status-scaffolding-orange" alt="Status: scaffolding">
</p>

---

## Status

Published v0.1.0 provides the framework-neutral chat state, render model, input
draft, attachment contract, safe Markdown subset, theme tokens, SVG icon override,
i18n-ready text catalog, vendor UI capability surface, and output handoff contract.

The working branch prepares v0.2.1 with public KUC 0.6 and the shared egui,
Floem, and GPUI adapters. `ChatUiTree::tree()` exposes a KUC type, so consumers
must use the matching KUC 0.6 dependency. This version is not published yet;
current-source checks and final public-package acceptance are tracked separately
in the [requirements audit](docs/release-requirements-audit.md).

## Why

LLM vendors keep diverging on UI affordances (tool calls, citations, file
attachment, streaming token shapes, account usage, permissions, and thinking
levels). Embedding per-vendor UI inside each host application would force
re-work each time a new vendor is added.

This repository keeps the chat surface and the agent transport behind neutral
contracts. The working tree includes an ACP v1 stdio transport backed by the
official Rust SDK, provider-neutral types, and a direct Ollama provider.
The transport negotiates capabilities and supports session prompts, streaming,
model configuration, cancellation, and permission responses. Full provider and
host acceptance verification is still in progress. Providers without ACP support use a secure direct
connector boundary that does not store secrets in plain settings files.

## Crates

- `katana-acp-client` — provider-neutral client types, ACP v1 stdio transport,
  explicit content encoding, and a direct Ollama provider. No UI dependency.
- `katana-chat-ui` — framework-neutral chat state, input contract, Markdown
  subset, theme/icon tokens, i18n text catalog, vendor UI capability surface,
  usage surface, output handoff contract, and render model. No UI framework
  dependency.
- `katana-chat-ui-egui` — egui adapter for the shared KUC chat UI.
- `katana-chat-ui-floem` — Floem adapter for the shared KUC chat UI.
- `katana-chat-ui-gpui` — GPUI adapter for the shared KUC chat UI.
- `katana-chat-connectors` — planned ACP connector, direct connector, and
  secret store boundary.

ACP v1 compatibility status and remaining acceptance checks are
documented in [docs/acp-compatibility.md](docs/acp-compatibility.md).

## Manual Host Checks

- `just harness-up` — starts the Floem host adapter for human UI/UX checks.
- `just harness-up egui` — starts the egui host adapter.
- `just harness-up gpui` — starts the GPUI host adapter.

## Storybook

`just storybook` starts the release-build, provider-free Storybook. It uses the same public-KUC runtime as the product UI and provides deterministic Codex, Claude Code, Copilot, and KatanAgent fixtures for input, send, history, vendor selection, model selection, Markdown, and scroll review. It never starts an LLM provider or needs a paid account.

`just storybook-check` verifies the entrypoint and fixture controller contracts without opening a native window. `just storybook-window-smoke` and the full `just check` render the real Storybook in an isolated Linux/Xvfb display, leaving the desktop available for other work. Storybook supports UI exploration and visual review; automated contracts and the required test gates verify correctness. User-performed manual acceptance is waived for the unused initial release. Agent verification of real Japanese IME input continues in private Linux displays and dedicated GitHub macOS runners; current results and unresolved cases are recorded in the [requirements audit](docs/release-requirements-audit.md).

Manual LLM checks use local Ollama. Automated checks do not call paid LLM APIs.

## Known Linux IME limitations in 0.2.1

The following Linux/X11 display issues remain open and are deferred to a later
release with the user's explicit approval:

- [#7](https://github.com/HiroyukiFuruno/katana-chat-ui/issues/7): GPUI can display
  duplicated Japanese preedit text while composing. The measured committed text
  is applied once.
- [#11](https://github.com/HiroyukiFuruno/katana-chat-ui/issues/11): the egui and
  Floem candidate window can overlap the preedit text.

These are known defects, not successful visual acceptance results. The deferral
does not cover lost input, duplicate commits, accidental submission, cancellation
failures, other rendering failures, or any new failure. Input regression checks,
the existing quality gates, and background acceptance of the published app and
interactive Storybook remain required.

## Non-Scope

- Host workspace shell, document state, lint integration, and output execution.
  Hosts normally mount the standard UI crate. API-only custom rendering is
  available for advanced customization, but it is not the default integration
  path.
- Diagram rendering — see
  [`katana-diagram-renderer`](https://github.com/HiroyukiFuruno/katana-diagram-renderer).
- Document export — see
  [`katana-canvas-forge`](https://github.com/HiroyukiFuruno/katana-canvas-forge).

## License

MIT — see [LICENSE](LICENSE).
