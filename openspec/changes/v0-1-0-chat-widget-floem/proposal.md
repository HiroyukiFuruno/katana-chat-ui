## Why

v0.0.1 で確立した `katana-acp-client` neutral interface の上に、chat session state 管理（`katana-chat-ui`）と Floem + cosmic-text による rendering impl（`katana-chat-ui-floem`）を実装する。

本 repo は **新規実装** であるため egui を経由しない。chat 入力の IME 完全対応・カラー絵文字対応を最初から保証する。

## What Changes

### katana-chat-ui（neutral state、UI フレームワーク非依存）

- `ChatSession`：message history / streaming buffer / turn 管理
- `AutofixRequestBuilder` / `AutofixPromptBuilder` / `AutofixResponseNormalizer`
- `AutofixState` / `FileAutofixRequest` / `FileAutofixCandidate`
- `DiffPreviewState`：行単位 before/after 差分
- `ChatConfig`：settings 注入（path 渡し / コールバック両方式）

### katana-chat-ui-floem（Floem + cosmic-text impl）

- `ChatPanelView`（`impl View`）：サイドパネル + overlay、固定/非固定、入力欄（IME・絵文字対応）、メッセージ一覧、streaming 表示
- `AutofixDiffView`（`impl View`）：行単位差分 preview / confirm / reject / apply

### docs/settings-schema.json

- `ollama.endpoint` / `ollama.selected_model` / `ollama.timeout_secs` / `chat.enabled` / `autofix.enabled`

## Capabilities

### New Capabilities

- `chat-state`: neutral chat session / autofix proposal 管理（UI フレームワーク非依存）
- `chat-ui-floem`: Floem + cosmic-text chat panel + autofix diff surface

### Inherited from v0.0.1

- `acp-interface`: `AiProvider` trait + `DocumentContext`（変更なし）
- `ollama-provider`: `OllamaProvider`（変更なし）

## Impact

- `crates/katana-chat-ui/` — neutral state crate（新規）
- `crates/katana-chat-ui-floem/` — Floem impl crate（新規）
- `docs/settings-schema.json` — settings JSON Schema（新規）
- `Cargo.toml` — workspace members 追加
