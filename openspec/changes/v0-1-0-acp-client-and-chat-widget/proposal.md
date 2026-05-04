## Why

KatanA v0.23.0 で chat / LLM 実装を直接 `katana-core/src/ai/` と `katana-ui/src/app/chat.rs` に書いた実装は技術的負債。KatanA が肥大化し、リリースごとの検証範囲が増え続ける。これを解消するため、chat UI と ACP client を katana-chat-ui v0.1.0 として独立させ、KatanA v0.23.0 はこれを git dependency として取り込むだけにする。

## Migration from KatanA

以下を KatanA から移管する。

- `crates/katana-core/src/ai/` （Ollama provider、registry、types）
- `crates/katana-ui/src/app/chat.rs`、`autofix.rs`、`autofix_request.rs`、`autofix_support.rs`
- `crates/katana-platform/src/settings/types/ai.rs`（AI settings types）

## What Changes

- `katana-acp-client`: stdio transport、ACP capability negotiation、Ollama adapter（プロセス起動・health check・model 一覧・turn 送受信）を実装する。
- `katana-chat-ui`: VS Code 風 chat サイドパネル widget（開閉・固定表示・streaming 表示・disabled state）を実装する。
- settings: JSON Schema を `docs/settings-schema.json` として定義。path 渡し / コールバック両方式を実装する。
- autofix diff surface: KML diagnostics + LLM 提案の diff preview / confirm / apply を reusable widget として実装する。
- `v0.1.0` として release tag を切り、KatanA v0.23.0 が取り込める状態にする。

## Capabilities

### New Capabilities

- `llm-agent-protocol`: ACP client, Ollama adapter, capability negotiation.
- `chat-ui-component`: egui chat side-panel widget with settings integration.

## Impact

- `crates/katana-acp-client/src/` — transport, capability, Ollama adapter
- `crates/katana-chat-ui/src/` — widget, state, diff surface
- `docs/settings-schema.json` — settings JSON Schema
