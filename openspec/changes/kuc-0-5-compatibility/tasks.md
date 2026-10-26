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
- [ ] historicalblack原因未確定の追跡と、既存DoDの扱いを解決する。正常再試行だけで解決にしない。

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
