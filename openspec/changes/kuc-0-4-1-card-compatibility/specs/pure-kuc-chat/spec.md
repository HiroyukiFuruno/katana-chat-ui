## MODIFIED Requirements

### Requirement: 依存更新は品質基準を維持する

KCU SHALL 全manifest/lockを最新化し、既存check、AST lint、coverage基準を変更せず検証する。KUC更新によって既存メッセージの幅、視覚基準、hit test、またはMarkdown code spanのmodel/raster契約が公開APIだけで維持できない場合、KCU SHALL 必要な上流契約を記録し、公開修正版を取得して全品質ゲートを通すまでリリースを完了扱いにしてはならない。

#### Scenario: 更新版に非互換性がある
- **WHEN** 最新依存で既存品質ゲートが失敗する
- **THEN** 互換修正を検証するか更新不採用の理由と残存版を明示する

#### Scenario: KUCの公開APIに互換性を維持する手段がない
- **WHEN** KUC更新後に既存の幅、視覚、hit test、またはcode span model/raster契約を公開APIだけで満たせない
- **THEN** KCUは必要な上流契約をIssueへ記録し、公開修正版と全品質ゲートの成功を確認するまでリリースを完了扱いにしない

#### Scenario: インデントされたCard本文のhit幅を維持する
- **WHEN** 本文をCard内で16pxインデントして既存画像基準の座標に描画する
- **THEN** KCUは同じstable本文nodeへ公開 `hit_outset_left(16)` を適用し、viewport hit幅560pxと既存draw/layout/scroll測定を同時に維持する

#### Scenario: code spanの意味と物理font faceを分離する
- **WHEN** Markdownのcode spanが `monospace = true` をmodel契約として持ち、既存画像基準が本文互換の物理font faceを要求する
- **THEN** KCUは本文System faceを変更せず、公開の独立code raster設定をcode rendererだけへ適用する
- **AND** draw、measurement、wrap、hitは同じcode renderer設定を利用する

### Requirement: provider-free Storybookを提供する

KCU SHALL `just storybook` で、公開KUC runtimeを通るprovider-freeの対話Storybookをrelease buildで起動する。Storybookは実LLM providerや有償契約を必要とせず、Codex CLI、Claude Code、GitHub Copilot、KatanAgentの選択肢を表示し、入力・送信・履歴・model選択を決定論的fixtureで確認可能にする。

#### Scenario: 開発者が標準UIを操作確認する
- **WHEN** `just storybook` を実行する
- **THEN** KCU Storybookのnative windowが開き、公開KUC runtimeによるfixture UIを操作できる

#### Scenario: 自動検証が対話起動を置き換えない
- **WHEN** `just storybook-check` を実行する
- **THEN** entrypointとfixtureのcontroller unit契約を検査し、hostのnative windowは起動しない
- **AND** controller unit単独を実windowの入力routingの証拠として扱わない

#### Scenario: native smokeは他の作業画面を奪わず必須検査に含まれる
- **WHEN** `just storybook-window-smoke` または通常の完全 `just check` を実行する
- **THEN** 既存Linux/aarch64 containerの専用Xvfb上で実Storybookのnative frameを検査し、失敗時は検査全体を失敗にする
- **AND** host DISPLAYやX socketを渡さず、macOSのnative windowを自動起動しない
- **AND** 物理IMEと最終人間UIの受入方針を変更しない

## ADDED Requirements

### Requirement: 標準hostは実行できるoutput操作だけを提示する

KCU SHALL 標準hostのmessage・message list・handoffの各output surfaceで、現在の候補状態が許可し標準hostが実装するCreateFile/ApplyDiff/UndoChangeだけを提示する。未対応のOpenPreview/Copy/Approve/Rejectを有効表示してはならない。公開libraryのintent/enum/callbackと保存payload/status、別経路のclipboard/vendor permissionは維持する。

#### Scenario: 標準hostが未対応output intentを受け取る
- **WHEN** File/Diff候補や他のoutput kindがlibraryのhost action intentsを持つ
- **THEN** 標準host surfaceの3投影は未対応操作を提示せず、本文・候補状態・公開library intentsを変更しない

#### Scenario: 適用と取消は現在の候補状態を維持する
- **WHEN** Ready/PendingHost、Applied、Failed/Rejected/RevertedのFile/Diff候補を表示する
- **THEN** 前二者はそれぞれ適用・取消だけを提示し、後者は未対応previewへ置き換えず操作なしとする

### Requirement: 共通composerは削除対象の範囲を維持する

KCU SHALL 公開KUCの操作だけで共通composerのforward Deleteを処理し、現在の選択範囲またはcaretから次のgrapheme境界までだけを削除する。KCU SHALL 末尾または空文字でDeleteを受けてもdraftを変更しない。Backspaceも選択範囲があればその範囲だけを削除し、なければ従来の直前grapheme削除を維持する。3つのadapterで同じ入力契約を維持する。

#### Scenario: caretの前方に文字がない
- **WHEN** 空のdraftまたは非空draftの末尾でDeleteを押す
- **THEN** draftとcaretは変わらず、draft変更を通知しない

#### Scenario: 選択範囲または次のgraphemeを削除する
- **WHEN** 前向き・逆向きのselectionがある状態、または文字列途中でDeleteを押す
- **THEN** selectionがあればその範囲だけを、なければ次のgraphemeだけを削除し、残りの本文を維持する
- **AND** 結合文字、emoji、改行の境界は公開KUCの文字境界処理を利用する

#### Scenario: Backspaceで選択範囲を削除する
- **WHEN** 前向き・逆向きのselectionまたは全選択がある状態でBackspaceを押す
- **THEN** 選択範囲だけを削除してcaretを範囲先頭へ戻し、未選択時の直前grapheme削除と空文字no-opを維持する
