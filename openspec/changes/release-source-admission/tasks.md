# Release source admission

## Tasks

- [x] 通常のマージ済み同一 repository `release/vX.Y.Z` と merge commit SHA を admission で検証する。
- [x] 手動復旧を既存 `refs/tags/vX.Y.Z` に限定し、tag、checkout HEAD、manifest、入力版の一致を検証する。
- [x] admission の SHA/version 出力を macOS・Linux/aarch64 検査と公開 jobへ渡し、失敗時に検査・公開を開始しない。
- [x] 一時 Git repository で成功、任意 branch、missing tag、tag commit 不一致、version 不一致、fork、wrong merge SHA を回帰検証する。
- [x] fixture初期化前に全`GIT_*`をclearし、fixture own-rootを確認する。foreign Git routing（`GIT_CONFIG_COUNT/KEY_0/VALUE_0`を含む）を使う恒久Python回帰で現行Bashのexit 0とforeign HEAD/index/config/status不変、永続fixture証跡を検証する。

## Verification

実施済み証跡:

- `bash scripts/release/test-verify-release-source.sh`: exit 0。`tmp/verification/release-source-admission-regression.log` にversion/source SHA出力とPASSを記録。
- `just release-source-check`: exit 0。`tmp/verification/release-source-admission-just.log` に記録。
- `bash -n scripts/release/verify-release-source.sh scripts/release/test-verify-release-source.sh`: exit 0。
- workflow構造検査、`git diff --check`: exit 0。
- `scripts/openspec validate --all --strict`: 8 passed, 0 failed。`tmp/verification/openspec-release-source-admission-strict.log` に記録。
- `python3 scripts/release/test-release-source-fixture-routing.py`: exit 0。foreign Git routing下でもfixture回帰とforeign HEAD/index/config/status不変を確認。
- legacy版のnonzeroとforeign config変更はtracked testの履歴依存なしにignored sandboxへ分離し、現行版greenは `tmp/verification/release-source-fixture-routing-s6bax92e/routing-evidence.json` に保存。
- `shellcheck scripts/release/test-verify-release-source.sh`、`python3 -m py_compile scripts/release/test-release-source-fixture-routing.py`: exit 0。

rootレビュー反映:

- admissionの初期job条件はmerged・同一repository・`release/v` PRまたはmanual dispatchに限定し、未merged/fork/non-release PRをskipする。
- regression fixtureは`rm -rf`、`git checkout`、`git tag -f`を使わず、repo内ignored temporary directoryを保持する。
- 旧回帰ログの `fatal: no tag message?` は失敗証跡であり、成功証跡として扱わない。修正版のexit 0ログのみをDoD証跡とする。

Cargo、公開registry、外部workflow実行はこの変更の検証対象にしない。
