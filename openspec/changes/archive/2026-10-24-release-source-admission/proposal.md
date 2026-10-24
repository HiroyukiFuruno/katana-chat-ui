# リリースソース admission

## Why

手動 `workflow_dispatch` が任意の branch/ref から両OS検査と5 crate公開へ進める余地があり、検査対象のcommitと公開対象のcommitを一意に固定できない。

初回のActions公開では認証後のtoken権限検査が403になった。公開対象5 crateに限定した専用認証へ分離し、既存tag/sourceのActions復旧と独立consumerで解消を確認する。

## What changes

- 通常のmerged release PRでは、同一repositoryの `release/vX.Y.Z` と merge commit SHA をadmissionで検証する。
- 手動復旧では既存 `refs/tags/vX.Y.Z`、checkout HEAD、manifest/input versionの一致を検証する。
- admissionが解決したsource SHAとversionを両OS検査・公開jobへ渡し、admissionまたは両OS検査が失敗した場合は公開しない。
- 実temporary Git repositoryを使い、成功条件と拒否条件を回帰検証する。
- 公開stepは専用Secretだけを参照し、共通token更新と分離する。公開しない認証名移行の診断はadmission・後続jobを抑止する。
- 実Actions公開・全5 registry artifact・独立consumerと、専用Secret名の実診断を別々に受入する。

## Non-goals

- ローカルからcrate公開は行わず、公開は既存GitHub Actions経路に限定する。外部provider接続の追加は行わない。
- Cargo依存、5 crateの公開順序、既存品質基準を変更しない。
