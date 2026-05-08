# katana-chat-ui Roadmap

この roadmap は `docs/chat-ui-spec.ja.md` のあるべき姿を、優先度と version に割り当てるための一覧である。
OpenSpec change が存在するものは、必ず change 名を併記する。

## 優先度の定義

| Priority | 意味 | 判定基準 |
| --- | --- | --- |
| P0 | v0.1.0 から外すと標準 chat UI の検証が成立しない | 人間が起動して入力、送信、応答、thinking、output handoff を確認できない |
| P1 | 標準 UI の実利用に必要だが、v0.1.0 の手動確認成立後に分けられる | 接続、provider 拡張、履歴、割り込みなど |
| P2 | AI 作業者としての体験を上げる拡張 | file / diff / tool result の詳細 UI など |
| P3 | 後続の polish または外部 component 待ち | クリップボード画像、絵文字、timestamp、document viewer など |

## Version Plan

### P0: v0.1.0 — 標準 chat UI foundation

OpenSpec change: `v0-1-0-chat-widget-floem`

目的:

- `katana-chat-ui` が API-only ではなく、標準 chat UI を提供する状態にする。
- egui / Floem / GPUI で同じ標準 UI を人間が起動確認できる。

v0.1.0 に含める:

- composer 下部固定、IME、`Enter` 改行、`Command + Enter` 送信。
- 送信中は send button の位置を stop button に切り替える。
- provider / runtime / model / thinking / permission の control row。
- 既定は Ollama `/api/tags` で取得できた local model を使う低コスト文書作成バックエンドにする。
- Ollama は編集・コマンド実行できる agent provider ではなく local runtime として扱う。
- Ollama 文書作成の生成本文は streaming response と file candidate output の両方で確認できる。
- streaming response。
- 現在の context usage を used / max / percentage / status として表示する。
- context usage は composer 内の送信ボタン左側に円グラフ percentage として表示する。
- provider 選択は composer 内ではなく、title 左の provider icon + pulldown 表示から行う。
- 右上に新規チャット開始、履歴、設定の操作群を置く。
- `Thinking: false` では thinking surface を出さない。
- thinking 有効時の考慮ログ、動作中表示、完了後折りたたみ。
- Markdown subset と code fence の途切れ防止。
- output contract と host action intent。
- `options: { debug: true }` の右端 output hover。
- 右上トグルから開く設定画面 shell。
- テーマ、言語、placeholder、SVG icon、provider 表示順、composer 挙動の設定。
- 入力欄で `/` を入力した時に候補表示を開く launcher contract。
- file / diff / tool result / permission request を後続 view へ渡せる output extension interface。
- attachment intent と、OS file / host resource / virtual attachment を区別できる attachment interface。

v0.1.0 から外す:

- OS file picker と drag and drop 添付の実装、実機確認。
- session 履歴復元。
- 送信済み message 編集。
- 処理中の steering / interrupt。
- provider 接続設定、secret、provider usage、account usage。
- prompt、skill、workflow、command、hook、MCP の実エントリ供給。
- file / diff / tool result の詳細 view。
- クリップボード画像、絵文字、timestamp、試行時間。
- `katana-document-viewer` 差し替え。

### P0: v0.2.0 — secure connector and account usage

OpenSpec change: `v0-2-0-secure-connector-and-usage`

目的:

- ACP 接続と direct connector の安全な境界を固定する。
- secret を settings JSON に保存しない。
- provider usage / account usage surface の contract を固定する。
- 利用側が指定した settings JSON へ typed settings を merge する contract を固定する。
- provider 接続設定と MCP server 設定を扱う。

### P1: v0.3.0 — provider adapter expansion

OpenSpec change: `v0-3-0-vendor-adapter-expansion`

目的:

- agent provider と local runtime を分類する。
- Claude Code、Codex CLI、GitHub Copilot、OpenCode などを adapter hook として扱う。
- Ollama を agent provider と同列に扱わない。
- prompt、skill、workflow、command、hook、MCP の capability catalog を provider / adapter / host から供給する。
- 入力欄の `/` launcher に利用可能な prompt、skill、workflow、command を出す。

### P1: v0.4.0 — conversation history and control

OpenSpec change: `v0-4-0-conversation-history-and-control`

目的:

- session 履歴を保存し、UI から復元できるようにする。
- session 履歴一覧を表示し、title、provider、最終更新時刻、preview、状態を表示する。
- 現在 chat 内検索と、保存済み履歴の横断検索を提供する。
- 送信済み message の編集と、その時点からの再開を提供する。
- 処理中の追加投稿を steering / interrupt / queue として扱う。

### P2: v0.5.0 — output extension views

OpenSpec change: `v0-5-0-output-extension-views`

目的:

- file / diff / tool result / permission request の詳細 view を標準 UI の拡張領域に載せる。
- 実行や適用は host / provider 所有のまま、kcu は表示と操作 intent を提供する。

### P3: v0.6.0 — rich input and renderer polish

OpenSpec change: `v0-6-0-rich-input-and-renderer-polish`

目的:

- OS file picker、drag and drop 添付、クリップボード画像添付、絵文字、試行時間、送受信時刻を追加する。
- provider selector の候補行に provider icon を表示する。
- `katana-document-viewer` へ Markdown renderer を差し替える。
