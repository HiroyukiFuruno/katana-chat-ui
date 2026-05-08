# katana-chat-ui Roadmap

この roadmap は、v0.1.0 の新スコープと後続候補を管理する一覧である。
v0.1.0 の実作業は [v0-1-0-scope.md](./v0-1-0-scope.md) を source of truth とする。

旧 `v0-*` change は 2026-05-09 時点で広すぎるため、`openspec/changes/archive/2026-05-09-superseded-*` へ退避した。

## 優先度の定義

| Priority | 意味 | 判定基準 |
| --- | --- | --- |
| P0 | v0.1.0 の実用品質に必須 | Markdown 生成・編集・取り消し、session、新規 chat、履歴復元、provider/model 切り替え |
| P1 | v0.1.0 後に追加する主要機能 | 詳細設定、接続設定、secret、usage、slash launcher |
| P2 | agent 体験を上げる拡張 | file / diff / tool result の詳細 UI、検索、割り込み |
| P3 | polish または外部 component 待ち | クリップボード画像、絵文字、timestamp、document viewer |

## Version Plan

### P0: v0.1.0 — 標準 AI エージェントチャット UI

Scope: [v0-1-0-scope.md](./v0-1-0-scope.md)

目的:

- Markdown ファイルの生成、編集、取り消しができる標準 chat UI を提供する。
- 新規 session、履歴一覧、履歴復元、最後の session 復元を提供する。
- Codex CLI、Claude Code、GitHub Copilot、KatanAgent を初期 provider として扱う。
- Ollama は KatanAgent の local runtime とし、provider selector には出さない。
- Floem、egui、GPUI で同じ標準 UI / UX を表示する。
- 左右 debug panel、output debug hover、設定ボタンは v0.1.0 から削除する。

### P1: v0.2.0 以降 — detailed settings and connector hardening

再作成予定。旧 change は archive 済み。

候補:

- provider 接続詳細設定。
- secret 管理。
- provider usage / account usage。
- settings JSON merge。
- prompt / skill / workflow / command / hook / MCP。
- slash launcher。

### P2: v0.3.0 以降 — richer agent operations

再作成予定。旧 change は archive 済み。

候補:

- file / diff / tool result 詳細 view。
- 送信済み message 編集。
- 処理中の割り込み投稿。
- chat 内検索。
- 履歴横断検索。

### P3: v0.4.0 以降 — rich input and renderer polish

再作成予定。旧 change は archive 済み。

候補:

- OS file picker。
- drag and drop 添付。
- clipboard 画像添付。
- 絵文字パレット対応。
- timestamp / 試行時間。
- `katana-document-viewer` 差し替え。
