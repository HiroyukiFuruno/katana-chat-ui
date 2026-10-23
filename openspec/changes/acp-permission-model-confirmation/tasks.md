# ACP最終review修正台帳

## DoR
- [x] P1のtool_call破棄とP2のconfig_options破棄を実コードで確認した。
- [x] 既存v0.1.0要件・無課金・自動承認禁止・純KUC・全品質gateを維持する。

## 修正・検証
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
