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
- [x] 通常commit/push、Draft review2回・指摘対応・Ready・required CI・mergeを回収する（PR #16、最終2 review、required checks、merge後master CI run 40750676495）。
- [x] 必要な版を既存GitHub Actionsで公開し、公開consumer/app/Storybookの背景受入と公開後cleanupを検証する（全5 crate、4 consumer binary/8 phase、public app/Storybookのroot受入、Linux IME機械契約、release run 40750678393 cleanup success）。#7/#11の表示失敗、Storybookがprovider-freeで実SDK通信を示さないこと、3 providerのresponse/cancel-resume計6 live-provider scenarioが未完了であることは別途維持する。
- [x] #3/#13/#14を実証拠付きでCloseし、上位の全Issue解決台帳を更新する。#7/#11は未解決のためOpenで維持する（2026-10-28 21:20 UTC、close evidence comments 6480961929/6480969083/6480969972）。

## 0.2.2公開後の受入状態（2026-10-29）

上記の公開・issue closeは、このchangeやKCU全要件の完了・archiveを意味しない。公開元tag `e96f38b85b1eeaf516ddf08e5e2ca0a4516b4902` はmerge commit `6254232901b8a8aefee2f7f710bc59c41a9843b4`を指す。最終review/merge、Release Actionsの公開とworkflow cleanup、全5 crateのregistry照合、registry-only consumer 4 binary/8 phase、public app/Storybookの記録済みシナリオ、3 host Linux IMEの機械契約を確認した。証跡は`tmp/verification/all-open-issues-20261029/release022/`、`public-preparation/v022-consumer/`、`acceptance-prep/public-ui-v0.2.2/root-public-ui-accepted.json`、`acceptance-prep/public-ime-v0.2.2/root-public-ime-accepted.json`。

public IME原画は#7 GPUI/Linux/X11 preedit重複と#11 egui/Floem候補窓重なりを示すため、両IssueはOpen・視覚不合格のまま残す。historical単色画像1件も原因未確定であり、独立追跡を続ける。3 providerのresponse/cancel-resume計6 live-provider scenarioは5件pass、Claude通常responseがtimeout・実exit 1で受入一式は未完了。Claude cancel/resumeはpassしているがtimeout原因は未確定。実通信/root rehash証跡は`public-preparation/v022-consumer/provider-acceptance-final-20261029.json`と`root-provider-evidence-rehash.json`。別シナリオpassやStorybook fixtureを全6 scenario成功へ読み替えず、全要件完了とも主張しない。

Codexの別v0.2.2 consumer-copy response/cancel-resume 2 scenarioは、公開sourceに束縛した検証consumerの誤期待をACP仕様に沿って訂正し、受入済み。元wrapper失敗は保持し、Claude通常response timeoutとIME #7/#11は未解決である。詳細とreceiptは[ACP受入記録](../../../docs/acp-compatibility.md)を参照。

旧`hooks/issue-3` worktreeはHEAD `a7adae52460e0a00a498f4b5ba00a5d6b00ccddf`のまま残り、5つの未コミット差分のSHAを`release022/root-scoped-release-hygiene.json`へ保存した。これらは現releaseへ統合・削除しておらず、上記のCloseで整理済みとは扱わない。対応する既存未完了taskは未チェックのまま保持する。active OpenSpec changeはarchiveしない。

## PR16のレビュー指摘

- [/] P1: 複数Issue参照を許可し、変更Cargo scopeを明示した証跡からownerを一意決定する。未対応・複数ownerは拒否する。`test_multi_issue_commit_maps_scopes_to_their_unique_evidence_owners`と両拒否条件を確認した。
- [/] P2: manifest実変更と正確なscope/hashがある場合、package解決結果が同じでも許可する。`test_manifest_only_equivalent_requirement_change_allows_empty_lock_diff`を修正前RED・修正後GREENで確認し、lock-onlyの無意味差分は引き続き拒否した。hook suite17件はroot再検査でも成功した。
