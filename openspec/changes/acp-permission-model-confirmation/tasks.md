# ACP最終review修正台帳

## DoR
- [x] P1のtool_call破棄とP2のconfig_options破棄を実コードで確認した。
- [x] 既存v0.1.0要件・無課金・自動承認禁止・純KUC・全品質gateを維持する。

## 修正・検証
- [/] 第13回P1: 標準composerの添付なし明示tmp Markdown編集をModify contextへ接続し、既存本文からDiffCandidateを生成する修正と回帰を追加した。既存添付・Create優先、物理workspace/tmp境界、missing/non-file/invalid UTF-8/parent/symlink/multiple-targetの明示拒否を維持する。旧版RED144成功/追加6失敗101、修正後focused host153/recorder2/AST19・strict fmt/lintと完全release-checkは実exit0。rootが506 source不変・coverage86.37%・native15/headless5の新規画像を照合した（`pr4-review13-markdown-edit-final-root-accepted.json`）。正式固定HEADのtext-only実Create→Apply→Modify→ApplyDiff→Undoと以後のrelease工程は継続する（discussion_r4365498531、0009/0010）。
- 第12回修正の完全release-checkも実child exit0。rootが506 source SHA不変、line coverage86.37%、native15/headless5新規画像を独立確認した（`pr4-review12-katan-model-final-root-accepted.json`）。host143/ACP42/core164/recorder2/AST19・既存全strict gate・背景Storybook smoke・package gateを維持。正式固定HEADの実受入・specific返信/resolve・最終review/CI・公開は親台帳で継続する。
- [/] 第12回P2: KatanAgent再選択・他providerからの復帰で、実executorの確定モデルをUIと保存履歴へ反映する。実行中・取消完了待ちの同一provider再選択も保持し、未確認の復元値を流用せず、fresh/cleared catalogと既存confirmation境界を回帰検証する（discussion_r4365405514）。旧版None対Some("confirmed") assert RED101を保持し、root指定ENV4のfocused gateはhost143/recorder2/AST19・strict fmt/lintが実exit0（`pr4-review12-katan-model-root-focused-attempt1.log`）。worker旧環境の実行は正式gateへ数えない。完全gate・固定HEAD実受入・review/公開は親台帳で継続する。
- 第11回修正は旧版134件成功・追加2assert RED101、修正後host139/recorder2/AST19・strict fmt/lintとOpenSpec8/8が成功。完全release-checkも実exit0、rootが506 source SHA不変・line coverage86.37%・native15/headless5新規画像を照合した（`pr4-review11-output-final-root-accepted.json`）。初回fixture行数lint失敗は関数分離で是正し、無効化は追加していない。正式固定HEAD実受入・specific返信/resolve・最終review/CI・公開は親台帳で継続する。
- [ ] 第11回P2: 保存済みChatOutputをsource_message_idで対応履歴へ関連付け、生成本文・編集後本文、種類と候補状態をescaped dataとしてACP/KatanAgentへ渡す。未適用候補を適用済みと扱わず、順序・孤立output・cursor/再試行を回帰検証する（discussion_r4365304596）。
- [ ] 第9回P2: provider別の会話文脈位置と終端確認を実装し、A→B→A・同一provider再選択・失敗・取消待ち・会話境界を回帰検証する（discussion_r4364914338）。旧版は切替先context欠落で実exit101（`pr4-review9-context-red-attempt4.log`）。初回0件実行とfixture設定/Runtime不足は別失敗として保持し、意味的REDへ数えない。完全gate・正式固定HEAD実受入・最終review/CI・公開以降は親台帳で追跡する。
- [x] PR #4 第4回P2: config要求をsession FIFOへ直列化し、実JSON-RPC遅延応答17件・保留中Cancel・Full拒否・Closed分岐・Guard dropを検証した。ACP37件と最新7lockの完全gateも実exit0（discussion_r4363017812、`pr4-review4-deps-final-root-accepted.json`）。最終同一HEADのcloud review/CIは親台帳で追跡する。
- [x] PR #4 第2回P1: 並行permission requestをFIFOで保持し、応答成功時だけ次を表示する。FIFO・ID選択・表示/待機列消去の回帰と既存lifecycle回帰、host-support94件・strict lintを確認した（`pr4-review2-host-check-attempt2.log`）。完全gateと再reviewは未完了。

第2回修正後の完全gateもexit0、502 source SHA不変、行coverage84.26%、native15画像と既存5画像一致をrootが確認した（`pr4-review2-complete-root-accepted.json`）。再review/CIの完了を引き続き待つ。
- [x] P1: 承認対象の操作情報をtransportからUIへ渡し、失敗する回帰を先に追加する。
- [x] P1: 実KUCでtitle/details・長い内容・button/modal操作を確認する。
- [x] P2: config成功応答の確定modelを投影・保存し、通知を別送しない場合の回帰を追加する（`tmp/verification/acp-config-response-{red,green,host}.log`）。
- [x] P2: model設定拒否のうちInvalidRequest・MethodNotFound・InvalidParamsだけを非致命エラーとして通知し、通常のmodel/readiness/permission/streamを保持する。InternalError等はfail closedを維持する。history復元ではenqueueだけで完了扱いせず、同じconfig_idの応答と要求modelに一致するcurrent_valueを待つ。拒否・不一致は保存済みmodelを維持してBlocked/Missingにし、理由を示す（`tmp/verification/config-rejection-{red,acp-redgreen,host-redgreen}.log`、`tmp/verification/restore-confirmation-focused.log`、`config-rejection-restore-host-tests.log`、`config-rejection-restore-acp-tests.log`、strict Clippy 2件とOpenSpec strict validate成功）。
- [x] 既存の失敗・permission選択肢検証・history/model保持を維持し、focused lint/testを通す（unit/strict lint成功、`acp-history-final-traced-real.log`でreal history/model保持とprompt3件のみを確認）。
- [ ] 公開KUC修正版採用後の完全check/release-checkとDraft PR reviewで再確認する。
- [x] PR #4 P2: 通知・設定応答で更新したconfig IDをUI候補と同じ共有状態へ反映し、削除済みIDの選択は非致命の設定拒否として扱う（discussion_r4361777668。通知/response/削除ID/session分離/通知先closedの回帰、ACP34・既存lint/AST19/unit成功。完全gateとcloud reviewは下記の最新証拠を参照）。

PR #4修正後の完全release-checkはexit0、502 source SHA不変、行coverage84.26%、native15画像一致をrootが確認した（`pr4-final-release-root-accepted.json`）。現行binaryの実Codex履歴probeもexit0、prompt3/new3/config2・capture failure0である（`pr4-final-real-visual-root-accepted.json`）。修正後cloud reviewと最新CIの受入はまだ未完了。

## DoD
全P0/P1解消、P2採否は既存要件のmodel保持に必要なため修正対象。実装・検証・reviewの証跡を追記し、未実施を完了にしない。

## 修正後の証拠
- P1 red/green: `tmp/verification/permission-context-{red,green}.log`。KUC permission7、modal composer wheel1、modal input1、ACP conversion1、host permission12が成功。
- strict all-target Clippy: `tmp/verification/permission-context-clippy.log`、KCU/ACP/host-supportが成功。
- 統合unit: `tmp/verification/acp-final-review-integration.log`でACP20成功、`host-final-review-integration.log`でhost77とrecorder1成功。これらの後にmodal input guardのfocused回帰1が成功した。公開KUC 0.4.4を採用した最終全体gateは未実施。
- 最新history初回はrecorder shimを設定し忘れ、実応答後にtraceが空のためstrict proof検査でexit1（`acp-history-final-real.log`）。製品不具合や成功の証拠とはせず、real npxをrecordするshimと空traceを用意して再検証する。検証条件は緩和しない。

- AST19成功、全OpenSpec strict validate4成功。local環境設定を明示launcherへ接続し、`claude-local-launcher-real.log`で実応答成功。全体release gateとpublic registry版採用は引き続き未完了。
- model拒否修正のfocused ACP/hostテストは完了（拒否code変換、provider projection、ready+permission+stream保持）。strict lintとOpenSpec再検証を続ける。
- model拒否と復元確認修正はACP unit 21件・host-support unit 84件成功、strict all-target Clippy両crate成功、OpenSpec strict validate成功。実履歴probeと公開KUC 0.4.4採用後の完全release gateはrootが再確認する。

- request拒否・履歴model確定順序の修正後、all7 fmtとAST19が成功（`config-rejection-final-{fmt,ast}.log`）。実recorder経由のhistory probeもexit0（`acp-history-confirmed-real.log`）、Codex prompt3/new3/config2、capture failure0をrootが集計した。wrapper終了時の予約済み変数エラーはprobe完了後で、product failureや全体gate成功とは扱わない。最終public KUC採用後の完全gateは未実施。

- [x] 第5回P1: ACP Stopはprompt終端までcancellingを保持し、残留projectionを抑止。次draftをmutation前に拒否し、Finished/terminal failureで解除する。strict host115/AST19と実Codex取消・再開probeはexit0。完全release-checkもexit0、504 source不変・line85.27%・既存20画像一致。固定HEAD最終review/CIは親release台帳で回収する。

- [/] 取消中のsession設定確定と設定拒否を残留turn出力から分離する。旧版は追加2回帰ともassert失敗、修正後host117/recorder2/AST19とstrict lint/fmtは実exit0（`pr4-review5-config-{red,focused-attempt1}.log`）。新sourceの完全gate・固定HEAD最終review/CIは親台帳で回収する。

取消中設定通知修正の完全release-checkも実child exit0。rootが504 source SHA不変、line coverage85.27%、新規native15/headless5画像のSHA・寸法・生成時刻を独立確認した（`pr4-review5-config-final-root-accepted.json`）。host117/recorder2/ACP37/core164、背景Storybook smoke、package gateを維持。次は通常commit/push、固定HEADの5binary実接続/背景実対話、specific返信/resolve、同一HEAD最終review/CI、公開以降を回収する。

- [/] 第6回P1: Stop後のlate permissionへCancelled応答し、次の受理promptでは通常permissionを再開する。同mutex flag/mapでraceを防ぎ、実JSON-RPCの無UI投影・prompt終端・次promptを検証。旧版unit assert RED101済み（discussion_r4364378777）。
- [/] 第6回P2: 設定拒否を報告する前に同poll内の確定modelを保存する。旧版FS2回帰assert RED101済み、lifecycle保存修正準備。strict/完全gate・固定HEAD最終review/CIは未実施（discussion_r4364378786）。

第6回修正のfocused gateは実child exit0（`pr4-review6-focused-attempt3.log`）。ACP41/host119/recorder2/AST19と既存strict fmt/lintが成功。late permission Cancelled/noUI/次Selectedの実SDK契約、実start_promptでbusy拒否時取消保持・受理時reset、先行確定model保存と保存失敗明示のFS回帰を含む。初回の新test型エラーとhelper行数超過は局所修正し、失敗ログも保持した。完全gate・正式保存・固定HEAD実受入・同一HEAD最終review/CI・公開以降は未完了。

第6回修正の完全release-checkも実child exit0。rootが504 source SHA不変、line coverage86.14%、新規native15/headless5画像のSHA・寸法・生成時刻を独立確認（`pr4-review6-final-root-accepted.json`）。ACP41/core164/host119/recorder2・既存全strict gate・背景Storybook smoke・package gateは成功。以降は通常commit/push、固定HEADの5binary実接続/背景実対話、2thread返信/resolve、同一HEAD最終review/CI、公開・consumer・archive・hygieneを回収する。

- [/] 実wireでPromptよりCancelが先行するraceを修正する。SDK同期publicationをspawn前に行い、prompt終端待ちはbackground taskに保つ。旧版ordering REDと最新strict/全gate/実trace順/最終reviewを確認する（`pr4-cancel-before-prompt-root-evidence.json`）。

送信順修正後の完全release-checkは実child exit0。rootが504 source SHA不変、line coverage86.37%、native15/headless5画像のSHA・寸法・生成時刻を独立確認した（`pr4-review6-order-retry1-final-root-accepted.json`）。ACP42/core164/host119/recorder2と既存strict gate・背景Storybook smoke・package gateを維持。正式固定HEADの実trace順序・背景実対話・最終review/CIと公開以降は未完了。

送信順修正の旧版はraw SDK ChannelでCancel→Promptのassert失敗101を回収（`pr4-cancel-order-red-attempt2.log`）。既存実Codex traceの順序検査もRED1（`pr4-cancel-trace-order-red.log`）。send_requestをspawn前に同期登録しSentRequestをtaskへ渡す修正後、ACP42/host119/recorder2/AST19とstrict fmt/lintは実child exit0（`pr4-review6-order-focused-attempt1.log`）。raw Cancelはprompt terminalを返さない間に到着するため、取消を終端までブロックしないことも確認した。新しい固定HEADの実traceでもPrompt→Cancel→次Promptを件数と別に確認する受入へ強化する。完全gate・正式保存・最終実受入/review/CI・公開以降は未完了。

review9修正後のfocused131/2/19と完全release-checkはactual0。rootが505 source不変・coverage86.37%・native15/headless5の新規画像を確認した。依存再照合は公開KUC0.4.4/ACP3.5.0/smallvec1.16.4を維持。正式保存・固定HEAD実dual/既存probe・thread返信/resolve・最終review/CI・公開以降は未完了。ローカルOllama実dualの失敗2試行は保持し、思考無効設定のattempt3を継続中。

- [ ] 第10回P2（4365138935）の履歴Text/File添付欠落をACP/KatanAgentで修正し、境界escape・添付だけの会話・cursor/失敗/復元との組合せ回帰、strict/完全gate、正式固定HEADの添付実dual・既存実受入・reply/resolve・最終review/公開以降を確認する。旧版は従前131件成功と追加3件assert失敗101（pr4-review10-attachment-red-attempt1.log）。

第10回修正は旧版131pass/追加3assert RED101、修正後focused134/2/19と完全release-check actual0。rootが505 source不変・coverage86.37%・native15/headless5の新規画像を確認（pr4-review10-attachment-final-root-accepted.json）。公開API・保存形式・依存・画像policy・cursor/取消の実装は不変。正式固定HEADで添付値を初回assistantへ露出させない実ACP/独立Katan履歴recall、既存実probe/背景画面、thread返信/resolve・最終review/CI・公開以降は未完了。
