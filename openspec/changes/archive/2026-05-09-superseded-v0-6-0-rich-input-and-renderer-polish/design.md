## Context

v0.6.0 は体験の磨き込みを扱うが、v0.1.0 で固定した composer、provider、streaming、thinking、output contract を変更しない。

入力機能は OS や host の違いが出やすいため、core は入力 event と attachment metadata を扱い、実ファイル解決や画像 decode は host callback に委譲する。

## Goals

- 画像貼り付けと絵文字挿入で draft、cursor、IME state を壊さない。
- message timing を render model と UI 表示の両方で扱う。
- provider selector に公式または明示登録された icon を表示する。
- Markdown renderer を `katana-document-viewer` に差し替えられる。

## Non-Goals

- `katana-document-viewer` 自体をこの change で実装すること。
- provider icon を欠落時に汎用記号へ silently fallback すること。
- OS clipboard の private API を kcu core に直書きすること。

## Input Extension

`RichInputController` は次を扱う。

- `ClipboardImageAttachment`
- `EmojiInsertion`
- `ExternalPathAttachment`

すべて `AttachmentResolver` interface を通り、OS ファイル、host resource、virtual attachment を区別する。

## Timing

`MessageTiming` は次を持つ。

- user sent at
- agent started at
- agent completed at
- agent failed at
- agent stopped at
- elapsed duration

UI は実行中だけ elapsed duration を更新し、完了 / 失敗 / 停止で固定する。

## Provider Icon

`ProviderIconRegistry` は provider id と icon asset を対応させる。

- 公式 icon が利用可能な provider は公式 icon を使う。
- 公式 icon がない場合は `IconMissing(provider_id)` を返す。
- `+` や汎用コード記号を provider icon として代用しない。

## Markdown Renderer

`MarkdownRenderer` interface を維持し、`comrak` adapter と `katana-document-viewer` adapter を差し替え可能にする。

v0.1.0 の safe subset と code fence contract は renderer 差し替え後も維持する。

## Zed Reference Mapping

- `crates/agent_ui/src/message_editor.rs`: paste external path、pasted image、cursor 位置、mention link。
- `crates/agent_ui/src/mention_set.rs`: file / image / selection / diagnostics / git diff mention。
- `crates/agent_ui/src/model_selector.rs`: model selector icon 表示。
- `crates/agent_settings/src/agent_settings.rs`: profile、permission、thinking 設定。

## Verification Strategy

- clipboard image、emoji、cursor、IME の regression test を追加する。
- provider icon registry の snapshot test を追加する。
- Markdown renderer adapter の contract test を追加する。
