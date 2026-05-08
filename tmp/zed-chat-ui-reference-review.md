# Zed chat UI 参照レビュー

## 結論

v0.1.0 の責務分離は、Zed の現在の Agent UI 実装と大きく矛盾していない。
Zed 側も、会話状態、入力 editor、mention/context、diff review、terminal/tool 実行、permission を分けて扱っている。

確認対象は `/tmp/zed-kcu-reference` に clone した `zed-industries/zed` の `main` である。
ユーザー指定の一部ファイル名は現在の tree では同名ファイルとして存在せず、以下の相当ファイルで確認した。

## 参照した Zed ソース

- `/tmp/zed-kcu-reference/crates/agent_ui/src/conversation_view.rs`
  - `Conversation` は session ごとの thread と permission request を管理している。
  - `AcpThreadEvent::NewEntry` / `EntryUpdated` / `Stopped` / `Error` / `TitleUpdated` / `TokenUsageUpdated` などを UI 更新イベントとして扱っている。
  - `MessageEditor` を会話本体から生成し、送信や queued message の再開を event として受けている。
- `/tmp/zed-kcu-reference/crates/agent_ui/src/message_editor.rs`
  - `SessionCapabilities` が画像、埋め込み context、available command を持つ。
  - `MessageEditorAddon` が modifier send 設定を key context へ渡す。
- `/tmp/zed-kcu-reference/crates/agent_ui/src/mention_set.rs`
  - file、symbol、thread、fetch、diagnostics、git diff などを `MentionUri` として扱い、内容解決は task に分離している。
- `/tmp/zed-kcu-reference/crates/agent_ui/src/agent_diff.rs`
  - agent diff は thread から分離された `AgentDiffPane` として workspace item に展開される。
  - keep / reject は editor と thread を受けて review operation として実行される。
- `/tmp/zed-kcu-reference/crates/agent/src/agent.rs`
  - internal `Thread` と `AcpThread` を同じ session に保持する。
  - terminal は `ThreadEnvironment::create_terminal` から作成され、`AcpTerminalHandle` 経由で output / wait / kill を扱う。
- `/tmp/zed-kcu-reference/crates/agent/src/thread.rs`
  - tool permission は settings を参照して authorize / prompt へ分岐する。
- `/tmp/zed-kcu-reference/crates/agent/src/tools/terminal_tool.rs`
  - terminal tool は command / cwd / timeout を input contract として扱う。

## kcu への写像

- 会話状態:
  - Zed: `ConversationView` と `AcpThreadEvent`
  - kcu: `ChatSession` と provider event / render model
- 入力:
  - Zed: `MessageEditor` と `SessionCapabilities`
  - kcu: `ChatInputDraft`、composer render model、slash launcher
- context / attachment:
  - Zed: `MentionSet` と `MentionUri`
  - kcu: `Attachment`、host callback、command launch intent
- diff / file / tool result:
  - Zed: `AgentDiffPane`、terminal tool、permission loop
  - kcu: `ChatOutputKind`、`HostActionIntent`、output extension slot
- settings:
  - Zed: `AgentSettings`
  - kcu: typed settings model と settings reference merge intent

## v0.1.0 判定

v0.1.0 では Zed の全機能コピーはしない。
ただし、標準 UI が file / diff / tool result を本文へ混ぜず、output extension と host action intent として扱う方針は Zed の責務分離と整合している。

実 LLM による編集系は token 消費を避けるため mock provider event で確認する。
