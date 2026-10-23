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
