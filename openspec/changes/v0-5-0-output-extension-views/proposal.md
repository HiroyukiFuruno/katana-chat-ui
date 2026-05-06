## Why

v0.1.0 では output を構造化して返す contract と拡張 slot だけを用意する。file、diff、tool result、permission request の詳細表示まで同時に作ると、標準 chat UI の成立確認が遅れる。

ただし Codex や Zed の AI chat では、ファイル変更、差分、terminal、権限確認が会話体験の中心になる。v0.5.0 では、これらを標準 UI の output extension view として扱えるようにする。

## What Changes

- file candidate、diff candidate、tool result、permission request の render model を追加する。
- command 実行中、read、editing、fetch などの状態表示と output view を接続する。
- kcu は表示と操作 intent を持つが、実際のファイル書き込み、差分適用、コマンド実行、権限承認は host / provider に委譲する。
- output view は chat 本文とは別の拡張領域として描画し、debug JSON と混ぜない。

## Capabilities

### New Capabilities

- `output-extension-view`: output を標準 UI の拡張領域へ表示する
- `file-candidate-view`: 生成 / 更新候補ファイルを表示する
- `diff-candidate-view`: 差分候補を表示する
- `tool-result-view`: コマンドや外部 tool の結果を表示する
- `permission-request-view`: 権限要求と user decision intent を表示する

## Impact

- `crates/katana-chat-ui/src/output/` — output render model と action intent
- `crates/katana-chat-ui/src/surface/` — output extension slot
- `crates/katana-chat-ui-floem/`、`egui`、`gpui` adapter — extension view 表示
- `docs/chat-ui-spec.ja.md` — output と状態表示の詳細化
