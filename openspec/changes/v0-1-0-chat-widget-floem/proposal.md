## Why

初期段階で kcu の境界を固定する。ここで `egui` や親アプリ前提が混ざると、後続の提供元（vendor）追加、ACP 接続、secret 管理、usage 表示がすべて host app 依存になる。

v0.1.0 は「標準的な AI チャット UI として何を受け取り、何を表示し、どこまで host に任せるか」を決める foundation change とする。

## What Changes

### `katana-chat-ui`

- UI framework 非依存の `ChatSession`、`ChatMessage`、`ChatInputDraft`、`Attachment`、`ChatRenderModel` を定義する。
- 入力は text、file attachment、image attachment、path drop を扱う。
- path drop は host から file content を受け取り、ACP の embedded resource 相当の context attachment として保持する。
- Markdown subset は code block、inline code、blockquote、ordered / unordered list、link を対象にする。
- user / assistant / tool / system の message role ごとに visual intent を分ける。
- theme は color token / spacing token / typography token として受け取り、core は色名だけを保持する。
- icon は SVG asset id として扱い、host override を可能にする。
- context token usage と account usage を表示できる render model を持つ。

### `katana-chat-ui-floem`

- Floem reference implementation として `ChatPanelView` を提供する。
- IME、multiline input、attachment tray、markdown rendering、usage meter、settings trigger、stop / send action を表示する。
- Floem crate は任意の reference UI であり、kcu core の contract ではない。

### docs

- `docs/settings-schema.json` は UI 表示設定だけを扱う。
- 接続設定、secret、account usage は v0.2.0 に分離する。
- 実動作検証用 UI harness は `crates/` に置かず、`tools/e2e-host-app/` のような非公開の外部 host fixture に分離する。
- 外部 host fixture は kcu を downstream dependency として実際に取り込み、起動・描画・入力・添付・usage 表示を E2E で検証する。

## Capabilities

### New Capabilities

- `chat-ui-foundation`: framework-neutral chat state と render model
- `chat-input-attachments`: text / file / image / path drop attachment
- `chat-markdown-subset`: 安全な Markdown subset rendering
- `chat-theme-and-icons`: theme token と SVG override
- `chat-usage-surface`: context token / account usage 表示枠

### Inherited from v0.0.1

- `acp-interface`: provider / request / response の neutral contract
- `ollama-provider`: MVP provider

## Impact

- `crates/katana-chat-ui/` — framework-neutral core に整理
- `crates/katana-chat-ui-floem/` — reference UI crate
- `tools/e2e-host-app/` — 配布対象に含めない外部 E2E host fixture
- `docs/settings-schema.json` — UI 表示設定
- `Cargo.toml` — `egui` dependency を除去
