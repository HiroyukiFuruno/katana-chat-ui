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
