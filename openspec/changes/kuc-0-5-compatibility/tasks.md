# KUC 0.5互換性の実装・受入

## 最新状態（2026-10-28）: 承認された0.2.1公開・公開版受入を完了

KCU 0.2.1全5 crateは公開済み。実公開元tag/merge sourceは`0c77dde206c32fbb4e34fd21a4459ed3c517c1ba`。全crate API/checksum/VCS、204 Rust filesと5 manifestsの一致は`tmp/verification/kcu021-first-release-20261028/public-preparation/registry-public021-attempt2/root-publication-accepted.json`に記録した。公開consumerはsource/lock/binaryを固定して4 binaryと8 phaseすべて実exit 0、registry-only graphとKUC0.6.0/overrideなしも確認済み（`public-runtime/build-result.json`、`graph-audit.json`、`root-stage-accepted.json`）。公開UI/StorybookもDocker/driver実exit 0、captureとbaseline一致・Root原画確認を完了（`public-runtime/public-ui/root-public-ui-accepted.json`）。Storybookはprovider-freeで固定replyを実SDK通信とみなさず、未設定appのselectorからprovider menuを主張しない。

公開版Linux IMEは3 hostすべてで確定1回・送信0・取消後marked解除・`日本語`保持を満たし、36 PNGを照合した。#7 GPUI preedit重複と#11 egui/Floem候補窓重なりだけは表示不合格を維持し、Openで後続版へ残す（`public-runtime/public-ime/root-public-ime-accepted.json`）。#7/#11の修正・上流公開およびhistorical black frame原因は未解決であり、承認された0.2.1完了範囲には含めない。Release用branchの整理は済んだ。このdocs-only branchはPR作業に必要なため保持し、PR運用・merge後cleanupは製品受入checkboxではなくrootの物理作業台帳で追跡する。過去の記録を消さず、この状態を最新として読む。

## 上流と独立した作業

- [x] 公開KUC0.5.0のchecksum/MSRV/公式tagと、0.4.4からの対象API・文字測定差分を確認する。
- [x] KUC/ACPのmanifestと公開5crateのversion条件、全7lockを更新し、locked metadataを確認する。
- [x] 第8回完全gateでcore170/Unicode・geometry/3adapter/host/AST/lint/coverage86.67%/既存headless5・native15/Storybook smoke/packageを実exit0で回収した。source311件は前後SHA一致。
- [x] 現sourceのStorybook原画・操作契約・source/bin対応と最終candidate Native appの視覚受入を記録した。公開版app/Storybook/IMEの再受入は本change末尾の公開完了記録を参照する。#7/#11の表示は失敗のまま保持する。
- [x] 既存無償/ローカル環境で4provider通常応答とCodex履歴/取消、Claude Gemma permission拒否/取消、Katanモデル選択を確認し、未実施vendor/model操作と分離して記録した。
- [x] 仕様/証跡の古い完了表示を照合し、source/公開版/単体/実通信/実IME/目視を区別した。公開版受入とhistoricalblackは未完了を維持する。

## 公開版の最終受入

- [x] 公開consumerの最終依存解決が公開registry由来で、未公開overrideを含まないことを確認した。`public-runtime/graph-audit.json`と`build-result.json`を参照。corrected decoder/zed-xim/GPUIの実公開・採用は未完了で、KCU #7の後続版作業として追跡し0.2.1条件に含めない。
- [x] 公開版と同じsource/lock/binary sealでLinux実IMEを再受入し、入力・確定・取消・誤送信なしを確認し、表示結果も記録した。#7のpreedit重複と#11の候補窓重なりは失敗のままOpenで後続版へ残す。両事象の解消や修正済み上流公開は0.2.1条件に含めない（`public-runtime/public-ime/root-public-ime-accepted.json`）。
- [x] 最終HEADのDraft review→指摘対応/reply/resolve→P0/P1ゼロ→Ready→required CI→mergeを完了する。公開前candidate `2fa74aec9546bf02520098cbf58fc2afaffe0d51`は3回cloud reviewで指摘なし、CI 9 jobs/79 steps成功、merge source `0c77dde206c32fbb4e34fd21a4459ed3c517c1ba`とtree一致（`tmp/verification/kcu021-first-release-20261028/reviews/`、`ci/ci-terminal-proof.json`、`merge-receipt.json`）。
- [x] GitHub ActionsでKCU 0.2.1全5 crateを公開し、registry/Release/独立consumerを確認した（`tmp/verification/kcu021-first-release-20261028/public-preparation/registry-public021-attempt2/root-publication-accepted.json`、`tmp/verification/kcu021-first-release-20261028/public-runtime/build-result.json`）。
- [x] 同じ公開版のappとStorybookを背景目視し、機械契約・原画・source/package/bin一致を回収した（`public-runtime/public-ui/root-public-ui-accepted.json`）。provider-free Storybookの固定replyは実SDK通信の証拠としない。
- [x] Release用の不要なlocal branch 3本を通常削除し、stash 0、新規worktree 0、remote ref無変更、他作業worktreeの5つのdirtyファイルを保持した（`tmp/verification/kcu021-first-release-20261028/hygiene-release-stage-accepted.json`）。

## ユーザーフィードバック

- [/] 上流待ち以外の作業を止めない。KUC0.5更新と現source/実通信の検証を独立して進める。
- [/] 未検証を完了扱いせず、agentが公開版app/Storybookをbackground確認した。KCU #7/#11は既知制限として不合格を保持し、IssueをOpenで後続版へ残す（`public-runtime/public-ui/root-public-ui-accepted.json`、`public-runtime/public-ime/root-public-ime-accepted.json`）。
- [/] 元のhistoricalblack 1件は、2026-10-27の明示承認「未解決で追跡し、他の必須受入後に公開」に従い、原因未確定のまま独立追跡へ分離した。正常再試行を原因特定・解決にはしない。

## 現sourceの限定受入（公開版へ流用しない）

- [x] Storybookのprivate display原画9枚とdraft/submit/scroll/history/provider操作をRootが確認した。3host appの全体検査は進行中。
- [x] 4providerの通常応答、Codex history/cancel host契約を実通信で確認し、Rootがsourceとtraceを照合した。
- [x] Claude/local Gemmaのpermission拒否・取消、KatanAgentのcatalog-backedモデル再選択・復帰、Codex生成開始後のSDK取消理由を実probeで確認し、未確認のvendor/model範囲を記録した。
- [x] Claude ACP0.96へ更新したsource/bin/traceで、local Gemma通常応答・permission reject/Cancelled decisionを再検証した。Qwen通常応答は90秒内未完了として保持し、全vendor/model成功へ拡大しない。
- [x] OpenSpec1.15・artifact v7更新後の第9回完全gateが実exit0、source312件SHA一致、coverage86.67%維持。d7のCI/review完了とは別に、新HEADのreview/CIは再取得する。
- [x] Floem softwareの古いpreedit表示を714反復測定で再現し、native callback後・新KUC frame前に撮影を始める検証側の同期不足を確定した。frame検査が早い画像を拒否し、後続の日本語画像を取得した。商用描画の不具合やhistorical GPUI単色画像の原因へ拡大しない。
- [x] 検証側が同じ古い画像2枚を安定と判定できる不足を純testで確認し、Floemのnative callback後・撮影前のruntime frameも要求する検査を追加した。商用処理・既存deadlineは不変。frame stampは表示完了や文字内容の証明とは扱わず、原画確認も維持する。
- [x] 同条件のsoftware追加5回を714と26e専用CIで回収し、拒否したframe前画像も保持した。Rootが714の30原画・4種類の画像と時系列を照合し、26eの全phase native/frame契約も再計算した。f43の正常再試行だけを原因確定へ昇格していない。
- [x] レビューP2の確定・取消でframe検査を省略する経路を修正し、first-returnも含めてnative baselineを明示した。insert callbackのnative markedと確定runtimeのfalseを分け、必須baselineなしでは失敗にする。純testsとprobe compile、4構成24phaseの既存raw時刻への判定を通過。新HEADの実runner検証は別途回収する。
- [x] software反復測定で同じ`にほんg`の古い画像を再現し、native日本語callback後・52.32ms後の新KUCframe生成前に撮影開始した時系列を捕捉した。frame検査はこれを拒否し、後続の日本語画像を取得した。公開版再受入は別途未完了。
- [x] 専用Mac runnerのGPUI windowが左へ128pxはみ出す撮影条件を、所有windowだけ画面内へ移す検証変更で是正した。26eの実AX位置x=0・幅1024と全幅原画をRootが確認した。商用のwindow設定や本人desktopは変更していない。

## egui候補窓の重なり（26eの目視で発見）

- [ ] #11の候補窓重なりと標準TextEdit/firstRectの関係・原因はまだ確定していない。公開版の失敗原画は`public-runtime/public-ime/root-public-ime-accepted.json`で保持し、この後続版調査では観測用問い合わせとOS/client呼出しを分け、上流責任を推測で断定しない。
- [x] 空composerのcaretが文字行の位置・高さを持たないKCU側の不具合を1x/2xの回帰REDで再現した。公開KUCの空白測定から空行geometryだけを作り、値・graphemes・compositionは空のまま保持する。eguiも共有caretを使用し、1x1 fallbackを除去した。core/egui対象testとmacOS全体gateが成功、coverage86.68%を確認した。
- [x] 修正による画像差分をRootが原画・全画素で照合した。headless5枚は各83画素、Linux native15枚は各21画素で、左上の誤caret除去と文字行caret追加だけだった。3hostは各caseでSHA一致。確認済みの20件のexact hashだけを更新し、許容差・検査範囲・基準は変更していない。
- [x] 修正後の固定HEADと公開版で実候補窓・preedit/変換/確定/取消を背景再受入し、原画を確認した。機械契約は通過したが#7/#11の表示は不合格のまま維持する（`public-runtime/public-ime/root-public-ime-accepted.json`）。

46eの追加traceではOS/clientからの初回firstRect参照が1x1、後続文字行が高さ20であることを確認した。同じ不正初期rectでも46e原画では重なりが再現しなかったため、候補窓事象の原因・解消の断定とは分離する。修正後はegui本体に加えて同条件5回の実IME測定を回収し、初期rectと原画を確認する。

## 変換中caretの共有描画

- [x] native IME位置はpreedit.caret、共有描画はselection.caretを使う不一致を実RGBA回帰で再現した。日本語preedit終端の画素は期待text色0x242424ではなく背景0xf3f3f3で、actual RED101を確認した。
- [x] 共有paint_caretもpreedit.caretを優先し、非変換時は従来のselection.caretを使用する最小修正を行った。公開API・依存・ACPは不変。preedit終端のcaret全領域を検査する回帰はactual GREEN0となった。
- [x] この修正を含む完全gate、最新固定HEADのreview/CI・実IME原画・source Storybookを再確認する。2fa74ae candidateの完全gate/review/CI/Native証跡を回収した（`ci/ci-terminal-proof.json`、`native-collection/native-final/root-acceptance.json`）。これは公開版のapp/Storybook/IME受入ではない。

## 2026-10-27 公開互換修正と独立した依存更新

- [x] Issue作成だけで止めず、公式winit #3966の0.30.x backportと必要なCI前提修正を[PR #4803](https://github.com/rust-windowing/winit/pull/4803)へDraftで提出した。同一sourceのコードレビュー2回で指摘なし・thread 0を回収してReadyへ昇格した。HEAD `1fd545f`はcanonical CIの全43環境検査を含む49件が成功。元baseと候補で同因を再現したcargo-deny 4件は失敗のまま保持し、maintainer reviewと実公開は未完了とする。
- [x] 公式nightly配布metadataでi686の標準ライブラリとcompiler hostの有無を分離した。同じi686ターゲットを配布済み64bit hostで実build/testし、対象削除や検査skipをせずに上流CIの成功を確認した。
- [x] 公開registryと全7設定のoutdatedを再確認し、rustix 1.1.6、openssl-probe 0.2.2、対象構成のwl-clipboard-rs 0.9.5へ更新した。Rootが全7lockのversion・checksum・参照を独立照合し、それ以外のpackage情報とGPUIのlibc制約の不変を確認した。
- [x] この依存更新後の完全release-checkを実exit 0で回収した。Rootが545ファイルのSHA前後一致、行カバレッジ86.69%維持、native 15画像の既存baselineとの完全一致と寸法を独立照合した。描画fixtureの原画も確認し、実provider・実IME・公開版の受入とは区別した。
- [x] この依存更新を含む最終HEADのreview/CIを回収する。2fa74ae candidateのreview/CIを回収した（`reviews/`、`ci/ci-terminal-proof.json`）。source `46e5439`の結果は流用していない。
- [x] KCUの同一公開版を使うconsumer・app・Storybookを背景確認した（`public-runtime/build-result.json`、`public-runtime/public-ui/root-public-ui-accepted.json`）。winitの成功を別packageのfloem-winitや独立したGPUI/XIMの公開・受入へ拡大しない。corrected GPUI/XIM依存の実公開・採用は未解決のままKCU #7後続版に残り、0.2.1停止条件にしない。

## 0.2.1 候補の依存更新と追加検証

- [x] 公開版と全7manifest/lockを照合し、Comrak 0.59.0、katana-ast-lint 0.5.3、対象manifestのTokio 1.54.1へ更新した。削除済みtagfilter設定を移行し、block/inline HTMLの文字列保持を既存契約として検査した。GPUI/Linux互換性に必要なlibc pinは維持する。
- [x] Actionsの最新互換リリースを調査し、checkout/upload-artifact/rust-cacheのmoving refと最新patch tagの実commit一致も確認した。tool-name tagとstable channelを含め、現在の参照方針で最新を取得できることを記録した。
- [x] workspaceと内部依存4件を0.2.1に揃え、7lockの完全構造を照合した。通常hook付きの依存更新checkpointはa6419a1。0.2.0は未公開候補であり、公開済み最新版0.1.0と区別する。
- [x] 追加回帰を含む546ファイルを固定し、2026-10-27の完全release-checkを実exit 0で回収した。行カバレッジ10078/未実行1341/86.69%、native15とheadless5の既存画像基準を維持した。前段のfmt失敗や初回86.68%の測定を最終結果へ書き換えていない。
- [x] 公開registry依存を使う596 baselineのLinux実IBus/Mozcを3hostで再測定し、確定1回・送信なし・取消後host marked=falseを確認した。Rootの原画確認ではGPUIの変換中重複とegui/Floemの候補窓重なりが残るため、視覚受入は不合格のまま維持する。0.2.1や公開版の受入へ流用しない。
- [x] 過去6e09の538ファイルと当時と同じruntime image contentを固定し、narrow GPUIを再取得した。期待SHA 58d1dc3c…ca52と一致し、別の1回観測では窓ID/実PIDとX11のMap/Configure/Visibility/Expose/終了イベントも記録した。正常再取得は当時の失敗原因の確定を意味しない。
- [x] 現候補8f7c8f3のDraft PR12・CodeReview 2回指摘なし・thread 0・CI全9job/79step成功を実回収した。通常pushの全hookもexit 0、source546不変・remote同一を確認した。旧596の成功とは別の証拠として保持する。
- [x] 公開registry依存を使う最終commitでDraft review 2回、指摘修正/reply/resolve、required CIを回収する。2fa74ae candidateの3回cloud review、指摘なし、required CI成功を回収した（`reviews/`、`ci/ci-terminal-proof.json`）。corrected upstreamの実公開・採用はKCU #7の後続版作業とする。
- [x] 公開版と同一のsource/lock/binary sealでLinux実IMEを再測定し、入力/確定/取消/誤送信契約と表示結果を記録した。#7/#11は既知制限として不合格を維持し、IssueをOpenで後続版へ残す。修正済み上流の実公開と2事象の解消は0.2.1条件から分離した（`public-runtime/public-ime/root-public-ime-accepted.json`）。
- [x] 全必須gate/coverage/画像基準、最終HEADのDraft review・指摘対応・P0/P1ゼロ・required CI・merge・全5crate公開・registry consumer・公開版app/Storybook受入を完了した。Linux IMEでは#7/#11だけ不合格を維持しIssueをOpenで後続版へ残す（証跡は上記public/runtime、`ci/ci-terminal-proof.json`、`release-actions/root-postmerge-jobs-accepted.json`）。

元の単色画像1件は未解決のまま[独立追跡](../../../docs/gpui-black-frame-followup.md)へ残す。原因特定を完了にせず、今後発生する受入失敗も免除しない。

## 2026-10-28 KUC patch更新

- [x] 公開KUC0.5.1のchecksum/source/MSRVを確認し、2つの直接manifestと全7lockを通常のupdate-safeで更新した。解決済み依存とRustソースは0.5.0から不変。
- [x] KUC0.5.1更新後の完全release-checkを実exit0で回収した。source547ファイルのSHA不変、行カバレッジ10078/未実行1341/86.69%、既存画像基準とLinux Storybook smokeを維持した。新HEADのreview2回/CIはPRの実結果で別途回収し、旧HEAD788bd8cの成功を流用しない。Linux実IME表示・修正済み上流の公開・同じ公開版app/Storybookの背景受入は引き続き未完了。

## macOS CIのビルドキャッシュ

- [x] HEAD54fd101の通常CIログでKUCのコンパイル7回と、外部workspaceごとのビルド先の分散を確認した。ローカルで完全gate成功済みの共有target/debug情報設定をmacOSの2jobへ揃え、検査・閾値・画像基準・timeoutは維持する。
- [x] 設定変更後の完全release-checkは実exit0、547ファイルSHA不変、行カバレッジ86.69%・既存画像基準を維持した。workflow構造比較と自己レビューで検査条件の不変を確認した。
- [x] HEAD1c3f7e3のreview2回と通常CI40548812632/公開前CI40548812650を終端回収し、全9job/79step成功・skip0を確認した。変更前54fd101のCIも回収済み。これは後続依存更新の受入へ流用しない。

## KUC 0.6公開版の評価と採用

- [x] 正規registry公開KUC0.6.0のsource/checksum/MSRV/featuresと使用APIを0.5.1と比較した。2manifest/全7lockを更新し、既存Backspace分岐とconsumerのKUC0.6型境界を確認した。globset0.4.21/ignore0.4.34の互換更新も解決した。
- [x] この依存graphの完全release-checkを実exit0で回収し、行カバレッジ10078/未実行1341/86.69%、native画像15枚の既存SHA/寸法一致、Storybook契約と専用Linux smokeを確認した。2回のENOSPC失敗も保持し、容量復旧後の同一source547件の再検査結果として記録した。
- [x] KUC0.6を含む公開用の最終HEADで通常push・Draft review2回・指摘対応と全CIを回収する。2fa74aeで通常push、3回cloud review、指摘なし、CI 9 jobs/79 steps成功を回収した（`normal-push/push-result.json`、`reviews/`、`ci/ci-terminal-proof.json`）。中間候補の結果は流用していない。公開版consumer/app/Storybook/Linux IME受入はこの節とは別の証拠で完了し、上流IME修正版の公開・採用だけが未解決である。
