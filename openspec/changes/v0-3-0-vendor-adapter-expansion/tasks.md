# Tasks: v0.3.0 — vendor adapter expansion

> `AiProvider` trait を変更せず、OpenAI 互換 / Anthropic / Vertex AI の adapter を追加する。

## Branch Rule

`release/v0.3.0` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [ ] v0.2.0 がリリース済みであること
- [ ] `design.md` の Streaming 拡張・Vertex AI 実装方針セクションをレビュー済みであること
- [ ] mock HTTP server ライブラリを `mockito` で統一することが合意済みであること
- [ ] Vertex AI ADC 認証の実装方針（`GOOGLE_APPLICATION_CREDENTIALS` 優先 → gcloud ADC フォールバック）が合意済みであること
- [ ] 各 provider の API リファレンス URL が design.md に記載済みであること

---

## 1. OpenAiCompatProvider を実装する

### 完了条件

- [ ] 1.1 `crates/katana-acp-client/src/openai_compat.rs` に `OpenAiCompatProvider` を実装する
  - `/v1/chat/completions` エンドポイント（POST）
  - SSE streaming（`data: {...}` 行パース、`data: [DONE]` で終端）
  - `api_key` は settings 注入・`OPENAI_API_KEY` env var fallback
  - `selected_model` は settings 注入・デフォルト `"gpt-4o-mini"`
  - `AiResponse.content_stream` に `Receiver<String>` を返す
- [ ] 1.2 `OpenAiCompatProvider` が `AiProvider` trait を実装していることを確認する
- [ ] 1.3 `mockito` を使った unit test（正常系 / API エラー / SSE streaming）を追加する

---

## 2. AnthropicProvider を実装する

### 準備完了条件

- [ ] Task 1 完了

### 完了条件

- [ ] 2.1 `crates/katana-acp-client/src/anthropic.rs` に `AnthropicProvider` を実装する
  - `/v1/messages` エンドポイント（POST）
  - SSE streaming（`event: content_block_delta` / `data: {...}` パース）
  - `api_key` は settings 注入・`ANTHROPIC_API_KEY` env var fallback
  - `selected_model` は settings 注入・デフォルト `"claude-3-5-haiku-20241022"`
  - `AiResponse.content_stream` に `Receiver<String>` を返す
- [ ] 2.2 `mockito` を使った unit test を追加する

---

## 3. VertexAiProvider を実装する

### 準備完了条件

- [ ] Task 2 完了

### 完了条件

- [ ] 3.1 `crates/katana-acp-client/src/vertex_ai.rs` に `VertexAiProvider` を実装する
  - Gemini REST API（`/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:generateContent`）
  - ADC 認証：`GOOGLE_APPLICATION_CREDENTIALS`（サービスアカウント JSON）優先、`~/.config/gcloud/application_default_credentials.json` にフォールバック
  - `project` / `location` / `selected_model` は settings 注入
  - `AiResponse.content_stream: None`（Gemini REST はストリームなし、v0.3.x で対応）
- [ ] 3.2 `mockito` を使った unit test を追加する（ADC トークン取得を mock）

---

## 4. settings schema を更新する

### 準備完了条件

- [ ] Task 3 完了

- [ ] 4.1 `docs/settings-schema.json` に以下を追加する
  - `provider`: `"ollama"` | `"openai-compat"` | `"anthropic"` | `"vertex-ai"`
  - `openai_compat.endpoint` / `openai_compat.api_key` / `openai_compat.selected_model`
  - `anthropic.api_key` / `anthropic.selected_model`
  - `vertex_ai.project` / `vertex_ai.location` / `vertex_ai.selected_model`
- [ ] 4.2 `ChatConfig` の provider selection ロジックを追加する（settings.provider 値で registry に登録する adapter を切り替え）
- [ ] 4.3 unknown provider 値は warn ログを出して no-op になることを確認する

---

## 5. 品質ゲート

### 準備完了条件

- [ ] Task 4 完了

- [ ] 5.1 既存の `OllamaProvider` / `ChatSession` / `AutofixState` テストがコンパイルエラーなしで通ること（`AiProvider` trait 変更なしの確認）
- [ ] 5.2 `cargo fmt --check` が通ること
- [ ] 5.3 `cargo clippy --workspace -- -D warnings` が通ること
- [ ] 5.4 `cargo test --workspace` が通ること

---

## 6. v0.3.0 release

### 準備完了条件

- [ ] Task 5 完了

### 完了条件（Definition of Done）

- [ ] 6.1 `release/v0.3.0` ブランチから PR を作成し master へ merge する
- [ ] 6.2 release tag `v0.3.0` を切り GitHub Release を作成する
- [ ] 6.3 以下がすべて満たされていること
  - `cargo test --workspace` が通る
  - `OpenAiCompatProvider` / `AnthropicProvider` / `VertexAiProvider` が `AiProvider` trait を実装していること
  - 既存の `OllamaProvider` / `ChatSession` / `AutofixState` のテストがコンパイルエラーなしで通ること
  - `docs/settings-schema.json` に全 provider の設定項目が追加されていること
