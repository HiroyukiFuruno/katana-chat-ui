# KUC 0.5互換性の実装・受入

## 上流と独立した作業

- [x] 公開KUC0.5.0のchecksum/MSRV/公式tagと、0.4.4からの対象API・文字測定差分を確認する。
- [x] KUC/ACPのmanifestと公開5crateのversion条件、全7lockを更新し、locked metadataを確認する。
- [x] 第8回完全gateでcore170/Unicode・geometry/3adapter/host/AST/lint/coverage86.67%/既存headless5・native15/Storybook smoke/packageを実exit0で回収した。source311件は前後SHA一致。
- [ ] 現sourceのStorybook原画・操作契約・source/bin対応は記録済み。Native appの最終HEAD視覚受入と公開版再受入は未完了。
- [x] 既存無償/ローカル環境で4provider通常応答とCodex履歴/取消、Claude Gemma permission拒否/取消、Katanモデル選択を確認し、未実施vendor/model操作と分離して記録した。
- [x] 仕様/証跡の古い完了表示を照合し、source/公開版/単体/実通信/実IME/目視を区別した。公開版受入とhistoricalblackは未完了を維持する。

## 公開版の最終受入

- [ ] corrected public decoder/zed-xim/GPUIをregistryから取得し、最終版へ未公開overrideがないことを確認する。
- [ ] 公開依存を使うLinux GPUI重複解消と3hostの実IME契約を再受入する。
- [ ] 最終HEADのDraft review→指摘対応/reply/resolve→P0/P1ゼロ→Ready→required CI→mergeを完了する。
- [ ] GitHub ActionsでKCU新versionを公開し、registry/Release/独立consumerを確認する。
- [ ] 同じ公開版のappとStorybookを背景目視し、機械契約・原画・source/package/bin一致を回収する。
- [ ] 公開後に不要branch/worktree/stashを既存ルールで整理する。既存他作業の差分は保持する。

## ユーザーフィードバック

- [/] 上流待ち以外の作業を止めない。KUC0.5更新と現source/実通信の検証を独立して進める。
- [ ] 未検証を完了扱いせず、修正版の公開後にagentが公開版app/Storybookをbackground確認する。現sourceのegui候補窓の重なりは視覚受入の残件として維持する。
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

- [ ] 原画で入力文字を覆う実候補窓と標準TextEdit比較を記録し、空入力時の1x1 anchorと実firstRect問い合わせの関係を背景CIで切り分ける。観測用問い合わせとOS/client呼出しを区別し、上流責任を推測で断定しない。
- [x] 空composerのcaretが文字行の位置・高さを持たないKCU側の不具合を1x/2xの回帰REDで再現した。公開KUCの空白測定から空行geometryだけを作り、値・graphemes・compositionは空のまま保持する。eguiも共有caretを使用し、1x1 fallbackを除去した。core/egui対象testとmacOS全体gateが成功、coverage86.68%を確認した。
- [x] 修正による画像差分をRootが原画・全画素で照合した。headless5枚は各83画素、Linux native15枚は各21画素で、左上の誤caret除去と文字行caret追加だけだった。3hostは各caseでSHA一致。確認済みの20件のexact hashだけを更新し、許容差・検査範囲・基準は変更していない。
- [ ] 修正後の固定HEADで実候補窓・preedit/変換/確定/取消を背景再受入し、機械検査成功だけで視覚合格としない。

46eの追加traceではOS/clientからの初回firstRect参照が1x1、後続文字行が高さ20であることを確認した。同じ不正初期rectでも46e原画では重なりが再現しなかったため、候補窓事象の原因・解消の断定とは分離する。修正後はegui本体に加えて同条件5回の実IME測定を回収し、初期rectと原画を確認する。

## 変換中caretの共有描画

- [x] native IME位置はpreedit.caret、共有描画はselection.caretを使う不一致を実RGBA回帰で再現した。日本語preedit終端の画素は期待text色0x242424ではなく背景0xf3f3f3で、actual RED101を確認した。
- [x] 共有paint_caretもpreedit.caretを優先し、非変換時は従来のselection.caretを使用する最小修正を行った。公開API・依存・ACPは不変。preedit終端のcaret全領域を検査する回帰はactual GREEN0となった。
- [ ] この修正を含む完全gate、最新固定HEADのreview/CI・実IME原画・source Storybookを再確認する。27fの空caret修正までの結果を新修正の受入へ流用しない。

## 2026-10-27 公開互換修正と独立した依存更新

- [x] Issue作成だけで止めず、公式winit #3966の0.30.x backportと必要なCI前提修正を[PR #4803](https://github.com/rust-windowing/winit/pull/4803)へDraftで提出した。同一sourceのコードレビュー2回で指摘なし・thread 0を回収してReadyへ昇格した。HEAD `1fd545f`はcanonical CIの全43環境検査を含む49件が成功。元baseと候補で同因を再現したcargo-deny 4件は失敗のまま保持し、maintainer reviewと実公開は未完了とする。
- [x] 公式nightly配布metadataでi686の標準ライブラリとcompiler hostの有無を分離した。同じi686ターゲットを配布済み64bit hostで実build/testし、対象削除や検査skipをせずに上流CIの成功を確認した。
- [x] 公開registryと全7設定のoutdatedを再確認し、rustix 1.1.6、openssl-probe 0.2.2、対象構成のwl-clipboard-rs 0.9.5へ更新した。Rootが全7lockのversion・checksum・参照を独立照合し、それ以外のpackage情報とGPUIのlibc制約の不変を確認した。
- [x] この依存更新後の完全release-checkを実exit 0で回収した。Rootが545ファイルのSHA前後一致、行カバレッジ86.69%維持、native 15画像の既存baselineとの完全一致と寸法を独立照合した。描画fixtureの原画も確認し、実provider・実IME・公開版の受入とは区別した。
- [ ] この依存更新を含む最終HEADのreview/CIを回収する。source `46e5439`までの成功を新lockの合格へ流用しない。
- [ ] 正規の修正済み公開依存を取得した後、同じ公開版のconsumer・app・Storybookを背景確認する。winitの成功を別packageのfloem-winitや独立したGPUI/XIMの公開・受入へ拡大しない。

## 0.2.1 候補の依存更新と追加検証

- [x] 公開版と全7manifest/lockを照合し、Comrak 0.59.0、katana-ast-lint 0.5.3、対象manifestのTokio 1.54.1へ更新した。削除済みtagfilter設定を移行し、block/inline HTMLの文字列保持を既存契約として検査した。GPUI/Linux互換性に必要なlibc pinは維持する。
- [x] Actionsの最新互換リリースを調査し、checkout/upload-artifact/rust-cacheのmoving refと最新patch tagの実commit一致も確認した。tool-name tagとstable channelを含め、現在の参照方針で最新を取得できることを記録した。
- [x] workspaceと内部依存4件を0.2.1に揃え、7lockの完全構造を照合した。通常hook付きの依存更新checkpointはa6419a1。0.2.0は未公開候補であり、公開済み最新版0.1.0と区別する。
- [x] 追加回帰を含む546ファイルを固定し、2026-10-27の完全release-checkを実exit 0で回収した。行カバレッジ10078/未実行1341/86.69%、native15とheadless5の既存画像基準を維持した。前段のfmt失敗や初回86.68%の測定を最終結果へ書き換えていない。
- [x] 公開registry依存を使う596 baselineのLinux実IBus/Mozcを3hostで再測定し、確定1回・送信なし・取消後host marked=falseを確認した。Rootの原画確認ではGPUIの変換中重複とegui/Floemの候補窓重なりが残るため、視覚受入は不合格のまま維持する。0.2.1や公開版の受入へ流用しない。
- [x] 過去6e09の538ファイルと当時と同じruntime image contentを固定し、narrow GPUIを再取得した。期待SHA 58d1dc3c…ca52と一致し、別の1回観測では窓ID/実PIDとX11のMap/Configure/Visibility/Expose/終了イベントも記録した。正常再取得は当時の失敗原因の確定を意味しない。
- [x] 現候補8f7c8f3のDraft PR12・CodeReview 2回指摘なし・thread 0・CI全9job/79step成功を実回収した。通常pushの全hookもexit 0、source546不変・remote同一を確認した。旧596の成功とは別の証拠として保持する。
- [ ] 修正済み上流依存を採用した最終commitでDraft review 2回、指摘修正/reply/resolve、required CIを回収する。
- [ ] 修正済み上流の実公開と、Linux実IMEの視覚受入を完了する。他の受入条件を暗黙に緩和しない。
- [ ] GitHub Actionsで0.2.1を公開し、独立registry consumerと同じ公開版app/Storybookをbackgroundで再受入して、公開後の整理を完了する。

元の単色画像1件は未解決のまま[独立追跡](../../../docs/gpui-black-frame-followup.md)へ残す。原因特定を完了にせず、今後発生する受入失敗も免除しない。

## 2026-10-28 KUC patch更新

- [x] 公開KUC0.5.1のchecksum/source/MSRVを確認し、2つの直接manifestと全7lockを通常のupdate-safeで更新した。解決済み依存とRustソースは0.5.0から不変。
- [x] KUC0.5.1更新後の完全release-checkを実exit0で回収した。source547ファイルのSHA不変、行カバレッジ10078/未実行1341/86.69%、既存画像基準とLinux Storybook smokeを維持した。新HEADのreview2回/CIはPRの実結果で別途回収し、旧HEAD788bd8cの成功を流用しない。Linux実IME表示・修正済み上流の公開・同じ公開版app/Storybookの背景受入は引き続き未完了。
