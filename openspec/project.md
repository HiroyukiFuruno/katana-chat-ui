# katana-chat-ui OpenSpec

## Project

`katana-chat-ui`（kcu）は、特定アプリケーションを知らない汎用の AI チャット UI 基盤です。Agent Client Protocol（ACP）対応エージェントを第一候補として扱い、ACP が使えない提供元（vendor）には安全な直接接続（direct connector）を用意します。

kcu は特定の親アプリを知らず、egui にも依存しません。egui を使う host application は、kcu の framework-neutral core を自分の UI 層で描画します。

## Source of Truth

- active change 配下の `proposal.md` / `design.md` / `specs/*/spec.md` / `tasks.md` を実装時の正とする。
- `archive/` 配下は履歴であり、新規実装の根拠にしない。
- host application 固有名、egui 型、vendor SDK 型を active spec の public contract に入れない。

## Design Principles

- core crate の public API に UI framework 型、vendor SDK 型、host application 固有型を漏らさない。
- ACP client、直接接続（direct connector）、chat state、rendering implementation を分離する。
- 提供元（vendor）ごとの UI 差分は、widget 内の `if vendor == ...` ではなく capability と adapter 境界で表す。
- ACP 対応 agent は、ACP の初期化（initialize）、能力交渉（capability negotiation）、session config options、content block を優先して使う。
- ACP 非対応 provider の秘匿情報（secret）は平文 settings に保存しない。通信ごとに秘匿情報ストアから取り出し、復号し、利用後に破棄する。
- 設定は host 統合と kcu 独立保存の両方を扱うが、保存先の責務を明示する。
- React、TypeScript、WebView 前提の設計を持ち込まない。

## Versioning

- `v0.0.1`: `katana-acp-client` neutral interface と Ollama MVP。UI framework 非依存。
- `v0.1.0`: chat UX foundation。framework-neutral core、標準 AI チャット入力、添付、Markdown subset、theme、SVG icon、context usage 表示、Floem reference implementation。
- `v0.2.0`: secure connector and account usage。ACP 接続簡略化、direct connector 用 secret store、account / usage 表示、provider settings schema。
- `v0.3.0`: multi vendor adapter expansion。Ollama を MVP 基準に、ACP agent adapters と direct provider adapters の分類、Claude Code / Codex / GitHub Copilot / Bedrock / Vertex AI の対応方針を固定。
- `v0.4.0`: 履歴永続化、複数会話管理、session resume。
- `v0.5.0` 以降: document generation、translation overlay など chat foundation 以外の応用機能。

## Tech Stack

### 確定方針

| 層 | 採用 |
|----|------|
| core state | Rust crate。UI framework 非依存 |
| ACP transport | JSON-RPC over stdio を必須対応。HTTP / WebSocket は ACP 側の安定化後に別 change |
| reference UI | Floem + cosmic-text + vello。任意 crate として提供 |
| settings | JSON Schema + typed Rust config |
| secret | host-provided secret store を必須境界にし、OS credential store を優先実装 |

### egui との関係

kcu は egui を依存に持たない。egui app から使う場合は、host が kcu の `ChatUiState` / `ChatRenderModel` / callback contract を egui 側に写像する。kcu 側には `egui::Ui`、`egui::Context`、`egui` feature を追加しない。

### Crate 構成

```
katana-acp-client              ACP client と direct provider 共通 contract
katana-chat-ui                 framework-neutral chat state / render model
katana-chat-ui-floem           Floem reference implementation
katana-chat-connectors         direct connector / ACP connector / secret boundary
```

実動作検証用の UI harness は `crates/` に含めない。`tools/e2e-host-app/` のような非公開の外部 host fixture とし、配布対象 crate と混ぜない。この fixture は kcu を downstream dependency として実際に取り込み、host application 視点の E2E で起動・描画・操作を検証する。
