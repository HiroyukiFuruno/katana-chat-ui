# 実装・検証タスク

## 最新状態（2026-10-28）: 承認された0.2.1公開・公開版受入を完了

KCU 0.2.1全5 crateは公開済み。公開元tag/merge sourceは`0c77dde206c32fbb4e34fd21a4459ed3c517c1ba`。独立consumerの4 binary build・全8 phaseは実exit 0、sealed source/lock/binaryを確認済み（`tmp/verification/kcu021-first-release-20261028/public-runtime/build-result.json`、`tmp/verification/kcu021-first-release-20261028/public-runtime/graph-audit.json`、`tmp/verification/kcu021-first-release-20261028/public-runtime/root-stage-accepted.json`）。公開版UI/StorybookはDocker/driver実exit 0、原画/baselineを照合済み（`tmp/verification/kcu021-first-release-20261028/public-runtime/public-ui/root-public-ui-accepted.json`）。公開版Linux IMEは3 hostのcommit/cancel/no-submit契約を満たし、#7/#11だけ表示不合格を維持する（`tmp/verification/kcu021-first-release-20261028/public-runtime/public-ime/root-public-ime-accepted.json`）。#7/#11の修正・上流公開とhistorical black frameの原因確定は未解決のまま独立追跡し、0.2.1完了には含めない。Release用branch整理は完了した。このdocs-only branchはPR作業に必要なため保持し、PR運用・merge後cleanupは製品受入checkboxではなくrootの物理作業台帳で追跡する。過去の履歴は消さず、この最新状態を優先する。

- [x] 1.1 公開KUC frameまたはadapter APIを変更せず、production adapterの公式Floem Vello featureをdefaultにした（`e5af641`、adapter37件・Floem54件・E2E9件、全7 scopeの依存込みlocked metadata成功）。
- [x] 1.2 明示的TinySkia fallbackを維持し、Native22の別条件で実測した（`macos-run22/artifacts/floem-software/`）。既定Velloと混同せず、feature・binary SHAと公式環境変数の記録に限定する。
- [x] 1.3 source `95cdd81`のNative22で3 vendorの全5phaseをfresh captureし、callback時刻と2点canonical RGBA一致を照合、agentが原PNGを目視した（`macos-run22/native22-root-accepted.json`、`native22-diagnosis-receipt.json`）。標準egui比較器の失敗とworkflow全体failureは保持し、全要件達成とは扱わない。
- [x] 1.4 Native25 source `a7ed8ec` のLinux Floem実rasterで、nihongo変換・確定とtesuto未確定・取消の原画、private display候補画像、生state/traceをagentが照合した。`native25-remote/linux25-root-accepted.json`。control labelで代用せず、確定後/取消後の原PNG完全一致も確認した。
- [x] 1.5 公開Vgerの6 frame後`None`観測を再現し、packing contractとは主張せず診断証拠として記録した（`tmp/verification/floem-vger-atlas-diagnostic/src/main.rs`、`tmp/verification/floem-vger-atlas-diagnostic/run.log`）。
- [x] 1.6 renderer設定で公開consumer、公開版app、Storybookの確認を再実行した（`public-runtime/build-result.json`、`public-runtime/public-ui/root-public-ui-accepted.json`）。
- [x] 1.7 GPUI依存の公開待ちは[Issue #7](https://github.com/HiroyukiFuruno/katana-chat-ui/issues/7)と上位作業台帳の`upstream_publication`で別に追跡した。2026-10-28の最新承認により、公開修正版の採用とGPUI重複解消はKCU 0.2.1の停止条件から分離する。Linux実IMEでの同事象の測定・不合格記録は維持し、Issue #7をOpenで後続版へ残す。
- [x] 1.8 `scripts/openspec validate floem-native-raster-backend --strict`を実行しexit 0を確認した。残りの項目は物理証拠が揃うまでpendingとする。

- [x] 1.9 PR #8 discussion4369221140 のLinux取消検証不足を解消した。人間の今回限りの4ファイル診断例外に基づくproducerを含むNative25 source `a7ed8ec` で、全3ホストのEscape前actual marked=true、後のfresh同一Context Hideとactual marked=falseを観測した。GPUIはKUC frame/native marked rangeの両方を要求し、source別sequence/時刻境界と取消後画像まで独立照合した。parser22件の古い・別Context・false後true・欠落の拒否も維持する。`native25-remote/linux25-root-accepted.json`。GPUI表示重複のIssue #7はこの取消契約から分離して未完了のまま保持する。

## ユーザー是正（2026-10-25）

- [ ] 上流Issueへのhandoffだけで停止せず、正しい依存source repoへ修正PRを出し、回帰結果・必要version・公開権限・Actions/maintainerへの公開手順を具体化する。検索語: upstream dependency PR, publication handoff。
- [ ] 画像取得の単発30秒黒frameを再試験成功だけで解決扱いしない。描画/ConfigureNotify/readbackの識別証拠、原因に対する修正と回帰を得る。検索語: first-frame root cause, black capture, no retry-only closure。
- [/] KCU 0.2.1をActionsで公開し、公開consumer・app・Storybookをagentが背景検証した（`public-runtime/build-result.json`、`public-runtime/public-ui/root-public-ui-accepted.json`）。Storybookはprovider-freeのUI契約で、実SDK通信の主張はしない。corrected GPUI/XIM依存を採用する将来版作業は未解決のまま残す。
- [ ] corrected GPUI/XIM互換修正版を選択した将来のKCUを公開する。これはIssue #7の後続版作業であり、承認された0.2.1停止条件ではない。

## 継続検証の実績（2026-10-26）

- [x] 寄稿forkのIssueを先行作成し、上流zed-ximの依存修正Draft PRを作成: https://github.com/zed-industries/xim-rs/pull/3 。既存reset APIを保持し、公開xim-ctext 0.4.1を使用する。
- [x] Rust 1.88で全機能検査を行い、上流候補のActionsで通常CIと公開依存だけの全機能package dry-runを成功させた: https://github.com/HiroyukiFuruno/xim-rs/actions/runs/40208335886 / https://github.com/HiroyukiFuruno/xim-rs/actions/runs/40208335899 。実upload/checksum検証は未実行であり公開済みとは扱わない。
- [x] cloud reviewのservice errorを3回回収し、失敗を合格扱いせず、自己・独立レビューの指摘/修正をPRへ記録した。当時Draftを維持し、その後human指摘による再監査で取り下げた。
- [x] Linux撮影失敗時に子停止前の状態を保存する診断を追加。元30秒/350ms/非単色・SHA条件を維持し、実X11故障注入と正規Cargo matrixで確認した。過去black failureの原因修正とは区別する。
- [x] guardテストのfixtureがDocker VMの共有ファイル保持と競合する原因を実lsof/プロセス実体で特定し、OS標準の一時領域へ隔離した。商用cleanup guardの安全条件は変更していない。
- [ ] 元の単発black failureの因果を確定し、対応する修正・回帰検証を完了する。診断追加、正常再試験、故障注入だけで原因確定済みと扱わない。
- [x] KCU 0.2.1のActions公開後、公開consumer/app/Storybook受入を完了した（`public-runtime/`のbuild/UI root receipts）。
- [ ] GPUI consumer修正PRと上流実公開後に、それを選択する将来のKCUを公開する。Issue #7の後続版作業として残す。

## 外部PRへの慎重さに関するhuman指摘（2026-10-26）

- [/] 外部ライブラリへの提出を急がない。既存Draft https://github.com/zed-industries/xim-rs/pull/3 は説明を訂正し、2026-10-26に未mergeのまま取り下げた。Zed本体への追加PRは未提出。
- [x] 現行Zed mainのgit依存は既にdecoder修正済みであることを一次ソースで再確認した。公開GPUI 0.2.2のregistry依存経路とは区別する。
- [ ] Issue #7のGPUI preedit重複は公開版でも既知のREDとして測定した。corrected dependency後のGREEN再受入は後続版で実入力・原画・callback/状態遷移を合わせて行い、decoder単体の欠陥をZed現行アプリの障害証明へ拡大しない。
- [ ] 既存Issue/PR/公開済み修正との重複、実公開に影響する修正先、MSRV/API変更の必要性をレビューし、不要なmain consumer変更・既存CI修理・公開workflow等を同梱しない最小提案へ絞る。
- [ ] 外部への再提出は上の証拠と独立レビューが揃ってから行う。原因未確定のblack画像問題も、正常再試行で解決済みと扱わない。

## 2026-10-26 現在の継続状態

2026-10-28の最新承認「既知の2種類だけ後続版へ回し、KCUを先行公開する」を優先する。KCU #7 GPUI/Linux/X11のpreedit重複とKCU #11 egui/Floem候補窓重なりは、解決済み・合格とはせず、0.2.1の既知制限として実測・記録したうえでIssueをOpenのまま後続版へ残す。両事象の修正済み上流公開・解消だけを停止条件から分離する。他の入力/確定/取消/誤送信、Mac、純KUC/multivendor/ACP、必須gate/coverage/画像基準、同一最終source/lock/binaryのLinux実IME測定、Draft review・指摘対応・P0/P1・CI、全5crate公開、同公開版app/Storybook背景受入、整理は維持する。historical単色画像1件の従前承認も維持する。

- [x] 公開ctext0.4.1候補にASCII(B/J)欠落があることを独立に再現し、候補を撤回した。公式decoder修正[PR55](https://github.com/Riey/xim-rs/pull/55)はDraftから同一base/headのfork CI全3件・review指摘なしを経てReady。canonical Actionsの寄稿承認と実公開はまだ待機中であり、上の旧PR3や単症例GREENを公開済みの証拠にしない。
- [x] 上流と独立したKUC0.5移行を[別change](../kuc-0-5-compatibility/tasks.md)で開始した。manifest/lockと公開版番号の準備は済み、完全gateは実行中である。
- [x] 1.6の現source app/Storybook受入と、新公開版public consumer/app/Storybook受入を別々の証拠として記録した（`public-runtime/public-ui/root-public-ui-accepted.json`、`public-runtime/build-result.json`）。
- [x] KCU 0.2.1の同一公開source/lock/sealed binaryでLinux実IMEを測定し、Issue #7の不合格を既知制限として記録した（`public-runtime/public-ime/root-public-ime-accepted.json`）。
- [ ] KCU #7を後続版で追跡し、Linux GPUIのcorrected registry依存取得と重複解消を行う。
- [ ] 元black failureの因果未確定と既存DoDの扱いを解決する。診断追加、正常再試行、候補WM機序だけで原因確定とはしない。
