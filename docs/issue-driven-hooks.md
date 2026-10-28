# Issue起点hookとrelease cleanup

この文書は、KCUのcommit/pushにIssue参照と依存更新証跡を結び、公開後cleanupを明示されたbranchだけに制限するhook契約を定義する。Git hook設定とCI/workflowからの呼出し設定は別途管理する。これらのPython entry pointを追加しただけではhookが設置・有効化されたことにならない。

## commit-msg

`commit-msg` hookは非default branchで作る各commit messageに対象repositoryの正のIssue番号を要求する。`release/*` も対象となる。default branchのみ参照を省略できる。bare `#123` や `Issue #123`、`Issue-123` などを認識し、`0` は無効。他repositoryを示す `owner/repo#123` や、URL内のfragment `.../issues/123#45` だけでは対象repositoryのIssue参照を満たさない。

入口:

```sh
python3 scripts/hooks/issue_gate.py commit-msg "$1"
```

default branchは `origin/HEAD` から読み、存在しない場合だけ `origin/master` または `origin/main` の一方を使う。両方存在して選択が曖昧ならhookは失敗する。

## pre-push dispatcher

入口は `python3 scripts/hooks/dispatch.py`。Git pre-pushがstdinで渡すref update行を読み、順番に:

1. `just release-verify` を実プロセスで実行する。Justfileや品質コマンドを置換できる`JUSTFILE`、`CARGO`、`RUSTFLAGS`などは子プロセスへ渡さず、`RTK`は通常の既定解決に任せる。
2. 失敗exitをそのまま返し、後続のIssue/dependency gateを実行しない。
3. `COVERAGE_MIN_LINES` はJustfile既定値以上の整数なら維持する。既定値未満または不正値なら既定値でgateを実行し、成功後に指定値を拒否する。
4. 成功した場合、gate前後でHEADが同一であること、gate後のsourceがcleanであること、検査中にsourceが変化しないことを確認する。
5. pre-push stdinが空なら拒否する。同じHEADを指すbranch pushについて、stdinのold/new SHAから新commit範囲だけを得て、default以外のcommit messageにIssue参照があることを確認する。
6. dependency manifest/lockfileの変更は、同じ範囲で変更された `issues/<番号>/dependency-update-evidence.json` とCargo.lock実差分に照合する。

既存hookのdispatcher順序は呼出し側で管理する。既存のrepository固有release gateを先に実行し、その失敗を成功へ変換したりIssue gateを続行したりしてはならない。

### dependency証跡JSON

現在の検証対象はCargo manifest/lockfile scopeである。manifestだけの変更、lockfileだけの変更のどちらも対象となり、対応するancestor `Cargo.lock` と `Cargo.toml` が同じscopeに必要。未対応形式のmanifest/lockfileはpassさせず失敗する。

`upstream_release.url` はHTTP(S)公開URL、`version` は公開版、`api_migration_note` は実際の変更説明（API変更なしならその明示）、`scopes` は変更された各Cargo scopeを列挙する。`manifest_sha256` と `lock_sha256` はpush対象commitのblob内容から照合される。`changed_files` はそのscopeで変更されたmanifest/lockfileだけの一覧。`direct_changes` と `transitive_changes` はCargo.lockの前後を比較して求めたname/source/from/to一覧と一致しなければならない。sourceを持たないlocal packageはsource値 `local` と実versionで差分に含める。packageのname/version/source/checksumに不正値がある場合は検査を失敗させる。lockにchecksumがあるcrateはfrom/toにversionとchecksumを含める。

1 commitが複数Issueを参照しても、また同じCargo pathを変更するpush範囲内のcommitが別々のIssueを参照しても検査できる。変更範囲の少なくとも1つのpath変更commitと、変更された `issues/<番号>/dependency-update-evidence.json` のcommitが同じIssueを参照し、そのevidence scopeが対応するmanifest/lockfileを明示する場合に、そのIssueをscope owner候補とする。候補がない場合、または複数Issueの証跡が同じscopeを主張する場合は失敗する。Cargo.lockのpackage差分が空でも、manifest自体がbaseから実際に変わり、その変更ファイル・manifest/lock hash・scope証跡が一致すれば通過できる（例: version requirement表記やfeature指定の変更）。manifestに実変更がなく、Cargo.lockにもpackage差分がないlock-only変更は失敗する。

各scopeの形式:

```json
{
  "manifest": "Cargo.toml",
  "manifest_sha256": "<push対象Cargo.tomlのSHA-256>",
  "lockfile": "Cargo.lock",
  "lock_sha256": "<push対象Cargo.lockのSHA-256>",
  "changed_files": ["Cargo.lock", "Cargo.toml"],
  "direct_changes": [{"name": "example", "source": "registry+https://github.com/rust-lang/crates.io-index", "from": "1.0.0", "to": "2.0.0"}],
  "transitive_changes": []
}
```

証跡の `verification.commands` には実施済み検査を明記し、`verification.strict_quality_gate` は `just release-verify` とする。JSON内の宣言だけでgate成功にはならない。dispatcherがpush HEADで実行したrelease-verifyの実exit、同じHEAD、clean sourceを確認する。

Python 3.11以降が必要（Cargo manifest/lock解析に標準の`tomllib`を使う）。限定自動検証入口:

```sh
python3 -m unittest scripts.hooks.test_issue_gate
```

## 公開後のbranch cleanup

`scripts/release/release_cleanup.py` はGitHub Releaseが存在し、draftでないことを `gh release view` で確認してからGit操作へ進む。`--release-tag` と、`--branch` または `--sync-only` のどちらかを明示する。最初に現在のcheckoutがcleanでuntracked fileもないことを確認し、`origin` をfetch、default branchへswitchして `git pull --ff-only` を実行する。CLIにguard回避経路はなく、Git engineのfixture検証とRelease guardの関数境界テストを分ける。

`--sync-only` はRelease guard、clean確認、fetch、default branchへのswitch、ff-only pull、worktree pruneを同じように実施し、branch評価・削除は行わない。`--branch` や `--delete-remote` とは併用できない。merged release branchのPR後処理ではworkflowから実際のrelease branchを `--branch` で明示する。workflow_dispatchでdefault branchを再同期する場合は `--sync-only` を使う。

Local targetはdefault/current/どのworktreeでも使用中/未mergedなら保持する。currentがdefaultであることとHEADへのancestorを事前確認し、保存済みupstreamに依存しないよう予約remote名 `__kcu_cleanup_no_upstream__` をコマンド限定設定して通常の `git branch -d` で最終判定する。このremote名が既に定義されていれば停止する。Git自身の-d判定でもdefaultに未mergedのbranchは保持され、repository config fileは変更しない。Remote deletionは別途 `--delete-remote` を指定した場合に限る。その場合も使用中worktree、default、live tipとfetch済みremote-tracking tipの不一致、未mergedを保持する。取得した期待tipを `--force-with-lease=<ref>:<oldsha>` に渡すcompare-and-swap削除を行い、取得後にremoteが進んだ場合は削除を拒否する。未指定branchは列挙・削除しない。

Git操作の回帰テストは一時bare repositoryを使い、`git worktree add` や偽の `gh` 実行ファイルを使わない。worktree使用判定は通常のfixtureの `git worktree list --porcelain` を直接読み取って検証する。GitHub Release照会は関数境界の単体テストで扱い、実公開確認は実Release上で別途行う。

実行例:

```sh
python3 scripts/release/release_cleanup.py --release-tag v0.2.1 --branch release/v0.2.0
```

branchを削除せず、公開済みReleaseを確認してdefault branchだけを同期する:

```sh
python3 scripts/release/release_cleanup.py --release-tag v0.2.1 --sync-only
```

Remote branchも削除する判断は、実行者がtargetを確認して明示的に選んだ場合にだけ追加する:

```sh
python3 scripts/release/release_cleanup.py --release-tag v0.2.1 --branch release/v0.2.0 --delete-remote
```

cleanupで指定targetが安全条件に合わなければ非zeroで報告し、targetを保持する。working tree dirty、Release未公開/draft、fetch/switch/pull失敗ではtarget処理へ進まない。release tagを指定することは公開確認の代用ではない。

限定自動検証入口:

```sh
python3 -m unittest scripts.release.test_release_cleanup
```

このtest suiteはtemp内のbare/local Git repositoryを使い、GitHub Release guardだけを関数境界で単体検証する。単体テストは実GitHub公開証跡ではない。実公開後Actionsからの実Release確認は別の運用証跡として保持する。
