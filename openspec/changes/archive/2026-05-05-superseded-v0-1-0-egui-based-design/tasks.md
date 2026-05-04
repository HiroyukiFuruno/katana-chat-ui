# Tasks: katana-chat-ui v0.1.0

## Branch Rule

- **標準ブランチ**: `release/v0.1.0`
- **作業ブランチ**: `feature/v0.1.0-task-x`

## 実装参照元（KatanA release/v0.23.0）

以下の実装が KatanA `release/v0.23.0` ブランチで先行実装されており、本 repo への移管元となる。

### AI Provider / Registry（移管元: `katana-core/src/ai/`）

- `types.rs`: `AiRequest` / `AiResponse` / `AiCapabilities` / `AiModel` / `AiError` / `Param`
- `registry.rs`: `AiProvider` trait + `AiProviderRegistry`（register / set_active / execute / list_models）
- `ollama.rs`: `OllamaProvider`（ureq ベース、`/api/generate` + `/api/tags`、endpoint normalize、timeout）

### Settings（移管元: `katana-platform/src/settings/types/ai.rs`）

- `AiSettings` / `OllamaSettings`（endpoint / selected_model / timeout_secs / chat_enabled / autofix_enabled）

### Chat State（移管元: `katana-ui/src/state/chat.rs`）

- `ChatSession`（message 履歴、pending flag、response_rx、turn 管理）
- `DocumentContext`（現在 document の path / content / diagnostics をまとめた構造体）

### Chat UI（移管元: `katana-ui/src/views/panels/chat.rs`）

- `ChatPanelWidget`（egui サイドパネル + overlay、固定/非固定切り替え、開閉、入力欄、メッセージ一覧、streaming 表示）

### Autofix（移管元: `katana-ui/src/app/autofix*.rs` + `state/autofix.rs` + `state/diff_preview.rs`）

- `AutofixRequestBuilder`: path + original_content + diagnostics → `FileAutofixRequest`（KML deterministic fix 後 content + residual diagnostics を含む）
- `AutofixPromptBuilder`: `FileAutofixRequest` → LLM prompt 文字列（`<<KATANA_AUTOFIX_CONTENT>>` マーカー形式）
- `AutofixResponseNormalizer`: LLM response → `FileAutofixCandidate`
- `AutofixState`（is_pending / response_rx / candidate）
- `DiffPreviewState`（original / proposed の行単位差分）

---

## 0. ワークスペース準備

- [ ] 0.1 `Cargo.toml` の `[workspace] members` に `crates/katana-chat-ui-egui` を追加する
- [ ] 0.2 `crates/katana-chat-ui-egui/Cargo.toml` を新規作成し `katana-chat-ui`・`katana-acp-client`・`egui` を依存に追加する
- [ ] 0.3 `cargo build --workspace` が通ること

---

## 1. katana-acp-client crate: neutral interface と Ollama adapter を確立する

### 実施内容

KatanA `release/v0.23.0` の `katana-core/src/ai/` をベースに、`katana-acp-client` として独立 crate に移管する。
KatanA が interface crate のみに依存して型エラーが出ないこと（egui / katana-ui / katana-core 非依存）を必須とする。

### 完了条件

- [ ] 1.1 `crates/katana-acp-client/src/types.rs` に `AiRequest` / `AiResponse` / `AiCapabilities` / `AiModel` / `AiError` / `Param` を移管する
- [ ] 1.2 `crates/katana-acp-client/src/provider.rs` に `AiProvider` trait と `AiProviderRegistry`（register / set_active / execute / list_models）を移管する
- [ ] 1.3 `crates/katana-acp-client/src/ollama.rs` に `OllamaProvider`（ureq ベース、`/api/generate` + `/api/tags`、endpoint normalize、timeout）を移管する
- [ ] 1.4 `crates/katana-acp-client/src/context.rs` に `DocumentContext`（path / content / diagnostics）を定義する
- [ ] 1.5 `crates/katana-acp-client/src/settings.rs` に `AiSettings` / `OllamaSettings`（endpoint / selected_model / timeout_secs / chat_enabled / autofix_enabled）を定義する
- [ ] 1.6 `cargo tree -p katana-acp-client | grep -i egui` が空であること
- [ ] 1.7 unit test（`OllamaProvider` mock + `AiProviderRegistry` register / set_active / execute / list_models）を追加する

---

## 2. katana-chat-ui crate: neutral chat state を確立する

### 準備完了条件

- [ ] Task 1 完了

### 実施内容

KatanA `release/v0.23.0` の `katana-ui/src/state/chat.rs` をベースに、neutral state 層として `katana-chat-ui` に移管する。
**egui 型をこの crate に含めない。** egui 依存はすべて Task 3 の `katana-chat-ui-egui` に置く。

### 完了条件

- [ ] 2.1 `crates/katana-chat-ui/src/session.rs` に `ChatSession`（message 履歴、pending flag、response_rx、turn 管理）を移管する
- [ ] 2.2 `crates/katana-chat-ui/src/autofix/request.rs` に `AutofixRequestBuilder` / `AutofixPromptBuilder` / `AutofixResponseNormalizer` を移管する（`<<KATANA_AUTOFIX_CONTENT>>` マーカー形式含む）
- [ ] 2.3 `crates/katana-chat-ui/src/autofix/state.rs` に `AutofixState` / `FileAutofixRequest` / `FileAutofixCandidate` を移管する
- [ ] 2.4 `crates/katana-chat-ui/src/diff.rs` に `DiffPreviewState`（行単位差分）を移管する
- [ ] 2.5 `crates/katana-chat-ui/src/config.rs` に `ChatConfig`（settings 注入：path 渡し / コールバック両方式）を実装する
- [ ] 2.6 `cargo tree -p katana-chat-ui | grep -E "egui|katana-core|katana-ui"` が空であること
- [ ] 2.7 unit test（`ChatSession` turn 管理 / `AutofixRequestBuilder` → prompt 文字列 / `DiffPreviewState` 行差分）を追加する

---

## 3. katana-chat-ui-egui crate: egui impl を確立する

### 準備完了条件

- [ ] Task 2 完了

### 実施内容

KatanA `release/v0.23.0` の `katana-ui/src/views/panels/chat.rs` / `app/autofix*.rs` / `state/diff_preview.rs` をベースに、egui rendering 層として `katana-chat-ui-egui` に移管する。
**KatanA v0.24.0 の準備完了条件である `ChatPanelWidget::show()` API と autofix diff surface API を確立することが本 Task の目的。**

### 完了条件

- [ ] 3.1 `crates/katana-chat-ui-egui/src/panel.rs` に `ChatPanelWidget::show(ui: &mut egui::Ui, session: &mut ChatSession, config: &ChatConfig)` を実装する
  - サイドパネル + overlay、固定/非固定切り替え、開閉
  - メッセージ一覧（ユーザー / アシスタント）
  - 入力欄（`egui::TextEdit`）と送信ボタン
  - streaming 中の中断ボタン
  - provider 未設定 / unavailable / pending の disabled state 表示
- [ ] 3.2 `crates/katana-chat-ui-egui/src/autofix.rs` に autofix diff surface widget を実装する
  - `AutofixDiffWidget::show(ui, autofix_state, diff_state)` API
  - diff preview（行単位 before/after 表示）
  - confirm / reject / apply ボタン
- [ ] 3.3 `crates/katana-chat-ui-egui/src/lib.rs` で `ChatPanelWidget` / `AutofixDiffWidget` を pub re-export する
- [ ] 3.4 egui impl に `katana-core` / `katana-platform` が漏れていないこと（`cargo tree -p katana-chat-ui-egui | grep -E "katana-core|katana-platform"` が空）
- [ ] 3.5 egui_kittest を使った widget smoke test（panel 表示 / disabled state / diff surface 表示）を追加する

---

## 4. settings schema と docs を整備する

### 準備完了条件

- [ ] Task 3 完了

- [ ] 4.1 `docs/settings-schema.json` を定義する（Ollama endpoint / selected_model / timeout_secs / chat_enabled / autofix_enabled）
- [ ] 4.2 `ChatConfig::from_path(path)` と `ChatConfig::from_slice(json, cb)` の両方式を integration test する
- [ ] 4.3 `cargo fmt --check` / `cargo clippy --workspace -- -D warnings` / `cargo test --workspace` がすべて通る

---

## 5. v0.1.0 release

### 準備完了条件

- [ ] Task 4 完了

- [ ] 5.1 release tag `v0.1.0` を切り GitHub Release を作成する
- [ ] 5.2 KatanA v0.24.0 の準備完了条件（`katana-chat-ui` v0.1.0 tag・`ChatPanelWidget::show()` API 確定・autofix diff surface API 確定）がすべて満たされていることを確認する
