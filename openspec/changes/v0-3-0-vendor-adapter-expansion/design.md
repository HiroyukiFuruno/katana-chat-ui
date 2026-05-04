## Context

`AiProvider` trait は v0.0.1 から `execute(&AiRequest) -> Result<AiResponse, AiError>` の contract で統一されている。adapter ごとの差分は実装内部に閉じ込める。

## Goals

- `AiProvider` trait の変更なしに adapter を追加する。
- streaming 対応を `AiResponse` の拡張（`content_stream: Option<Receiver<String>>`）で表現する。
- 認証情報（API key / OAuth2）はすべて settings schema 経由で注入し、env var fallback を提供する。

## Non-Goals

- 独自 agent protocol（tool use / function calling）— 別 spec で検討。
- 履歴永続化 — v0.4.0。

## Streaming と AiResponse の拡張

v0.3.0 で `AiResponse` に `content_stream: Option<Receiver<String>>` を追加する。
**既存の `OllamaProvider` は `content_stream: None` を返すだけでよく、コンパイルエラーにならない。**
`ChatSession` は `content_stream` が `Some` の場合のみ streaming 表示パスを使い、`None` の場合は `content` フィールドを一括表示する。

```rust
pub struct AiResponse {
    pub content: String,               // 既存（非 streaming provider はここに全文）
    pub content_stream: Option<Receiver<String>>, // v0.3.0 追加
    pub metadata: Vec<Param>,
}
```

## Vertex AI の実装方針

Vertex AI は **REST API**（`/v1/projects/.../generateContent`）を使用する。gRPC は対象外。
ADC 認証は `GOOGLE_APPLICATION_CREDENTIALS` 環境変数（サービスアカウント JSON）を優先し、
`gcloud auth application-default login` の ADC ファイル（`~/.config/gcloud/application_default_credentials.json`）にフォールバックする。

## Architecture 追加分

```
katana-acp-client（追加）
  openai_compat.rs    OpenAiCompatProvider（SSE streaming / `/v1/chat/completions`）
  anthropic.rs        AnthropicProvider（`/v1/messages`、SSE streaming）
  vertex_ai.rs        VertexAiProvider（REST / ADC 認証）
```

## API リファレンス

| Provider | Endpoint | 認証 | ドキュメント |
|---|---|---|---|
| OpenAI 互換 | `POST /v1/chat/completions` | `Authorization: Bearer {api_key}` | https://platform.openai.com/docs/api-reference/chat |
| Anthropic | `POST /v1/messages` | `x-api-key: {api_key}` | https://docs.anthropic.com/en/api/messages |
| Vertex AI | `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:generateContent` | OAuth2 Bearer（ADC） | https://cloud.google.com/vertex-ai/docs/reference/rest/v1/projects.locations.publishers.models/generateContent |

## Provider Selection

```rust
// settings.provider の値で AiProviderRegistry に登録するものを切り替える
match settings.provider.as_str() {
    "ollama"        => registry.register(Box::new(OllamaProvider::from_settings(&s))),
    "openai-compat" => registry.register(Box::new(OpenAiCompatProvider::from_settings(&s))),
    "anthropic"     => registry.register(Box::new(AnthropicProvider::from_settings(&s))),
    "vertex-ai"     => registry.register(Box::new(VertexAiProvider::from_settings(&s))),
    _               => { /* unknown provider: log warn, no-op */ }
}
```

## Streaming

`AiResponse` に `content_stream: Option<Receiver<String>>` を追加する。
非 streaming provider は `None` を返す。`ChatSession` は `Some` の場合のみ streaming 表示パスを使う。

## Verification

- `OpenAiCompatProvider` の unit test（mock HTTP server / SSE）が通る
- `AnthropicProvider` の unit test（mock HTTP server）が通る
- `VertexAiProvider` の unit test（ADC mock）が通る
- `AiProvider` trait の変更がないことを型チェックで確認する（既存 test がコンパイルエラーなし）
- `cargo test --workspace` が通る
