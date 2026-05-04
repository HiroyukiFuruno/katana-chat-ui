# katana-chat-ui OpenSpec

## Project

`katana-chat-ui`（kcu）は、vendor 非依存の LLM chat UI と ACP (Agent Client Protocol) client を提供する library。KatanA はこれを git dependency として consume するだけ。ACP 実装・vendor adapter・chat widget・settings 管理はすべてここで行う。

## Design Principles

- KatanA の public interface に egui 型・vendor SDK を漏らさない。KatanA が見るのは `ChatPanel` builder API と settings 統合の 2 点のみ。
- `katana-acp-client` は `egui` に依存しない。widget と protocol は分離する。
- vendor 固有 UI 分岐を widget 内に持たない。差分は ACP capability negotiation で表現する。
- settings は JSON Schema で管理し、path 渡し（デフォルト）またはコールバック（IDE 統合）で host と統合する。

## Versioning

- `v0.1.x`: ACP client (stdio transport + Ollama adapter) + chat widget + settings schema + autofix diff surface。KatanA v0.23.0 がこれを取り込む。
- `v0.2.x`: 追加 vendor adapter（OpenAI 互換、Vertex AI 等）
- `v0.3.x`: 履歴永続化、複数会話管理

## Consumers

- [KatanA](https://github.com/HiroyukiFuruno/KatanA) — git tag pinned dependency（v0.23.0 で取り込み）

---

## UI フレームワーク移行方針（egui → Floem）

このセクションはエコシステム全体で共通の方針。詳細は [KatanA openspec/project.md](https://github.com/HiroyukiFuruno/KatanA/blob/master/openspec/project.md) を正とする。

### 技術選定（確定）

| 層 | 採用 |
|----|------|
| UI フレームワーク | **Floem**（Rust 純正・クロスプラットフォーム） |
| 文字描画 | **cosmic-text**（IME 完全対応・カラー絵文字 SBIX/CBTF） |
| 2D レンダリング | **vello + wgpu**（compute-shader・Metal/DX12/Vulkan） |
| レイアウト | **taffy**（flexbox + CSS Grid） |
| アーキテクチャ参考 | **GPUI / Zed**（設計の教材として活用） |

React / TypeScript / WebView は使用しない。Rust 純正のみ。

### egui から脱却する理由（要約）

- カラー絵文字：epaint が SBIX/CBTF 非対応 → cosmic-text で解決
- IME 不完全：egui TextEdit の composition が壊れる → cosmic-text + winit で解決
- レイアウト拡張不可：vendor パッチなしに行間・マージンを変えられない → vello Scene への直接描画で解決
- immediate mode の再描画コスト → vello の retained 描画で解決

### この repo の責務

各 `-egui` impl crate を `-floem` impl crate に差し替える。neutral interface crate は変えない。
KatanA の `Cargo.toml` の impl crate 行を変えるだけで移行が完了する。

### katana-chat-ui の移行

```
katana-acp-client               neutral ACP interface（変わらない）
katana-chat-ui                  neutral chat state（変わらない）
katana-chat-ui-egui             MVP 実装（Phase 1 で置き換え対象）
katana-chat-ui-floem            Floem + cosmic-text 実装（Phase 1 で新規作成）
```

chat 入力の IME・絵文字問題は editor と同様に最優先。
