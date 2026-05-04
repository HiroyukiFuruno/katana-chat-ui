## ADDED Requirements

### Requirement: chat 状態は katana-chat-ui neutral crate が管理しなければならない

システムは、chat session（message 履歴・streaming buffer・turn 管理）と autofix proposal（診断 + LLM 提案 + diff 状態）を `katana-chat-ui` neutral crate に閉じ、egui 等の UI フレームワークから独立して管理しなければならない（MUST）。

#### Scenario: chat session を更新する

- **WHEN** ホストが `ChatSession::push_user_turn()` を呼ぶ
- **THEN** message 履歴に user turn が追加される
- **THEN** `katana-chat-ui` の `cargo tree` に `egui` は含まれない

#### Scenario: autofix proposal を保持する

- **WHEN** LLM が autofix 候補を返す
- **THEN** `AutofixProposal` が診断・元 content・提案 content・diff 状態を保持する

### Requirement: chat panel widget は katana-chat-ui-egui が egui 実装を提供しなければならない

システムは、egui ベースの chat サイドパネル widget と autofix diff surface widget を `katana-chat-ui-egui` impl crate として提供しなければならない（MUST）。KatanA は `ChatPanelWidget::show(ui, session, config)` を呼ぶだけで chat UI を配置できる。

#### Scenario: chat サイドパネルを表示する

- **WHEN** ホストが `ChatPanelWidget::show(ui, &mut session, &config)` を呼ぶ
- **THEN** egui ui に chat サイドパネル（履歴・draft・送信ボタン）が描画される
- **THEN** streaming token が逐次表示される

#### Scenario: autofix diff preview を表示する

- **WHEN** ホストが autofix proposal を持つ状態で diff surface widget を呼ぶ
- **THEN** 元 content と提案 content の差分が preview される
- **THEN** apply / cancel が user 操作で選べる

#### Scenario: chat panel settings をホストに統合する

- **WHEN** ホストが `ChatPanel::builder().settings_slice(json).on_settings_changed(cb).build()` で初期化する
- **THEN** `katana-chat-ui` は渡されたスライスを初期値として利用する
- **THEN** 設定変更時に `on_settings_changed` でスライスをホストに返す
- **THEN** スキーマ外のキーはスライスに含まれていても無視される
