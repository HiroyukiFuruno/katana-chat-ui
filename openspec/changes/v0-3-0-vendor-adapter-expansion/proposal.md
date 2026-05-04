## Why

v0.1.0 の `OllamaProvider` のみでは、OpenAI API 互換エンドポイント（LM Studio / llama.cpp server 等）や Anthropic / Vertex AI への接続ができない。`AiProvider` trait は v0.0.1 から provider 非依存に設計されているため、新規 adapter を追加するだけで対応できる。

## What Changes

### katana-acp-client（adapter 追加）

- `OpenAiCompatProvider`：OpenAI API 互換エンドポイント（`/v1/chat/completions`）adapter
  - LM Studio / llama.cpp server / Ollama `/v1` エンドポイント対応
  - streaming（SSE）対応
- `AnthropicProvider`：Anthropic Messages API adapter（`/v1/messages`）
- `VertexAiProvider`：Vertex AI Gemini API adapter（OAuth2 / ADC 認証）

### docs/settings-schema.json（更新）

- `provider`: `"ollama"` | `"openai-compat"` | `"anthropic"` | `"vertex-ai"`
- `openai_compat.endpoint` / `openai_compat.api_key` / `openai_compat.selected_model`
- `anthropic.api_key` / `anthropic.selected_model`
- `vertex_ai.project` / `vertex_ai.location` / `vertex_ai.selected_model`

## Capabilities

### New Capabilities

- `openai-compat-provider`: OpenAI API 互換 adapter（SSE streaming）
- `anthropic-provider`: Anthropic Messages API adapter
- `vertex-ai-provider`: Vertex AI Gemini adapter

### Inherited

- 既存 capabilities すべて変更なし

## Impact

- `crates/katana-acp-client/` — adapter 追加
- `docs/settings-schema.json` — provider 設定項目追加
