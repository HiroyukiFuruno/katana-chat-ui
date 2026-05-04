## Context

v0.0.1 で確立した `katana-acp-client`（neutral interface + OllamaProvider）の上に chat UI を乗せる。
本 repo は新規実装のため egui を採用せず、最初から Floem + cosmic-text を採用する。

## Goals

- neutral state 層（`katana-chat-ui`）と rendering 層（`katana-chat-ui-floem`）を明確に分離する。
- chat 入力の IME 完全対応・カラー絵文字対応を v0.1.0 から保証する。
- autofix diff surface の confirm / reject / apply flow を確立する。
- host application は `ChatPanelView` と `AutofixDiffView` を embed するだけでよい。

## Non-Goals

- 追加 vendor adapter（OpenAI 互換等）— v0.3.0。
- 履歴永続化・複数会話管理 — v0.4.0。

## Architecture

```
katana-acp-client           neutral ACP interface（v0.0.1 から継承）
  └─ AiProvider trait
  └─ DocumentContext
  └─ OllamaProvider

katana-chat-ui              neutral state（UI フレームワーク非依存）
  session.rs                ChatSession（history / streaming buffer / turn 管理）
  autofix/
    request.rs              AutofixRequestBuilder / PromptBuilder / ResponseNormalizer
    state.rs                AutofixState / FileAutofixRequest / FileAutofixCandidate
  diff.rs                   DiffPreviewState（行単位 before/after）
  config.rs                 ChatConfig（path 渡し / コールバック両方式）

katana-chat-ui-floem        Floem + cosmic-text impl
  panel.rs                  ChatPanelView（impl View）
  autofix.rs                AutofixDiffView（impl View）
```

## State Flow

```
[host] DocumentContext ──► ChatSession.push_user_turn(prompt, ctx)
                                │
                                ▼
                         AiProvider::execute(AiRequest)
                                │
                           streaming rx
                                │
                                ▼
                        ChatSession.push_assistant_turn(content)
                                │
                                ▼
                    [katana-chat-ui-floem] ChatPanelView が再描画

[autofix flow]
AutofixRequestBuilder ──► FileAutofixRequest
AutofixPromptBuilder  ──► LLM prompt（<<KATANA_AUTOFIX_CONTENT>> マーカー）
OllamaProvider::execute ──► LLM response
AutofixResponseNormalizer ──► FileAutofixCandidate
DiffPreviewState ──► AutofixDiffView（before/after 表示）
confirm ──► host apply callback
```

## Settings 統合

```rust
// path 渡し（デフォルト）
ChatConfig::from_path("~/.config/katana-chat-ui/settings.json")

// コールバック（host の settings.json に統合）
ChatConfig::from_slice(host_settings.chat_ui_slice(), |slice| {
    host_settings.apply_chat_ui_slice(slice)
})
```

スキーマ外キーは無視する。スキーマは `docs/settings-schema.json` で公開する。

## Streaming 実装方針

v0.1.0 時点での streaming は `OllamaProvider` のみ対象。`AiProvider::execute` の返り値 `AiResponse` には現時点で streaming フィールドはない。代わりに `OllamaProvider` の内部で `mpsc::channel` を展開し、`ChatSession.response_rx` に渡す。

```rust
// ChatSession内部の streaming 接続パターン（v0.1.0）
let (tx, rx) = mpsc::channel::<String>();
self.response_rx = Some(rx);
std::thread::spawn(move || {
    // OllamaProvider 内部で /api/generate の chunk を tx.send
    provider.execute_streaming(&request, tx);
});
```

v0.3.0 で `AiResponse` に `content_stream` フィールドを追加する際に、この内部実装を統一 API に移行する。

## Verification

- `cargo tree -p katana-chat-ui | grep -E "floem|egui|vello"` が空
- `cargo tree -p katana-chat-ui-floem | grep -E "egui|epaint"` が空
- `cargo tree -p katana-chat-ui-floem | grep -E "katana-core|katana-platform"` が空
- `ChatSession` turn 管理 unit test が通る
- `AutofixRequestBuilder` → prompt マーカー形式 unit test が通る
- `DiffPreviewState` 行差分 unit test が通る
- `ChatPanelView` / `AutofixDiffView` Floem headless smoke test が通る
- `ChatConfig` path 渡し / コールバック両方式 integration test が通る
