# katana-chat-ui OpenSpec

## Project

`katana-chat-ui`（kcu）は、vendor 非依存の LLM chat UI と ACP (Agent Client Protocol) client を提供する汎用 library。host application は git dependency として consume する。ACP 実装・vendor adapter・chat widget・settings 管理はすべてここで行う。

## Design Principles

- host application の public interface に UI フレームワーク型・vendor SDK を漏らさない。
- `katana-acp-client` は UI フレームワークに依存しない。widget と protocol は分離する。
- vendor 固有 UI 分岐を widget 内に持たない。差分は ACP capability negotiation で表現する。
- settings は JSON Schema で管理し、path 渡し（デフォルト）またはコールバック（host 統合）で統合する。

## Versioning

- `v0.0.1`: `katana-acp-client` neutral interface のみ（AiProvider trait / DocumentContext / OllamaProvider / DTO）。UI フレームワーク非依存。
- `v0.1.0`: neutral chat state（`katana-chat-ui`）+ Floem impl（`katana-chat-ui-floem`）+ autofix diff surface + settings schema。
- `v0.2.0`: document generation + translation overlay。
- `v0.3.0`: 追加 vendor adapter（OpenAI 互換 / Anthropic / Vertex AI）。`AiResponse` に `content_stream` フィールドを追加（既存 provider は `None` を返すため後方互換）。
- `v0.4.0`: 履歴永続化、複数会話管理

---

## Tech Stack

### 技術選定（確定）

| 層 | 採用 |
|----|------|
| UI フレームワーク | **Floem**（Rust 純正・クロスプラットフォーム） |
| 文字描画 | **cosmic-text**（IME 完全対応・カラー絵文字 SBIX/CBTF） |
| 2D レンダリング | **vello + wgpu**（compute-shader・Metal/DX12/Vulkan） |
| レイアウト | **taffy**（flexbox + CSS Grid） |
| アーキテクチャ参考 | **GPUI / Zed**（設計の教材として活用） |

React / TypeScript / WebView は使用しない。Rust 純正のみ。

### egui を採用しない理由

本 repo は新規実装であり、egui の既知制約を引き継がない。

- カラー絵文字：epaint が SBIX/CBTF 非対応 → cosmic-text で解決
- IME 不完全：egui TextEdit の composition が壊れる → cosmic-text + winit で解決
- レイアウト拡張不可：行間・マージンを vendor パッチなしに変えられない → vello Scene への直接描画で解決
- immediate mode の再描画コスト → vello の retained 描画で解決

### Crate 構成

```
katana-acp-client               neutral ACP interface（UI フレームワーク非依存）
                                  AiProvider trait / DocumentContext / OllamaProvider / DTO
katana-chat-ui                  neutral chat state（UI フレームワーク非依存）
                                  ChatSession / AutofixState / DiffPreviewState / ChatConfig
katana-chat-ui-floem            Floem + cosmic-text impl
                                  ChatPanelView / AutofixDiffView
```
