# katana-chat-ui OpenSpec

## Project

`katana-chat-ui`（kcu）は、vendor 非依存の LLM chat UI と ACP (Agent Client Protocol) client を提供する library。KatanA はこれを git dependency として consume するだけ。ACP 実装・vendor adapter・chat widget・settings 管理はすべてここで行う。

## Design Principles

- KatanA の public interface に egui 型・vendor SDK を漏らさない。KatanA が見るのは `ChatPanel` builder API と settings 統合の 2 点のみ。
- `katana-acp-client` は `egui` に依存しない。widget と protocol は分離する。
- vendor 固有 UI 分岐を widget 内に持たない。差分は ACP capability negotiation で表現する。
- settings は JSON Schema で管理し、path 渡し（デフォルト）またはコールバック（IDE 統合）で host と統合する。

## Versioning

- `v0.1.x`: ACP client (stdio transport + Ollama adapter) + chat widget + settings schema + autofix diff surface。KatanA v0.23.0 がこれを取り込む。
- `v0.2.x`: 追加 vendor adapter（OpenAI 互換、Vertex AI 等）
- `v0.3.x`: 履歴永続化、複数会話管理

## Consumers

- [KatanA](https://github.com/HiroyukiFuruno/KatanA) — git tag pinned dependency（v0.23.0 で取り込み）
