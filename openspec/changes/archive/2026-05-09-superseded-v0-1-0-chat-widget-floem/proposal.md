## Why

初期段階で kcu の境界を固定する。ここで `egui` や親アプリ前提が混ざると、後続の提供元（vendor）追加、ACP 接続、secret 管理、usage 表示がすべて host app 依存になる。

v0.1.0 は「標準的な AI チャット UI として何を受け取り、何を表示し、どこまで host に任せるか」を決め、kcu が提供する標準 UI 部品を実装する foundation change とする。
MVP でも、単なる問答 UI ではなく、LLM の生成物を host が安全に扱うための output handoff contract を含める。
API だけを使って独自 UI を作る利用方法は許容するが、それは標準 UI に満足できない場合の拡張手段であり、v0.1.0 の完了条件ではない。

## What Changes

### `katana-chat-ui`

- UI framework 非依存の `ChatSession`、`ChatMessage`、`ChatInputDraft`、`Attachment`、`ChatRenderModel` と、標準 UI 部品が読む `ChatUiSurface` を定義する。
- 入力は text、file attachment、image attachment、path drop を扱う。
- path drop は host から file content を受け取り、ACP の embedded resource 相当の context attachment として保持する。
- Markdown は `comrak` を暫定採用し、見出し、段落、強調、取り消し線、link、inline code、code block、blockquote、通常 list、番号付き list、task list、table を対象にする。raw HTML は text 扱い、画像 Markdown は安全な alt/link 表示に限定する。
- user / assistant / tool / system の message role ごとに visual intent を分ける。
- theme は color token / spacing token / typography token として受け取り、core は色名だけを保持する。
- button icon は SVG として表示し、host から受け取った SVG override を render model / UI surface へ反映できる。
- UI 文言は text catalog を通す。MVP は英語だけでもよいが、後続で locale を追加できる構造にする。
- vendor / provider capability によって model selector、mode、thinking、permission などの UI affordance を出し分けられる構造にする。表示/非表示は公式ドキュメント根拠付きの `VendorFactRegistry` から決める。
- 現在の context token usage を表示できる render model を持つ。
- account usage と provider usage の取得は v0.2.0 に分離するが、v0.1.0 では取得不可状態を扱える枠を持つ。
- generated text、code block、file candidate、diff candidate、tool result、permission request を output として分類する。
- kcu は agent event から本文、thinking、output、host action intent への分類を所有する。
- 物理的な file 書き込み、diff 適用、tool プロセス実行は host / adapter が行い、その結果を kcu の event / output contract に戻す。
- kcu 標準 UI は output を本文とは別の contract として読み、生成物の要約、実行状態、host action intent を扱う。debug JSON や検証用差分カードを標準 thread に常時混ぜず、詳細は host の差分ビューや file view へ渡す。
- 右上トグルから開く設定画面を標準 UI の一部として提供する。
- 入力欄で `/` を入力した時に、prompt / skill / workflow / command を起動するための launcher contract を提供する。
- 編集できる AI 作業者としての提供元（agent provider）と、Ollama のようなローカルモデル実行基盤（local model runtime）を分ける。Ollama は会話検証に使えるが、file edit / terminal 実行権限を持つ provider として扱わない。

### `katana-chat-ui-floem`

- kcu が提供する標準 Floem UI 実装として、実際に描画できる chat widget を提供する。
- IME、multiline input、attachment tray、markdown rendering、usage meter、provider controls、stop / send action を表示する。
- button は `ChatIconSet` の SVG を描画し、override 後の SVG を使う。
- UI 文言は text catalog を参照する。
- vendor capability によって composer 周辺の操作表示を変えられる。標準 UI は `VendorControlRenderModel` を読み、widget 内で vendor ID の文字列分岐をしない。
- output の実体は kcu の contract として保持し、本文とは別の extension surface / host action intent として扱う。物理的な file 書き込み、diff 適用、tool プロセス実行は host / adapter が行い、結果を kcu に戻す。確認補助は `debug: true` のときだけ右端 output handoff hover として表示する。
- Floem crate は API descriptor だけではなく、標準 UI 実装である。

### docs

- `docs/settings-schema.json` は UI 表示設定だけを扱う。
- 接続設定、secret、provider usage、account usage、MCP server 設定は v0.2.0 に分離する。
- prompt / skill / workflow / command / hook の実エントリ供給は v0.3.0 に分離する。
- 実動作検証用 UI harness は `crates/` に置かず、`tools/e2e-host-app/` のような非公開の外部 host fixture に分離する。
- 外部 host fixture は kcu を downstream dependency として実際に取り込み、起動・描画・入力・添付・usage 表示を E2E で検証する。
- 人間が触る UI/UX 確認用 host は `tools/manual-host-egui/`、`tools/manual-host-floem/`、`tools/manual-host-gpui/` に分ける。これらは標準 UI 部品を載せる枠であり、host 側で独自 chat UI を実装しない。
- 手動 LLM 検証で低コスト runtime を使う場合は、Ollama を agent provider の runtime として扱う。有料 LLM API token は消費しない。自動検証は LLM 呼び出しを行わず、mock agent event を使う。

## Out of Scope for v0.1.0

以下は v0.1.0 の完了条件に含めない。対応 version は `openspec/roadmap.md` と該当 change に委譲する。

- session 履歴保存 / 復元 — v0.4.0。
- 送信済み message 編集と、その時点からの再開 — v0.4.0。
- 処理中の steering / interrupt / queue — v0.4.0。
- file / diff / tool result / permission request の詳細 UI — v0.5.0。
- クリップボード画像添付、絵文字、試行時間、送受信時刻、provider selector icon、`katana-document-viewer` 連携 — v0.6.0。

## Capabilities

### New Capabilities

- `chat-ui-foundation`: framework-neutral chat state と render model
- `chat-input-attachments`: text / file / image / path drop attachment
- `chat-markdown-subset`: `comrak` ベースの安全な Markdown rendering
- `chat-theme-and-icons`: theme token と button SVG override
- `chat-i18n-text-catalog`: UI 文言の locale 差し替え
- `chat-settings-screen`: 右上トグルから開く標準設定画面 shell
- `chat-slash-launcher`: 入力欄の `/` 起動 contract
- `chat-vendor-ui-capabilities`: vendor / provider capability による UI affordance の出し分け
- `chat-usage-surface`: context token / account usage 表示枠
- `chat-output-handoff`: 生成物と host action intent の分類

### Inherited from v0.0.1

- `acp-interface`: provider / request / response の neutral contract
- `ollama-runtime`: local LLM runtime。v0.1.0 では agent provider が利用する runtime として扱い、編集・コマンド実行できる agent provider としては表示しない
- `agent-provider-adapter`: 既存 Rust 製 agent や ACP / CLI adapter からの `Chunk`、`ThinkingChunk`、`Output`、`Complete`、`Failed` を kcu session へ反映する境界

## Impact

- `crates/katana-chat-ui/` — framework-neutral core と標準 UI surface contract
- `crates/katana-chat-ui-floem/` — 標準 Floem UI crate
- `tools/e2e-host-app/` — 配布対象に含めない外部 E2E host fixture
- `tools/manual-host-egui/` — 人間が触る UI/UX 確認用 egui host fixture
- `tools/manual-host-floem/` — 人間が触る UI/UX 確認用 Floem host fixture
- `tools/manual-host-gpui/` — 人間が触る UI/UX 確認用 GPUI host fixture
- `docs/settings-schema.json` — UI 表示設定
- `Cargo.toml` — `egui` dependency を除去
