# acp-compatibility-boundary Specification

## Purpose
KCUのACP v1通信をprovider固有の実行環境から分離し、capabilityとpermissionの境界を維持する。
## Requirements
### Requirement: ACP v1 transport を提供する

KCU SHALL `agent-client-protocol` を使う ACP v1 stdio transport を提供し、provider の接続、capability、session、permission、streaming を共通 host へ投影する。`initialize` は接続後の最初の request とし、広告されない capability は利用可能として扱わない。

#### Scenario: ACP agent へ接続する
- **WHEN** Codex、Claude Code adapter、または GitHub Copilot の ACP provider を選択する
- **THEN** KCU は stdio subprocess に対して `initialize`、`session/new`、`session/prompt`、`session/update`、`session/cancel` を ACP v1 wire schema で処理し、agent が送る model config、permission request、text / thinking / tool update、完了または失敗を共通 host へ投影する

#### Scenario: agent が capability を広告しない
- **WHEN** prompt の image または embedded resource を送ろうとする
- **THEN** KCU は agent の prompt capability を確認し、未広告の content を wire request に含めず明示的な error にする

#### Scenario: permission request を受け取る
- **WHEN** ACP agent が `session/request_permission` を送る
- **THEN** KCU はユーザーが選んだ option、cancel、または deny を request ID に対応付けて応答し、絶対パスでない対象は選択可能にしない

### Requirement: 内部contentをwire形式と混同しない

KCU SHALL 内部AcpContentBlockの保存形式を仕様確認だけのために破壊せず、公式wire型へ変換が必要な箇所を明示する。

#### Scenario: 画像参照をACPへ送る設計を行う
- **WHEN** 内部型がdata_ref等の参照だけを保持する
- **THEN** 参照文字列をbase64 dataとして送信可能と扱わず、明示的な変換とcapability交渉を要求する
