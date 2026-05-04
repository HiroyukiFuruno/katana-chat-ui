## Context

KatanA v0.23.0 は Ollama 接続・chat UI・autofix を直接実装した。これを katana-chat-ui v0.1.0 として独立させることで、KatanA の検証範囲を「取り込みの正しさ」のみに絞る。

## Goals

- KatanA `katana-core/src/ai/` と `katana-ui/src/app/chat.rs` 系の実装をすべてここへ移管する。
- `katana-acp-client` は `egui` に依存しない（protocol と UI の分離）。
- widget の public API に `egui::Ui` を使うが、KatanA に vendor SDK / Ollama 型は漏れない。
- settings は JSON Schema で管理。host は path 渡しかコールバックで統合する。

## Non-Goals

- OpenAI / Vertex AI / Bedrock adapter（v0.2.x）。
- 履歴永続化（v0.3.x）。

## Architecture

```
katana-acp-client
  transport/   — stdio JSON-RPC
  ollama/      — process spawn, health check, model list, turn送受信
  capability.rs — feature negotiation

katana-chat-ui
  panel.rs     — サイドパネル widget（開閉・固定・streaming）
  diff.rs      — diff preview / confirm / apply surface
  settings.rs  — path渡し / コールバック両方式
```

## Settings 統合

```rust
// path 渡し（デフォルト）
ChatPanel::builder()
    .settings_path("~/.config/katana-chat-ui/settings.json")
    .build()

// コールバック（KatanA の settings.json に統合）
ChatPanel::builder()
    .settings_slice(app_settings.chat_ui_slice())
    .on_settings_changed(|slice| app_settings.apply_chat_ui_slice(slice))
    .build()
```

スキーマ外キーは無視する。スキーマは `docs/settings-schema.json` で公開する。

## Verification

- `katana-acp-client` の `cargo tree` に `egui` が含まれない。
- `katana-chat-ui` の `cargo tree` に KatanA / katana-core が含まれない。
- Ollama adapter の unit test（mock server）が通る。
- diff surface の apply 後に re-lint が動作する（統合 test）。
- path 渡し / コールバック両方式の integration test が通る。
