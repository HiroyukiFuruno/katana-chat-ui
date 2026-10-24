# リリースソース admission

## Why

手動 `workflow_dispatch` が任意の branch/ref から両OS検査と5 crate公開へ進める余地があり、検査対象のcommitと公開対象のcommitを一意に固定できない。

## What changes

- 通常のmerged release PRでは、同一repositoryの `release/vX.Y.Z` と merge commit SHA をadmissionで検証する。
- 手動復旧では既存 `refs/tags/vX.Y.Z`、checkout HEAD、manifest/input versionの一致を検証する。
- admissionが解決したsource SHAとversionを両OS検査・公開jobへ渡し、admissionまたは両OS検査が失敗した場合は公開しない。
- 実temporary Git repositoryを使い、成功条件と拒否条件を回帰検証する。

## Non-goals

- crates.io、GitHub Release、外部providerへの接続や公開を行わない。
- Cargo依存、5 crateの公開順序、既存品質基準を変更しない。
