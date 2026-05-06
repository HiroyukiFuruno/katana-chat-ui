## Why

v0.1.0 では標準 chat UI の成立を優先し、入力と表示の高度な磨き込みは分離する。ただし実利用では、画像貼り付け、絵文字、時間表示、provider icon、Markdown renderer の品質が chat UI の信頼性に直結する。

v0.6.0 では、v0.1.0 から v0.5.0 までの contract を壊さず、rich input と renderer polish を追加する。

## What Changes

- クリップボード画像添付を追加する。
- macOS 絵文字パレットからの挿入を壊さない。
- 試行時間、送信時刻、応答開始 / 完了 / 失敗 / 停止時刻を表示する。
- provider selector の候補行に provider icon を表示する。
- `comrak` 暫定 renderer から `katana-document-viewer` へ差し替える境界を実装する。

## Capabilities

### New Capabilities

- `clipboard-image-attachment`: 画像貼り付け添付
- `emoji-input`: macOS 絵文字パレット入力
- `message-timing`: 試行時間と送受信時刻
- `provider-selector-icons`: selector 候補行の provider icon
- `document-viewer-markdown-renderer`: `katana-document-viewer` 連携

## Impact

- `crates/katana-chat-ui/src/input/` — clipboard image、emoji、IME guard
- `crates/katana-chat-ui/src/message/` — timing metadata
- `crates/katana-chat-ui/src/vendor_ui/` — provider icon registry
- `crates/katana-chat-ui/src/markdown/` — renderer adapter
- framework adapter crates — UI 表示
