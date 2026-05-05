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

Scaffolding. The active OpenSpec changes define the framework-neutral chat
state, ACP connection contract, secure direct connector boundary, and vendor
capability model.

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
- `katana-chat-ui` — framework-neutral chat state and render model. No UI
  framework dependency.
- `katana-chat-ui-floem` — planned reference UI implementation.
- `katana-chat-connectors` — planned ACP connector, direct connector, and
  secret store boundary.

## Non-Scope

- Host workspace shell, document state, lint integration, and UI-framework
  adapters. Hosts consume kcu as a library and map its render model into their
  own UI.
- Diagram rendering / document export — see
  [`katana-canvas-forge`](https://github.com/HiroyukiFuruno/katana-canvas-forge).

## License

MIT — see [LICENSE](LICENSE).
