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

### Chat UI（移管元: `katana-ui/src/views/panels/chat.rs` + `state/chat.rs`）

- `ChatPanel`（egui サイドパネル + overlay、固定/非固定切り替え、開閉、入力欄、メッセージ一覧）
- `ChatState`（message 履歴、pending flag、response_rx）

### Autofix（移管元: `katana-ui/src/app/autofix*.rs` + `state/autofix.rs` + `state/diff_preview.rs`）

- `AutofixRequestBuilder`: path + original_content + diagnostics → `FileAutofixRequest`（KML deterministic fix 後 content + residual diagnostics を含む）
- `AutofixPromptBuilder`: `FileAutofixRequest` → LLM prompt 文字列（`<<KATANA_AUTOFIX_CONTENT>>` マーカー形式）
- `AutofixResponseNormalizer`: LLM response → `FileAutofixCandidate`
- `AutofixState`（is_pending / response_rx / candidate）
- `DiffPreviewState`（original / proposed の行単位差分）

---

## 1. katana-acp-client crate: neutral interface と Ollama adapter を確立する

### 実施内容

KatanA `release/v0.23.0` の `katana-core/src/ai/` をベースに、`katana-acp-client` として独立 crate に移管する。
KatanA が interface crate のみに依存して型エラーが出ないこと（egui / katana-ui / katana-core 非依存）を必須とする。

### 完了条件

- [ ] 1.1 `crates/katana-acp-client/src/types.rs` に `AiRequest` / `AiResponse` / `AiCapabilities` / `AiModel` / `AiError` / `Param` を移管する
- [ ] 1.2 `crates/katana-acp-client/src/provider.rs` に `AiProvider` trait と `AiProviderRegistry` を移管する
- [ ] 1.3 `crates/katana-acp-client/src/ollama.rs` に `OllamaProvider`（ureq ベース）を移管する
- [ ] 1.4 `crates/katana-acp-client/src/settings.rs` に `AiSettings` / `OllamaSettings` を定義する
- [ ] 1.5 `katana-acp-client` の `cargo tree` に `egui` が含まれないことを確認する
- [ ] 1.6 unit test（`OllamaProvider` mock + `AiProviderRegistry` register / execute）を追加する

---

## 2. katana-chat-ui crate: chat サイドパネル widget を確立する

### 準備完了条件（Definition of Ready）

- [ ] Task 1 完了

### 実施内容

KatanA `release/v0.23.0` の `katana-ui/src/views/panels/chat.rs` / `state/chat.rs` をベースに、`katana-chat-ui` として移管する。
KatanA 側は `ChatPanel::builder()` API のみを呼ぶ。`katana-core` / `katana-ui` には依存しない。

### 完了条件

- [ ] 2.1 `crates/katana-chat-ui/src/panel.rs` に `ChatPanel`（egui サイドパネル / overlay、固定/非固定切り替え、開閉、入力欄、メッセージ一覧）を移管する
- [ ] 2.2 `crates/katana-chat-ui/src/state.rs` に `ChatState`（message 履歴、pending flag、response_rx）を移管する
- [ ] 2.3 `ChatPanel::builder().settings_slice(json).on_settings_changed(cb).build()` API を実装する
- [ ] 2.4 provider 未設定 / モデル未選択 / unavailable / timeout / invalid response の disabled state と recovery 導線を確認する
- [ ] 2.5 `katana-chat-ui` の `cargo tree` に `katana-core` / `katana-ui` が含まれないことを確認する

---

## 3. autofix diff surface を確立する

### 準備完了条件（Definition of Ready）

- [ ] Task 2 完了

### 実施内容

KatanA `release/v0.23.0` の `autofix_request.rs` / `autofix_support.rs` / `state/autofix.rs` / `state/diff_preview.rs` をベースに移管する。

### 完了条件

- [ ] 3.1 `crates/katana-chat-ui/src/autofix/request.rs` に `AutofixRequestBuilder` / `AutofixPromptBuilder` / `AutofixResponseNormalizer` を移管する（`<<KATANA_AUTOFIX_CONTENT>>` マーカー形式含む）
- [ ] 3.2 `crates/katana-chat-ui/src/autofix/state.rs` に `AutofixState` / `FileAutofixRequest` / `FileAutofixCandidate` を移管する
- [ ] 3.3 `crates/katana-chat-ui/src/diff.rs` に `DiffPreviewState`（行単位差分）と diff preview surface を移管する
- [ ] 3.4 KML diagnostics + LLM 提案の diff preview / confirm / apply flow が成立することを integration test する
- [ ] 3.5 元 content と LLM proposal content の差分を apply 前に preview できることを確認する

---

## 4. settings schema と docs を整備する

### 準備完了条件（Definition of Ready）

- [ ] Task 3 完了

- [ ] 4.1 `docs/settings-schema.json` を定義する（Ollama endpoint / model / timeout_secs / chat_enabled / autofix_enabled）
- [ ] 4.2 `cargo fmt` / `cargo clippy --workspace -- -D warnings` / `cargo test --workspace` が通る

---

## 5. v0.1.0 release

### 準備完了条件（Definition of Ready）

- [ ] Task 4 完了

- [ ] 5.1 release tag `v0.1.0` を切り GitHub Release を作成する
