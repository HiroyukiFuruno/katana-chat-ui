## Context

Zed は会話表示、message editor、thread store、ACP thread を分けている。kcu でも同様に、UI 表示、session 保存、provider event、output を同じ構造体へ詰め込まない。

v0.4.0 は履歴と会話制御を追加するが、provider 固有実装には依存しない。provider が rewind / cancel / queue を持たない場合は、できるように見せず、明示的に unsupported state を返す。

## Goals

- session 履歴を保存し、一覧表示し、UI から復元できる。
- 現在 chat 内検索と履歴横断検索を提供する。
- 送信済み message 編集時に、後続 response / thinking / output を無効化できる。
- 処理中の追加投稿を steering / interrupt / queue として表現できる。
- 古い generation の event を current thread に混ぜない。

## Non-Goals

- file / diff / tool result の詳細 UI を作ること。
- provider ごとの private API に合わせた巻き戻し処理を core に直書きすること。
- session 履歴の保存先を特定の DB / file format に固定すること。

## Architecture

```
ConversationControlSurface
  -> ConversationEditController
  -> ConversationInterruptController
  -> SessionHistoryStore
  -> ConversationSearchIndex
  -> ProviderEventRouter
```

### Session History

`SessionHistoryStore` は `SessionSnapshot` を保存、一覧、復元する interface である。

`SessionSnapshot` は次を持つ。

- session id
- title
- message list
- thinking log references
- output references
- provider / runtime selection
- attachments metadata
- draft
- scroll restore state

保存形式は implementation detail とし、UI は file path や DB schema を知らない。

### History List

`SessionHistoryListItem` は次を持つ。

- session id
- title
- provider id / display label
- updated at
- preview text
- status
- unread or changed marker

履歴一覧 UI は `SessionHistoryStore` の list API だけを使い、保存形式を知らない。

### Search

`ConversationSearchIndex` は current session search と history search を分ける。

- current chat search: 開いている session の message、thinking summary、output summary を検索する。
- history search: 保存済み session の title、message、metadata、output summary を検索する。

検索結果は session id、message id、matched range、preview、score を持つ。UI は検索結果から対象 session と message へ移動できる。

### Edit / Rewind

`ConversationEditController` は送信済み user message を編集可能 state にし、commit 時に次を行う。

- 編集対象 message 以降の response / thinking / output を invalidated にする
- provider が rewind を support する場合だけ rewind intent を出す
- support しない場合は unsupported state を返し、勝手に fallback しない

### Interrupt / Steering / Queue

`ConversationInterruptController` は処理中の追加投稿を次のいずれかとして扱う。

- `SteerMessage`: 進行中 generation に追加指示を送る
- `InterruptAndSend`: 進行中 generation を止め、新しい message で再開する
- `QueueMessage`: 現在の generation 完了後に送る

provider が対応していない intent は UI に disabled reason として返す。

### Stale Generation Guard

すべての provider event は `generation_id` を持つ。

UI state は current generation と一致しない event を本文、thinking、output へ反映しない。

## Zed Reference Mapping

- `crates/agent_ui/src/conversation_view.rs`: message editing、queued message、interrupt、title editor。
- `crates/acp_thread/src/acp_thread.rs`: cancel、rewind、AgentThoughtChunk、ToolCall、usage update。
- `crates/agent/src/thread.rs`: draft prompt、snapshot、tool input / output。
- `crates/agent/src/thread_store.rs` と `crates/agent/src/db.rs`: thread save / load。
- `crates/agent_ui/src/thread_metadata_store.rs`: thread metadata、session id、reload。

## Verification Strategy

- state unit test で save / restore、edit invalidate、interrupt intent、stale generation guard を固定する。
- UI adapter test で v0.1.0 の composer / thread 表示を壊していないことを確認する。
- manual harness では、履歴復元、message 編集、処理中追加投稿を別シナリオとして確認する。
