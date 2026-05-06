## Why

v0.1.0 では標準 chat UI の基礎と手動確認成立を優先するため、session 履歴、送信済み message 編集、処理中の割り込みは v0.4.0 に分離する。

これらを曖昧に v0.1.0 へ混ぜると、streaming、output、provider event の再開点が不明確になり、古い応答 chunk や古い output が会話へ混入する。

## What Changes

- session ごとの履歴保存、履歴一覧表示、UI からの復元を追加する。
- 現在 chat 内検索と、保存済み履歴を含めた履歴横断検索を追加する。
- 送信済み user message を編集し、その message 以降の agent response と output を無効化して再開できるようにする。
- 処理中の追加投稿を steering / interrupt / queue として扱い、通常送信と区別する。
- generation id を導入し、古い request から届いた chunk / output / thinking event を current thread へ混ぜない。

## Capabilities

### New Capabilities

- `chat-session-history`: session 履歴の保存、一覧、復元
- `chat-history-list`: title、provider、最終更新時刻、preview、状態を持つ履歴一覧
- `chat-search`: 現在 chat 内検索と履歴横断検索
- `message-edit-rewind`: 送信済み message 編集と再開点管理
- `conversation-interrupt-steering`: 処理中の追加投稿、割り込み、queue
- `stale-generation-guard`: 古い generation からの event 混入防止

## Impact

- `crates/katana-chat-ui/src/session/` — session snapshot、history store、restore contract
- `crates/katana-chat-ui/src/message/` — message edit state、invalidated range
- `crates/katana-chat-ui/src/surface/` — edit / interrupt / queue の UI intent
- `crates/katana-chat-ui/src/output/` — output と generation id の関連付け
- `crates/katana-chat-ui-floem/`、`egui`、`gpui` adapter — UI 表示の実装
- `openspec/roadmap.md` — v0.4.0 scope
