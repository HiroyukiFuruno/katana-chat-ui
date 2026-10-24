## ADDED Requirements

### Requirement: リリースjobは一つの不変ソースをadmissionする

Release workflow SHALL platform verification jobまたはpublicationを実行する前に、checkout commitとrelease versionを一つだけadmissionする。

#### Scenario: merged release pull requestを検証する

- **WHEN** same-repository pull requestが `release/vX.Y.Z` からmergedされる
- **THEN** admissionはcheckout HEADがpull request merge commit SHAと一致することを確認し、manifest versionを一度解決する
- **AND** both platform gates and publication SHALL successful admission and its source SHA/version outputsへ依存する

#### Scenario: manual recoveryをdispatchする

- **WHEN** `workflow_dispatch` がpartial-publication recoveryに使われる
- **THEN** admissionは `refs/tags/vX.Y.Z`、matching input or manifest version、tag commitとcheckout HEADの一致を要求する
- **AND** arbitrary branches、missing tags、mismatched tag commits、version mismatchesはverificationまたはpublicationの前に失敗する

#### Scenario: fixtureのGit routingを隔離する

- **WHEN** release-source admission regressionが継承した`GIT_*` routing/config環境下でfixtureを初期化する
- **THEN** 検査は全`GIT_*`を破棄し、init直後にfixture自身のGit rootを確認してからconfig/add/commitを実行する
- **AND** legacy版のforeign config変更をredとして記録し、修正版はforeign HEAD/index/config/statusを不変のまま成功する

### Requirement: 公開認証はKCU専用Secretに限定する

GitHub Actions の crate 公開 step は、全5公開対象に許可された `KCU_CARGO_REGISTRY_TOKEN` を唯一の認証情報として使用しなければならない（SHALL）。共通 Secret への fallback を設けてはならない。

#### Scenario: 共通tokenの更新経路と公開経路を分離する

- **WHEN** 共通token更新が `CARGO_REGISTRY_TOKEN` を置き換える
- **THEN** 通常公開 step は引き続き `KCU_CARGO_REGISTRY_TOKEN` だけを参照する
- **AND** 専用Secretが欠落していれば既存の非空検査で公開を拒否する

#### Scenario: Secret名の移行を公開せず検証する

- **WHEN** `workflow_dispatch` で `verify_registry_auth=true` を指定する
- **THEN** release admissionと後続の検証・公開jobを開始しない
- **AND** 読み取り専用jobで新旧Secret名の非空と値の一致を確認する
- **AND** tokenの値・長さ・ハッシュをログへ出さない
- **AND** この診断を公開成功やregistry consumerの代替にしない
