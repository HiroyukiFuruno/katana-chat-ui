<h1 align="center">katana-chat-ui</h1>

<p align="center">
  Vendor-neutral chat UI and Agent Client Protocol (ACP) client for
  <a href="https://github.com/HiroyukiFuruno/KatanA">KatanA</a> and other
  egui-based hosts.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/status-scaffolding-orange" alt="Status: scaffolding">
</p>

---

## Status

Scaffolding. The ACP client transport, capability negotiation, and chat
widget state model are migrated/implemented during the
[`v0.22.14`](https://github.com/HiroyukiFuruno/KatanA/tree/master/openspec/changes/v0-22-14-llm-acp-and-chat-ui-extraction)
change.

## Why

LLM vendors keep diverging on UI affordances (tool calls, citations, file
attachment, streaming token shapes, etc.). Embedding per-vendor UI inside
KatanA would explode complexity and force re-work each time a new vendor is
added.

This repository keeps the chat surface and the agent transport behind a
single neutral contract — Agent Client Protocol (ACP), the same protocol
adopted by Zed, VS Code, and JetBrains products — so KatanA only consumes
a stable widget and one client trait.

## Crates

- `katana-acp-client` — ACP client library. Vendor-neutral types, transport
  (stdio / WebSocket), capability negotiation. No UI dependency.
- `katana-chat-ui` — egui chat side-panel widget that talks to any
  `AcpClient`. KatanA hosts this widget; per-vendor variation lives in the
  ACP server side.

## Non-Scope

- KatanA workspace shell, document state, lint integration. Those live in
  KatanA and consume this crate as a library.
- Diagram rendering / document export — see
  [`katana-canvas-forge`](https://github.com/HiroyukiFuruno/katana-canvas-forge).

## License

MIT — see [LICENSE](LICENSE).
