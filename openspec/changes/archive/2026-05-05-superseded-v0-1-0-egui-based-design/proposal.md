## Why

v0.0.1 で確立した `katana-acp-client` neutral interface（`AiProvider` trait + `DocumentContext`）の上に、chat state 管理（neutral）と egui rendering（egui impl）を実装する。

**egui の制約（IME 不完全・カラー絵文字非対応）を chat 入力に持ち込まないため、`katana-chat-ui`（neutral）と `katana-chat-ui-egui`（egui impl）を分ける。** chat state・message history・streaming buffer は egui を知らない。egui に依存するのはレンダリング層のみ。将来の独自 UI フレームワーク導入時は `katana-chat-ui-egui` を差し替えるだけで、`katana-chat-ui`（neutral）は変えない。

## 準備完了条件（Definition of Ready）

- `katana-chat-ui` v0.0.1（`katana-acp-client` neutral crate）がリリース済みであること

## Migration from KatanA

以下を KatanA `release/v0.23.0` から移管する：

- `crates/katana-ui/src/views/panels/chat.rs` → `katana-chat-ui-egui` の ChatPanelWidget
- `crates/katana-ui/src/app/autofix.rs`、`autofix_request.rs`、`autofix_support.rs` → `katana-chat-ui-egui` の autofix diff surface
- `crates/katana-ui/src/state/autofix.rs`、`diff_preview.rs`、`chat.rs` → `katana-chat-ui`（neutral）の state 層
- `crates/katana-platform/src/settings/types/ai.rs` → settings schema に統合

## What Changes

### katana-chat-ui（neutral、egui ゼロ）

- `ChatSession`：message history、streaming buffer、turn 管理
- `AutofixProposal`：診断 + LLM 提案 + diff 状態
- `ChatConfig`：設定注入（endpoint、model、enabled フラグ等）
- `ChatPanel` trait：UI フレームワーク非依存の panel widget 契約

### katana-chat-ui-egui（egui impl）

- `ChatPanelWidget::show(ui, session, config)`：chat サイドパネル描画
  - IME / 絵文字制約を docs に明記（egui 段階の既知制約）
  - streaming 表示、disabled state
- autofix diff surface widget：diff preview / confirm / apply

KatanA は `katana-chat-ui-egui` の `ChatPanelWidget::show()` を呼ぶだけ。

## Capabilities

### New Capabilities（v0.1.0）

- `chat-state-component`：neutral chat session / autofix proposal 管理（egui ゼロ）
- `chat-ui-egui`：egui chat panel + autofix diff surface

### Inherited from v0.0.1

- `acp-interface`：`AiProvider` trait + `DocumentContext` + `AiIntent`（変更なし）
- `ollama-provider`：ureq ベース Ollama adapter（変更なし）

## Known Constraints（egui impl 段階）

`katana-chat-ui-egui` の chat 入力は egui TextEdit を使用するため、以下の制約がある：

- **IME composition 不完全**：日本語・中国語等の確定前インライン表示が正しく動作しないケースがある
- **カラー絵文字非対応**：入力・表示ともに egui フォントアトラスの制約で欠落する

根本解決は独自 UI フレームワーク導入時に `katana-chat-ui-egui` を差し替えることで対応する。`katana-chat-ui`（neutral）の state 層は影響を受けない。

## Impact

- `crates/katana-chat-ui/` — neutral state crate（egui 非依存）
- `crates/katana-chat-ui-egui/` — egui impl crate
- `docs/settings-schema.json` — settings JSON Schema
