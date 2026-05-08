# v0.1.0 新スコープ

この文書は、v0.1.0 を作り直すための単一の作業基準である。
旧 `openspec/changes/v0-*` は広すぎるため、`2026-05-09-superseded-*` として archive へ退避した。

## 1. 目的

`katana-chat-ui` は、利用側が独自に AI エージェントチャット画面を作らなくても使える標準 UI を提供する。

v0.1.0 は「とりあえず会話できる」ではなく、次を満たす最小の実用品質に絞る。

- チャット指示で Markdown ファイルを生成できる。
- チャット指示で Markdown ファイルを編集できる。
- 生成、編集した変更を元に戻せる。
- 新規セッションを開始できる。
- 過去セッションを履歴から復元できる。
- 最後に開いていたセッションを再起動時に復元できる。
- provider と model を UI から切り替えられる。
- 同一セッション内で model を切り替えても会話を継続できる。

## 2. v0.1.0 に含めるもの

### 動作

- Markdown ファイルの生成。
- Markdown ファイルの編集。
- Markdown ファイルの生成、編集で発生した変更の取り消し。
- 生成、編集の結果を構造化 output として返す。
- `+` 操作による新規セッション開始。
- 履歴操作によるセッション一覧表示。
- セッション選択による復元。
- 最後のセッションの自動復元。

### UI / UX

- 下部固定の composer。
- `Enter` は改行。
- `Command + Enter` は送信。
- 送信中は送信ボタン位置を停止ボタンに切り替える。
- provider 選択はタイトル左側に置く。
- composer 内には model と provider ごとの可変 control を置く。
- context 使用率は composer 内の送信ボタン左側に円グラフで表示する。
- user message は右寄せで、本文量に応じた幅にする。
- agent response は composer とほぼ同じ幅に固定し、本文量で幅を変えない。
- thinking / execution / processing などの状態表示は本文幅ルールの対象外とする。
- Markdown 表示は code fence が途中で途切れないことを必須とする。

### provider

- 初期対応 provider は次の 4 つとする。
  - Codex CLI
  - Claude Code
  - GitHub Copilot
  - KatanAgent
- KatanAgent は `katana-chat-ui` 側が所有する agent 抽象である。
- KatanAgent の初期 runtime は Ollama を利用する。
- Ollama は provider selector に出さない。Ollama は KatanAgent の local model runtime として扱う。
- goose は KatanAgent core ではなく、拡張可能なら runtime adapter 候補として採用する。

## 3. v0.1.0 から外すもの

- 左端 hover の debug / harness panel。
- 右端 hover の output debug panel。
- 右上の設定ボタン。
- 設定画面。
- provider 接続詳細設定。
- secret 管理。
- provider usage / account usage。
- OS file picker。
- drag and drop 添付。
- clipboard 画像添付。
- 絵文字パレット対応。
- `/` command launcher。
- prompt / skill / workflow / command / hook / MCP の詳細 UI。
- file / diff / tool result の高度な詳細 view。
- 送信済み message 編集。
- 処理中の割り込み投稿。
- chat 内検索。
- 履歴横断検索。

## 4. 手動確認枠

- `harness-up` は標準 UI を載せるだけの枠にする。
- Floem、egui、GPUI は同じ標準 UI / UX を表示する。
- 完全な見た目一致までは必須にしないが、操作、配置、責務は同じにする。
- 手動確認枠は独自の会話表示、debug panel、output panel を作らない。
- screenshot 検証は headless を基本にする。
- native window を開く確認は最終確認だけにする。

## 5. 000X タスク

### 0001-archive-old-scope

旧 v0.1.0 から v0.6.0 の OpenSpec change を active から外す。

DoD:

- `openspec/changes/v0-*` が active に残っていない。
- archive 先が `openspec/changes/archive/2026-05-09-superseded-*` で残っている。
- `openspec list --json` で旧 change が active に出ない。

### 0002-remove-debug-and-settings-surfaces

左右 debug 機能と設定ボタンを削除する。

DoD:

- 左端 hover で harness / debug panel が出ない。
- 右端 hover で output debug panel が出ない。
- 右上に設定ボタンが出ない。
- 標準 UI の入力、送信、provider 切り替えを妨害する overlay がない。
- 回帰テストが debug / settings surface の再混入を検知する。

### 0003-provider-selector-header

provider 選択を title 左側の UI に固定する。

DoD:

- composer 内に provider selector がない。
- title 左側に現在 provider と pulldown 表示がある。
- provider 候補は利用可能なものだけ出る。
- Ollama が provider 候補に出ない。

### 0004-composer-input-contract

composer の入力契約を安定化する。

DoD:

- placeholder は入力値ではない。
- 日本語 IME 入力が壊れない。
- `Enter` は改行する。
- `Command + Enter` は 1 回だけ送信する。
- 送信後に draft が消え、session は消えない。

### 0005-composer-control-layout

composer 内 control の配置を固定する。

DoD:

- model selector と provider ごとの control が下揃えで並ぶ。
- context 使用率が送信ボタン左側に円グラフで表示される。
- SVG icon button に不要な二重枠が出ない。
- 横幅が狭くても入力欄が消えない。

### 0006-message-layout-contract

会話表示の幅と位置を固定する。

DoD:

- user message は右寄せで、短文なら短い幅になる。
- user message の文字は上下中央に揃う。
- agent response は composer とほぼ同じ幅で表示される。
- agent response は本文量で幅が変わらない。
- thinking / execution / processing は agent response 幅ルールに含めない。

### 0007-provider-runtime-model

provider と runtime の境界を実装する。

DoD:

- Codex CLI、Claude Code、GitHub Copilot、KatanAgent が provider 候補になる。
- KatanAgent は Ollama runtime を持てる。
- Ollama は provider ではなく runtime として扱われる。
- 利用不可 provider は選択できない。
- fallback で別 provider に切り替えない。

### 0008-katanagent-markdown-file-generation

KatanAgent で Markdown ファイル生成を扱う。

DoD:

- Markdown 作成指示から file candidate output が生成される。
- `./tmp` 配下など許可された場所だけを書き込み候補にできる。
- 許可外 path は失敗理由を返す。
- chat 本文と output contract が分離している。

### 0009-katanagent-markdown-file-editing

KatanAgent で Markdown ファイル編集を扱う。

DoD:

- Markdown 編集指示から diff または file update candidate が生成される。
- 元ファイル、変更後内容、差分概要を output に持てる。
- 物理書き込みは host / adapter 境界で行う。
- chat 本文に debug JSON を混ぜない。

### 0010-markdown-change-undo

Markdown 生成、編集で発生した変更を元に戻す。

DoD:

- 生成した Markdown ファイルを取り消し対象として記録できる。
- 編集した Markdown ファイルの変更前内容または逆差分を保持できる。
- 取り消し操作で対象変更だけを元に戻せる。
- 取り消し結果を output contract に持てる。
- 取り消しできない状態では理由を返し、別の変更へ fallback しない。

### 0011-session-history-store

セッション履歴を保存する。

DoD:

- session id、title、provider、model、message、output reference を保存できる。
- harness では `./tmp/harness-${provider}` 配下に保存される。
- 新規セッション開始で現在 session と draft が分離される。
- 再起動時に最後の session を復元できる。

### 0012-session-history-ui

履歴一覧と復元 UI を実装する。

DoD:

- 右上の履歴操作から session 一覧を開ける。
- 一覧に title、provider、updated at、preview が出る。
- session を選ぶと会話が復元される。
- 復元時に provider へ再送信しない。

### 0013-markdown-rendering-minimum

Markdown 表示の最低品質を満たす。

DoD:

- code fence が途中の ``` で途切れない。
- 見出し、段落、list、code block が崩れない。
- raw HTML は描画せず text として扱う。
- renderer は将来差し替え可能な境界を持つ。

### 0014-three-host-ui-conformance

Floem、egui、GPUI の表示差を許容範囲に収める。

DoD:

- 3 host が同じ render model を使う。
- 3 host で header、thread、composer の責務が同じである。
- 3 host screenshot の比較で、標準 UI として同じ操作対象に見える。
- host 側に独自 debug / output UI がない。

### 0015-headless-screenshot-gate

目視前の screenshot 検証を自動化する。

DoD:

- headless screenshot で 3 host の主要レイアウトを検証する。
- native window を通常検証で開かない。
- screenshot 差分で composer 欠落、provider 誤表示、debug 混入を検知する。

### 0016-final-ui-polish

v0.1.0 の最後に UI 調整だけをまとめて行う。

DoD:

- button、spacing、text vertical align、message width を最終調整する。
- 既存の動作テストを壊さない。
- 人間の手動確認に出せる状態である。

## 6. 完了判定

v0.1.0 は、0001 から 0016 までが完了し、次の検証が通った時だけ完了とする。

- `just fmt`
- `just lint`
- `just ast-lint`
- `just unit-test`
- `just host-e2e`
- `just harness-screenshot-matrix-check`

手動確認は最後に行う。手動確認で初めて分かる不具合が大量に出る状態は不合格とする。
