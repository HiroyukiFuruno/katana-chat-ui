# 実装・検証

- [x] 最新Open Issue5件、現行master、既存worktree、公開registry、上流PRを読取り確認した。`tmp/verification/all-open-issues-20261029/`。
- [x] Issue13の両現行イメージで正常対照200・拒否対象4件も200となるREDを実測し、root単独除外ではAAA経路が残ることを記録した。
- [x] #3 commit/依存更新gateと品質検査dispatcherを実装し、unitと実Git fixtureで検証する。初回28件成功、root再検査28+16件成功、issue3-final-tests/にsource SHAと実結果。レビュー修正の追加回帰は下記で区別する。
- [x] #3 公開確認・対象限定cleanup、switch/pull、保持条件、remote明示選択を検証しworkflowへ組み込む。unit/実Git fixtureと既存v0.2.1のlive公開guardを確認。新workflowの実公開後実行は後段で回収する。
- [x] #13 Debian公式更新をsource Dockerfileへ反映し、両イメージを再構築する。issue13-source-builds/final-checks/source-build-receipt.jsonに両build exit0・source SHA・image ID。
- [x] #13 同じ標準TLS client/storeで正常対照成功、拒否4件の証明書エラー、信頼ストア在庫を検証する。最終probe 00642c05の両画像でcontrol200/bad4 verify_code20、native CA在庫を確認。実SIGTERM/所有CIDの通常stop・消失も確認。issue13-checker-fixes-v2/とissue13-{sigterm,stop}-real/。
- [x] #14 固定allowlistのpreview/removeと保持方針を実装し、実Dockerで参照中・停止中・無関係保護、拒否・再生成を検証する。検査7件（単体6・実Docker統合1）成功と既存2cacheの安全なremove・通常native入口でcold再生成・対象外volume保持、15画像とStorybook smoke成功。cache-cold-validation/。依存更新後の完全gateは後段で実行する。
- [ ] 既存issue-3 worktree差分の採否と検証を確定し、他者差分を失わず統合・整理する。
- [x] 直接/推移依存と全7lockを最新互換に評価し、既存完全gate/coverage/画像基準を維持する。ACP3.6/schema1.13/derive3.6、AWS-LC1.19/sys0.47とKUC0.6の公式source/checksumを全7lockで照合。更新後just release-check actual0、line coverage86.69%。libc互換pinと上流由来のCargo future-incompat noticeは区別して追跡する。
- [ ] 通常commit/push、Draft review2回・指摘対応・Ready・required CI・mergeを回収する。
- [ ] 必要な版を既存GitHub Actionsで公開し、公開consumer/app/Storybookの背景受入と公開後cleanupを検証する。
- [ ] #3/#13/#14を実証拠付きでCloseし、上位の全Issue解決台帳を更新する。#7/#11は修正未完了ならOpenで維持する。

## PR16のレビュー指摘

- [/] P1: 複数Issue参照を許可し、変更Cargo scopeを明示した証跡からownerを一意決定する。未対応・複数ownerは拒否する。`test_multi_issue_commit_maps_scopes_to_their_unique_evidence_owners`と両拒否条件を確認した。
- [/] P2: manifest実変更と正確なscope/hashがある場合、package解決結果が同じでも許可する。`test_manifest_only_equivalent_requirement_change_allows_empty_lock_diff`を修正前RED・修正後GREENで確認し、lock-onlyの無意味差分は引き続き拒否した。hook suite17件はroot再検査でも成功した。
