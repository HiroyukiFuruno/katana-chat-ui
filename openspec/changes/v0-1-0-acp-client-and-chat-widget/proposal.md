## Why

v0.0.1 で確立した `katana-acp-client` neutral interface（`AiProvider` trait + `DocumentContext`）の上に、egui chat widget と autofix diff surface を実装する。これにより KatanA v0.28.0 が `katana-chat-ui` v0.1.0 を git dependency として取り込み、KatanA 内の直接実装（`katana-core/src/ai/`、`katana-ui/src/app/chat.rs` 等）を除去できる。

## 準備完了条件（Definition of Ready）

- `katana-chat-ui` v0.0.1（`katana-acp-client` neutral crate）がリリース済みであること

## Migration from KatanA

以下を KatanA `release/v0.23.0` から移管する：

- `crates/katana-ui/src/views/panels/chat.rs` → chat side-panel widget
- `crates/katana-ui/src/app/autofix.rs`、`autofix_request.rs`、`autofix_support.rs` → autofix diff surface
- `crates/katana-ui/src/state/autofix.rs`、`diff_preview.rs`、`chat.rs` → widget 内部 state
- `crates/katana-platform/src/settings/types/ai.rs` → settings schema に統合

## What Changes

- `katana-chat-ui`（egui widget crate）に以下を実装する：
  - VS Code 風 chat サイドパネル widget（開閉・固定表示・streaming 表示・disabled state）
  - autofix diff surface：KML diagnostics + LLM 提案の diff preview / confirm / apply
  - `ChatPanel::show(ui, context, config)` を KatanA が呼ぶ唯一のエントリポイントにする
- settings：JSON Schema を `docs/settings-schema.json` として定義
- `v0.1.0` として release tag を切り、KatanA v0.28.0 が取り込める状態にする

## Capabilities

### New Capabilities（v0.1.0）

- `chat-ui-component`：egui chat side-panel widget（v0.0.1 の AiProvider 実装を使用）
- `autofix-diff-surface`：KML diagnostics + LLM 提案の diff preview / confirm / apply reusable widget

### Inherited from v0.0.1

- `acp-interface`：`AiProvider` trait + `DocumentContext` + `AiIntent`（変更なし）
- `ollama-provider`：ureq ベース Ollama adapter（変更なし）

## Impact

- `crates/katana-chat-ui/src/` — widget、state、diff surface（egui 依存）
- `docs/settings-schema.json` — settings JSON Schema
