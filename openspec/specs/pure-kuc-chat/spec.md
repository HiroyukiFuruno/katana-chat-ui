# pure-kuc-chat Specification

## Purpose
公開KUCを唯一の共通UI runtimeとして使い、multi-vendor chat体験を3 hostへ一貫して提供する。
## Requirements
### Requirement: KUCが共通UIを所有する

KCU SHALL 公開レジストリのKUCを利用してチャットUIの構成・描画・操作を共通化する。native hostはOSイベントと表示接続だけを担当する。

#### Scenario: 複数frameworkへの提供
- **WHEN** egui、Floem、GPUI hostで同じチャット状態を表示する
- **THEN** 同じKUCの共通UIを通り、チャット固有の個別描画実装を必要としない

### Requirement: 既存機能とprovider独立性を維持する

KCU SHALL composerのIME・入力、送信、キャンセル、履歴、設定、vendor/control選択、添付、出力の契約を維持し、LLM vendor差分をcapability/model境界で扱う。

#### Scenario: UI移行に必要な公開APIが不足する
- **WHEN** 公開KUCで既存操作を維持できない
- **THEN** 未対応の機能と必要な上流契約を記録し、純KUC移行を完了と報告しない

### Requirement: provider-free Storybookを提供する

KCU SHALL `just storybook` で、公開KUC runtimeを通るprovider-freeの対話Storybookをrelease buildで起動する。Storybookは実LLM providerや有償契約を必要とせず、Codex CLI、Claude Code、GitHub Copilot、KatanAgentの選択肢を表示し、入力・送信・履歴・model選択を決定論的fixtureで確認可能にする。

#### Scenario: 開発者が標準UIを操作確認する
- **WHEN** `just storybook` を実行する
- **THEN** KCU Storybookのnative windowが開き、公開KUC runtimeによるfixture UIを操作できる

#### Scenario: 自動検証が対話起動を置き換えない
- **WHEN** `just storybook-check` を実行する
- **THEN** entrypoint、fixture契約、短時間のnative window smokeを検証する。物理IMEと最終人間UIの受入可否は、この自動smokeとは独立したrelease方針で判断する

### Requirement: 依存更新は品質基準を維持する

KCU SHALL 全manifest/lockを最新化し、既存check、AST lint、coverage基準を変更せず検証する。

#### Scenario: 更新版に非互換性がある
- **WHEN** 最新依存で既存品質ゲートが失敗する
- **THEN** 互換修正を検証するか更新不採用の理由と残存版を明示する
