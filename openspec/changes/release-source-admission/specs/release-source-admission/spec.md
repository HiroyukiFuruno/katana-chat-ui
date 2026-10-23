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
