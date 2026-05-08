## Context

output は利用側へ渡す付随情報である。同時に、標準 UI 自身も output を読んで、生成物の要約、差分候補、tool 結果、権限要求を表示する必要がある。

v0.5.0 では、標準 UI と host を密結合させず、表示用 render model と操作 intent のみを定義する。

## Goals

- file / diff / tool result / permission request を標準 UI の output extension view として表示できる。
- file 書き込み、差分適用、コマンド実行、権限承認の実処理は host / provider に委譲する。
- 状態表示と output view を関連付けられる。
- output JSON debug surface と production UI を混ぜない。

## Non-Goals

- host の差分 viewer や file viewer を kcu core に実装すること。
- kcu core が OS ファイルを書き換えること。
- provider 固有 tool 名で UI を直書き分岐すること。

## Output Render Model

```
OutputExtensionView
  id
  source_message_id
  generation_id
  kind
  status
  summary
  actions
```

`kind` は次を持つ。

- `FileCandidate`
- `DiffCandidate`
- `ToolResult`
- `PermissionRequest`
- `CommandExecution`
- `ReadResult`
- `FetchResult`

## Action Intent

UI は action button を表示するが、処理そのものは実行しない。

- `OpenFile`
- `OpenDiff`
- `ApplyDiff`
- `RejectDiff`
- `ApproveTool`
- `RejectTool`
- `CopyOutput`

すべての intent は `output_id` と `source_message_id` を持つ。

## Zed Reference Mapping

- `crates/acp_thread/src/acp_thread.rs`: ToolCall、ToolCallStatus、ToolCallContent。
- `crates/acp_thread/src/diff.rs`: pending / finalized diff。
- `crates/acp_thread/src/terminal.rs`: terminal output。
- `crates/agent/src/tools/`: read、edit、fetch、terminal などの tool result contract。
- `crates/agent_ui/src/conversation_view.rs`: permission button、tool call 表示、agent diff 入口。

## Verification Strategy

- output render model の unit test で file / diff / tool / permission を固定する。
- action intent test で UI が処理を実行せず intent だけを返すことを検証する。
- AST lint で provider ID 直書き分岐と debug JSON の本文混入を禁止する。
