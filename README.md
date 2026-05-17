<h1 align="center">katana-chat-ui</h1>

<p align="center">
  Host-agnostic AI chat UI foundation and Agent Client Protocol (ACP) client.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/status-scaffolding-orange" alt="Status: scaffolding">
</p>

---

## Status

v0.1.0 prepares the framework-neutral chat state, render model, input draft,
attachment contract, safe Markdown subset, theme tokens, SVG icon override,
i18n-ready text catalog, vendor UI capability surface, output handoff contract,
and the first standard Floem chat UI widget.

## Why

LLM vendors keep diverging on UI affordances (tool calls, citations, file
attachment, streaming token shapes, account usage, permissions, and thinking
levels). Embedding per-vendor UI inside each host application would force
re-work each time a new vendor is added.

This repository keeps the chat surface and the agent transport behind neutral
contracts. ACP-compatible agents are preferred. Providers without ACP support
use a secure direct connector boundary that does not store secrets in plain
settings files.

## Crates

- `katana-acp-client` — ACP and provider-neutral client types. No UI
  dependency.
- `katana-chat-ui` — framework-neutral chat state, input contract, Markdown
  subset, theme/icon tokens, i18n text catalog, vendor UI capability surface,
  usage surface, output handoff contract, and render model. No UI framework
  dependency.
- `katana-chat-ui-floem` — standard Floem chat UI widget provided by kcu.
- `katana-chat-connectors` — planned ACP connector, direct connector, and
  secret store boundary.

## Manual Host Checks

- `just harness-up` — starts the Floem host adapter for human UI/UX checks.
- `just harness-up egui` — starts the egui host adapter.
- `just harness-up gpui` — starts the GPUI host adapter.

Manual LLM checks use local Ollama. Automated checks do not call paid LLM APIs.

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
