# OpenSpec要件達成・リリース台帳

## 最新指示と完了条件

ユーザー指示: 「OpenSpecの要件を満たしてリリースまで行って下さい」。先行指示の純KUC依存、multi-vendor提供、ACP仕様影響確認、依存更新を維持する。

一次情報は`v0-1-0-scope.md`の0001〜0017と最新ユーザー指示。追加changeはこれを補うもので、既存P0要件をNon-Goalsへ落とさない。リリース版は現在の公開状態を確認して決定する。

## 2026-10-24 現在の検証状態

Draft PR #4（`53436e6`）の初回cloud reviewでOllama受信処理P1×2とACP config ID更新P2×1を取得した。Ollama修正は通常commit `7ff5c70` へ保存し、旧版の実2ケース失敗と現行各ケース成功、4ファイル復元SHA一致をrootが確認した（`pr4-remediation-{red-old,green-utf8,green-error,restore-hash}.log`）。ACPは通知・設定応答の候補と受理IDをsessionごとの共有状態で更新し、削除済みIDを非致命の設定拒否にする。ACP34件と既存`just lint ast-lint unit-test`はexit0である（`pr4-remediation-repository-gates-red-green-final.log`）。CIの新Rust lint4件とLinuxリンク依存不足も同値変換・依存導入で修正した。

修正後の完全release-checkも実child exit0となり、502ファイルの開始/終了SHA不変、行coverage84.26%（基準77%）、既存5画像、Linux native15画像、Storybook smoke、package gateをrootが独立確認した（`pr4-final-release-{result,root-accepted}.json`、`pr4-final-release-check-debug0.log`）。最新Storybook実対話・provider切替の9画像を照合し、rootが実PNGを目視した。現行binaryで実Codex履歴probeとKatanAgent/Ollama probeもexit0、Codexはprompt3/new3/config2・再送信なし・capture failure0を独立集計した（`pr4-final-real-visual-root-accepted.json`）。残る受入は修正commit/push、各thread返信・resolve、修正後cloud review、最新CI、公開・downstream・archive・hygieneである。以下の旧gate記録は経緯として区別する。

公開KUC0.4.4、Linux GPUIのlibc0.2.189互換制約、source祖先拒否を含むcheckpoint `88eae64` 時点のソースで、完全`just check`の構造化exit0を回収した（`tmp/verification/kuc044-full-just-check-libc189.log`）。rootが359ソースの凍結hash、既存5画像、Linux native15画像のhashを独立照合し、Storybookのprivate Xvfb実window smoke終端も確認した（`kuc044-fullcheck-root-accepted.json`）。strict OpenSpecは8/8成功。以下の2026-10-23以前の失敗・実行中記録は経緯であり、現在の判定にはこの節を優先する。

関心事別の4 checkpoint commitは完了し、rootが全359パスと旧native52ファイルのHEAD blobを照合した（`migration-checkpoint-chain-root.json`）。その後、release回帰fixtureが継承Git環境で別repositoryのconfigを変更するP1を隔離sandboxで再現した。全`GIT_*`の分離とown-root照合を追加し、恒久回帰は現行ソースだけでforeign HEAD/index/config/status不変を検証する。旧版redはignored証跡へ保持し、shallow cloneに旧commit履歴を要求しない。

fixture修正も通常hook付きcommit `af1a0d7` へ保存した。そのclean HEADで旧native52ファイルを`tmp/trash/2026-10-24-003943/`へ退避し、rootが全52 SHAと変更対象52件だけのdeletionを照合した。使用中source・assets等136ファイルのhashは不変である（`legacy-native-quarantine-root-accepted.json`）。

整理後の完全`release-check`も構造化exit0となった（`final-release-check-kuc044-cleaned.log`）。rootが499ファイルのhash不変、coverage行83.83%（基準77%）、既存5画像、Linux native15画像のSHAと寸法を独立照合した（`final-release-check-root-accepted.json`）。package検証はACPの実buildとpublish dry-run、残4crateのpackage一覧、全5crateの未公開確認であり、全5crateのpublish dry-runとは区別する。

最新binaryのprivate Xvfb実対話で入力・送信・固定応答・scroll・履歴選択を検証し、rootが実PNGを目視した。4provider候補とClaude CodeからCodex CLIへの切替も実画面で確認した（`storybook044-provenance.json`、`storybook044-provider-provenance.json`）。ホスト前面操作やprovider通信は行わず、app/Xvfb/containerの終了も確認した。

残る順序は、整理差分の正式保存、Draft PR・最低2回cloud review・CI/merge、5crate公開・独立registry consumer受入、OpenSpec archiveとbranch hygieneである。最終検証成功をリリース完了とは扱わない。

## 2026-10-23 公開・検証の経緯

環境設定の承認境界: ユーザーの「この設定をそちらでしてください」とOllama再downloadの指示は、作業者による無課金の環境設定を承認する。製品が起動時に認証・課金・モデル取得を自動実行しない契約を、その依頼済み作業の禁止へ読み替えない。残るprovider経路は無料/local構成で再設定・実接続を進め、本人認証が必要な場合だけその操作を依存条件として記録する。

KUC公開workflow run 39854764311は成功し、GitHub Release v0.4.3とcrates.ioからの取得を確認した。KCUの直接manifest2箇所と全7lockfileを公開registry版0.4.3へ更新済みである。本文hit outsetとcode専用raster設定を適用し、120/320/560pxの幅・pointer境界回帰およびcore 136テストは成功した。code専用のmacOS候補をSFNS.ttfへ合わせ、thinkingの行高漏れをglyph可視回帰のred/greenで修正した。5画像は基準を変えず個別に同一hashとなり、agentによる実画像再確認も完了した。厳格lintも成功した（`tmp/verification/kuc043-lint-complete.log`）。一方、実consumerの追加回帰でKUC #104/#105が再現したため、KUC `release/v0.4.4` で上流修正を実装した。上流PR #106は4回目reviewの非明示折り返し境界P1まで修正し、commit 54c1782の完全just checkは4755 tests/136 suitesと既存strict gateが成功した。通常push後の5回目cloud reviewは同一HEADについてcompleted・指摘なしと確認した。最新HEADの3 OS CI（39917955277）とNative IME compile（39917955326）は全てsuccessで、未解決review thread 0を確認してReady化した。release-preflight（39917955273）もrootの最新取得でcompleted/successを確認した。最終HEAD/review/required checksの再照合後にPR #106は通常merge済み（merge 87063e1、09:14:57 UTC）。Release run 39930491269はsuccessとなり、GitHub Release v0.4.4（native ZIP 3件）と独立workspaceでの公開registry版実downloadをrootが確認した（`tmp/verification/kuc-registry-info/cargo-info-katana-ui-core-0.4.4.log`）。KCUの直接manifest2箇所・全7lockへの採用と最終全体検証へ進む。KCUへは公開registry版だけを採用する。active change `kuc-0-4-1-card-compatibility` の1.5以降が残件の一次台帳である。以下の実装タスクの既存完了表示は過去の実装証拠を含み、現在の依存での全体受入完了を意味しない。

公開KUC v0.4.4を直接manifest2箇所・全7lockへ採用し、全7構成のupdate-safe/outdatedがexit0となった（`{update-safe,outdated}-final-kuc044.log`、root実検査`kuc044-seven-locks-root.json`）。最初のcore検査は160成功・2失敗だったが、実pixel/copy診断で、色付きglyph inkと下線の混同、および前行と共有する選択境界を指定するテスト前提の誤りを確認した。下線のx/幅の厳密一致、default glyph可視性、5区切りcopy、各変更pixelの既存containmentとactual hit内部pixel必須は維持し、対照色と行中央の入力で測り直した。rootは一度失われたactual hit内部pixel assertをレビューで復元させ、最終core162/162・focused5・strict lint・AST19・fmtのexit0を確認した（`kuc044-final-gates.log`）。全source凍結で完全just checkを実行中であり、最終release-checkはまだ未実施である。

全7構成の`just outdated`と`just update-safe`は成功し、ACP 3.5.0 / Tokio 1.54.1へ更新した。更新後はACP17、host-support75、adapter Egui13/Floem10/GPUI11、Storybook8、native host Egui7/Floem53/GPUI6が成功し、host-supportとnative hostのstrict lint/buildも成功した。証跡は`tmp/verification/`の`*-updated-*`ログに保存している。host E2Eはstrict lint成功後、adapter conformanceが7成功・message幅契約1失敗で停止した（`host-e2e-updated-check.log`）。KUC 0.4.3取り込み後に幅・code raster・bidi geometry/copy・既存5画面基準・完全release-checkを再検証する。

Storybookの自動window smokeはmacOSの前面操作を奪わないよう、既存Linux native matrixのprivate Xvfbで実windowを2 frame描画する構成へ移した。host側のstorybook-checkはentrypointとunit testを保持し、実window検査はrequired Linux matrixに含める。公開KUC 0.4.3で背景実window smokeは成功した（`tmp/verification/storybook-background-real.log`）。別途採取した対話windowの実画像をagentが確認し、日本語質問の2行目が見切れるKCU側の行高推定バグを発見した。`kuc-measured-message-row-height`で実KUC計測に基づく行高とscroll範囲へ修正し、layout21件・strict Clippy・AST19・fmtは成功した。従来の1行44pxを維持する回帰も追加した。最終ソースの `harness-screenshot-check` はexit0で全5画像が既存hashと一致し、agentが実画像を再確認した。背景Linux実Storybookもsmoke exit0で、日本語2行目の見切れ解消、ASCII入力・送信・固定応答・末尾scroll・Codex CLIへのprovider切替を実画面で確認した（`tmp/verification/storybook-native-interaction.log`、再現scriptも同directory）。scroll後のヘッダー重なり疑いは40px全域の画素差0と座標で否定した。これらは公開KUC 0.4.3上の局所検証であり、公開0.4.4採用後の最終全体gateとnative matrixは未完了である。

Codex ACP 2.2.2の同一session 2応答に加え、CopilotとKatanAgentの実応答、Claude Agent ACP 0.94.0のlocal qwen3:4b経路も2026-10-23のprobeで成功した。Claudeは検証用CLAUDE_CONFIG_DIRの通常default permission modeを使用し、追加認証・課金・global設定変更はしていない。証跡は `tmp/verification/{codex-acp-latest-real,copilot-current-real,katanagent-current-real,claude-local-default-mode-real}.log`。ACPの許可context表示とmodel確定応答反映は最終reviewで追加修正し、`acp-permission-model-confirmation`で検証を継続する。最終検査、Draft PR、最低2回のcloud review、Ready/CI/merge、5crate公開とdownstream確認、branch hygieneは引き続き未完了である。

独立した公開手順reviewで、任意branchのworkflow_dispatchが公開へ進めるP1を修正した。`release-source-admission` は通常mergeと手動の既存tag復旧を検証し、同一SHA/version出力を両OS検査と公開jobへ渡す。実Git fixture回帰とShellCheckはrootでexit0、全OpenSpec strictは7/7、独立reviewも指摘なし（`tmp/verification/release-source-admission-root-final.log`、`release-source-shellcheck.log`、`openspec-release-source-root-final.log`）。実CI・公開の確認は親リリースgateへ残す。

KUC v0.4.4の上流対応は公開確認後に#104/#105/#107へ証跡を記録してcloseし、masterをorigin/masterへfast-forwardしてclean/stash0を確認した。マージ済みlocal release/v0.4.3・v0.4.4を通常削除し、remoteの手動削除はしていない（`tmp/verification/kuc044-final-hygiene.json`）。閉じたIssueの外部編集許可はignored証跡へ移し、以後KUC sourceを変更する場合は新たなOpen Issueが必要である。

## 2026-10-18 KUC 0.4.1 互換性待ち（過去の経緯）

公開 `katana-ui-core 0.4.1` と `agent-client-protocol 3.4.0` への更新後、最終release-checkは5画面screenshot matrixで停止した。KUC 0.4.1の`Card`は公開設定できない16pxのchild indentを加えるため、KCUの560px wide message幅契約と既存視覚基準を同時に満たせない。KCUは画像基準を更新せず、上流Issue [katana-ui-core#83](https://github.com/HiroyukiFuruno/katana-ui-core/issues/83) の公開修正版を待つ。

この待機は `openspec/changes/kuc-0-4-1-card-compatibility/` の1.3として追跡する。未公開KUCへのpath/git/fork依存、品質基準の緩和、baseline hashの更新を行わず、公開版取得後にfocused layout・画像matrix・release-checkを再実行する。egui/eframe/egui_kittest 0.37.0は公開registryから既に採用し、Egui adapter、manual Egui host、Storybook、screenshot executorの局所検証を通過した。KUC互換版との最終画像・release検証は残る。

## 禁止事項・検証原則

- 旧native widget経路のgreenや共通UiTree変換だけで純KUC移行完了としない。
- source文字列一致、provider CLIのversion検出だけで実操作・通信が動くとしない。
- 要件を既存実装へ縮小しない。検証対象・coverage77%・lintを緩和しない。
- KCUは公開KUC primitiveによる共通runtimeを実装する。必要な上流修正はOpen Issue #104/#105/#107と登録済み外部編集許可のKUC範囲に限定し、KCUには公開registry版だけを採用する。その他の別repoは診断とhandoffまでとする。
- 既存`.serena/project.yml`は設定値を保持する公式schema移行として検証済み。製品実装とは関心事を分けて履歴へ取り込む。生成物`.DS_Store`と`__pycache__`は共有しない。
- commit/push/releaseは今回のリリース指示の範囲で行い、Draft→review→指摘取得→修正/reply/resolve→再review→Ready→checks→merge→公開確認を守る。
- 再開・圧縮・subagent統合時にはこの台帳と最新指示を再読する。

## 実装タスク

### Draft PR #4 初回レビュー（2026-10-24）

- [x] P1: OllamaのHTTP chunkがUTF-8文字の途中で分割されても、完全なNDJSON行になるまでバイト列で保持する（discussion_r4361777631。旧版RED・修正版GREEN、commit `7ff5c70`）。
- [x] P1: Ollamaのstream内errorを成功扱いせず、部分応答後でも失敗を伝播する（discussion_r4361777653。同じpartial content条件で旧版RED・修正版GREEN）。
- [x] P2: ACPの通知・設定応答が更新したconfig IDを、UI候補と同じ共有状態で受理する（discussion_r4361777668。通知・response・削除ID拒否・session分離・通知先closed時不変の回帰成功）。既存ACP要件の不具合修正であり、scopeやDoDは変更しない。
- [ ] 修正後の回帰・全品質gate、正式commit/push、各threadへの根拠返信とresolve、最低2回目のcloud reviewを完了する。

- [x] 0001〜0017を実装・直接検証の証拠と照合する（`docs/release-requirements-audit.md`。2026-10-18の最新release-check、OpenSpec archive、agentによるStorybook 5画面目視を反映）
- [x] 0001 旧scope archive境界を維持する（旧changeは`openspec/changes/archive/2026-10-18-v0-2-0-pure-kuc-multivendor-acp/`へarchive済み。2026-10-18の一覧は空だったが、現在は後続の互換修正・ACP修正のactive changeを本台帳で追跡する）
- [x] 0002 debug/settings surfaceを標準UIから排除する（標準tree不在回帰と実KUC frame/dispatch conformance、18167 PASS）
- [x] 0003 header左の利用可能provider selectorを維持する（空activeでもReady候補を保持、KUC選択action/候補除外回帰、53468全check PASS。実OS操作は0004で別途未検証）
- [x] 0004 KUC共通composerでIME・改行・一度だけ送信・draft clearを満たす（native adapter回帰とrelease-checkで確認。現在未利用の初回リリースはユーザー明示により物理macOS日本語IME受入を延期し、初回consumer integrationで実施する）
- [x] 0005 composer control・円形context usage・狭幅を満たす（実frame/画素回帰と39967全check PASS、独立review P0/P1なし）
- [x] 0006 user/agent/activityの幅・位置契約を満たす（上下余白・長短本文幅不変・thinking/execution/processing除外、39967全check PASS）
- [x] 0007 4provider/runtime/model境界と利用可判定の実接続を確認する（2026-10-23のCodex ACP 2.2.2は同一sessionの2応答に成功。再buildしたKCU probeでCopilot・KatanAgentも成功し、Claude Agent ACP 0.94.0は検証専用CLAUDE_CONFIG_DIR・通常default permission mode・ローカルqwen3:4bで厳密応答検査に成功した。証跡は `tmp/verification/{copilot,katanagent}-current-real.log`、`claude-local-default-mode-real.log`。課金・trial・追加ログイン・model downloadやglobal設定変更はない。承認contextとmodel確定応答の最終review修正は別active changeで追跡し、リリース完了とはしない）
- [x] 0008 Markdown生成outputと許可path契約を実入力から満たす（local Ollama実KatanAgentが生成したfile candidateを、隔離`tmp/`へ明示CreateFile適用するprobeがPASS）
- [x] 0009 Markdown編集outputと物理書込境界を満たす（local Ollama実KatanAgentの添付Markdown edit→diff candidate→明示ApplyDiffがPASS。macOSの実体パス正規化回帰も追加）
- [x] 0010 対象変更だけの取り消しと失敗理由を満たす（実KatanAgentのApplyDiff→同一output UndoChangeで元本文を復元。失敗理由はhost-support回帰で保持）
- [x] 0011 session履歴保存と最後のsession復元を満たす（実Codex ACPで保存→新Host→latest復元→model保持→会話継続を確認）
- [x] 0012 履歴一覧と再送信なしのsession復元を満たす（KUC history action/layout回帰と、実Codex Managed SelectHistoryで明示3 prompt以外の再送信なしを確認）
- [x] 0013 Markdown minimumをKUC描画で満たす（構造化block/raw HTML/code fenceの12 Markdown回帰、long MarkdownのKUC ScrollArea実pixel scroll回帰、runtime 42件PASS）
- [x] 0014 3hostを共通KUC runtimeへ接続し操作・配置・責務を一致させる（標準entryの共通ChatUiSurface/runtime、29316 conformance、85035の同scenario3host画像一致。IME/実providerは別番号で未完了）
- [x] 0015 3host主要layoutのheadless検証を成立させる（85035 exit0/native15画像、Xvfb仮想display、寸法とhash必須検証、通常check組込。実CIはリリースgateで未完了）
- [x] 0016 標準UIの仕上げと最終native確認を行う（agentが実Storybook artifactのempty/narrow/standard/tall/thinkingの5画面を目視し、header、composer、会話、出力カード、処理表示の破綻なしを確認）
- [x] 0017 activityとplaceholderの分離を実経路で満たす（7種typed activity、本文到着時切替、完了・失敗時clear、placeholder/IME分離の回帰に加え、実KatanAgent/OllamaでProcessing表示から完了時clearまで確認）
- [x] Storybookを公開KUC runtime上のprovider-free対話面として追加する（`just storybook`はrelease native windowを起動、`storybook-check`はentrypointと4 providerの入力/送信/履歴/model選択のcontroller unit契約を検査する。描画frameの入力routingは共通runtime/adapterの別回帰で検査し、Linux native matrixで実window smokeを検査する。controller unit単独を実window操作の証拠にはしない。実provider、物理IME、最終人間UI確認の代替にはしない）
- [x] ACP仕様影響の必要修正を既存P0要件との関係で確定・実装する（v1 wire変換/capability/permission境界を維持し、provider profileのchild環境引渡しを追加。過去の4provider実prompt完了と現在の利用可判定を文書で分離し、未認証・未導入環境で自動認証やmodel取得を行わないことを確認）
- [x] 依存更新の採否を最終実装で再検証する（公開KUC0.4.4、ACP3.5.0、egui0.37.0を全7構成で採用。全7outdatedと整理後release-checkはexit0。Linux GPUIのlibc0.2.189互換制約は理由を文書化し、品質基準を維持した）
- [x] 0012追加回帰: 長い会話でも右上の履歴操作からsession一覧を表示・スクロール・選択できる。正しいaction hitでのred（`tmp/verification/storybook-history-red-corrected.log`。初回node hit判定のredは無効）から独立scrollを修正した。目視で追加検出した背後会話との重複・空ラベル・長title見切れも、最終描画tree分離とwrapped KUC Textで修正済み。rootが現行ソースのfocused13件・full workspace strict lintのexit0を回収した（`history-readability-root-final-{focused,lint}.log`）。private Xvfbの履歴開閉・選択の実PNGもroot目視済み。追加クリックcoverageのP1（title40px/action20px）は実KUC測定の明示row高で修正した。row高合計が履歴header22pxを落とす二次P1もglobal max boundsへ修正し、旧SUM1342/1320のredと同一test greenを記録した。rootの最終14回帰・whole strict lint・AST19・fmtはexit0、private Xvfb実preview下段clickも成功し、rootが最終PNGを目視した（`history-readability-root-accepted-{focused,lint,ast}.log`、`history-readability-multirow-{old-sum-red,globalmax-green}.log`、`history-readability-native-preview-click-final.log`）。公開KUC0.4.4採用後の最終全体gateは別途未完了。

## 検証cacheの同時整理防止（実失敗の再発防止）

2026-10-23の共有target再検査は、macOS側の全段階と既存5画像hashが成功した後、LinuxのGPUI依存ビルドでexit1となった（`tmp/verification/kuc044-full-just-check-shared.log`）。`gpui 0.2.2 → gpui_http_client 0.2.2 → zed-async-tar 0.5.0-zed → xattr 0.2.3` が、`libc 0.2.191`に存在しない`ENOATTR`を参照する。詳細ログは`tmp/verification/kuc044-linux-gpui-xattr-build-failure.log`へ保存した。最終Storybook window smokeには到達しておらず、公開依存の互換性を修正して同じ完全gateを再実行する。

- [x] cleanup対象のsource祖先を拒否するP1を修正した。正しい所有タグがあってもtracked/nonignored fileを含む対象は拒否し、literal pathspec・Git環境分離・checkout root照合・Git失敗時拒否を追加した。rootの実配置18回帰はexit0（`tmp/verification/guard-source-boundary-root-unit.log`）。全7manifestの実metadata・strict OpenSpec・dry-runも成功した。全体gateの受入とは区別する。
- [x] Linux GPUIの公開推移依存の互換性を修正し、最新化の不採用理由を依存判断記録へ残して全体検証した。libc0.2.189制約で完全check exit0、全7構成のoutdatedもexit0（`outdated-final-libc189.log`）。最終release-checkは別タスクで確認する。

- [x] 公開KUC0.4.4の完全check中にnested screenshot targetが整理された原因を特定する。初回のdep-info書込exit101は`kuc044-full-just-check.log`に保持し、製品回帰の成功やgate通過とは扱わない。
- [x] KCUのCargo実行/clean/sweepとscreenshot entryを同じrepo lockで保護し、全7targetとunguarded open file/不正targetを検査する。実processによる回帰を通常gateへ追加する（`build-cache-clean-serialization`）。別repoは編集せず、既存の検証範囲と基準を維持した。
- [x] 局所検証・root review後に同じ完全just checkを再実行し、exit0を回収してからcheckpointへ進む（`kuc044-full-just-check-libc189.log`、root独立照合`kuc044-fullcheck-root-accepted.json`）。

## 旧native source整理（既存移行残件）

- [x] 公開KUC採用後の完全check成功と事前migration commitで、既存差分を正式履歴へ保存した。旧312 path indexを保全・再構成し、A317bb92/B0738346/C6cfc758/D88eae64の通常hook付き4commitと全359 HEAD/現物hash、旧native52 HEAD blobの一致を確認した（`migration-checkpoint-chain-root.json`）。
- [x] 公開入口・mod/path/includeから到達しない旧Rust source52件だけを監査済みhash一覧で確認し、framework別のbatchで`tmp/trash/2026-10-24-003943/`へ退避した。rootが52 SHAとdeletion対象の一致、使用中source・assets等136ファイル不変を照合した（`legacy-native-quarantine-root-accepted.json`）。実行中worktreeは保持し、runtime fallback不在の監査を維持する。
- [x] 整理後の全品質・画像・coverage・package gateを最終release-checkで維持する（`final-release-check-root-accepted.json`、行83.83%、native15画像一致。事前commit後の退避で基準変更なし）。
- [ ] 検証済みの整理差分を正式履歴へ取り込み、PRと公開確認まで進める。

## リリースタスク

- [x] 公開状態を確認しtarget versionを固定する（v0.1.0、全5crate同版未公開、tag/Releaseなし、既存PR#2はclosed）
- [x] 全OpenSpec項目の直接証拠を最新差分で確認する（0001〜0017、pure KUC、multi-vendor、ACP、依存更新を2026-10-18のrelease-checkと監査表で照合）
- [x] 最新lockfileで `just check` と `just VERSION=v0.1.0 release-check` を最終再実行する（`kuc044-full-just-check-libc189.log`と整理後`final-release-check-kuc044-cleaned.log`はexit0。後者で現行fixture隔離と旧source整理を含む全check・coverage83.83%・画像基準・package gateを再確認した）
- [x] 自己レビューと未対応指摘を解消する（Claude fixtureの有償model表示とACP wire未提供の旧specを修正。最新差分のP0/P1なしをchange reviewへ記録）
- [ ] release branchへcommit/pushしDraft PRを作成する
- [ ] cloud reviewを最低2回行い最後の指摘を解消する
- [ ] P0/P1ゼロとrequired checksを確認しReady化・mergeする
- [ ] Release workflow、tag、GitHub Release、公開registryで全対象crateを確認する
- [ ] branch-hygieneで不要な一時worktree/branchを整理する（remote削除はしない）

## ユーザーフィードバック

- [/] 「要件って満たせてるの？」: 実装要件の直接証拠と初回release-checkは揃えたが、KUC 0.4.1更新後の既存画像基準とwide幅契約の両立が公開APIでは不可能と判明した。上流Issue #83の公開修正版、最終release-check、PR/公開が未完了であり、現時点ではリリース完了ではない。egui 0.37.0更新と局所検証は完了している。
- [/] 「openspecとか見た？」: 既存scope全体を読み、実装・検証と番号単位で照合する。未達番号は本台帳で継続する。
- [/] 再発防止: 新changeの狭いNon-Goalsや旧経路のgreenを既存P0要件の代わりにしないことを本台帳へ固定した。
- [/] 「storybook全然動いてなくない？」: native harnessとStorybookを混同せず、`just storybook`の対話起動と`storybook-check`を追加して実native smokeまで確認した。agentによる5画面目視は完了したが、KUC 0.4.1更新後の画像matrixは既存基準不一致を検出したため、上流修正版後に再確認する。

## 現状

### 最新検証（下記の過去経過より優先）

- 2026-09-07 最新lockfileで `just fmt`、`just lint`、`just ast-lint`（19件）、`just unit-test`（185件）、`just host-support-check`（75件）、`just host-e2e`（9件）、manual Egui/Floem/GPUI gate（7/53/6件）、headless screenshot matrix（adapter34件・tool12件・5 scenario）、Linux native matrix、`just coverage`（82.86%/基準77%）を成功させた。0004/0016の手動確認と0001のarchive DoD解釈は自動gateで代替しない。

- 2026-09-07 Storybook要件を追加。公開KUC 0.3.7を通る`kcu-storybook`をworkspace toolとして実装し、`just storybook-window-smoke`をrelease profileで実行してnative window起動・自動closeをexit 0で確認。5件のfixture契約testとentrypoint検証も成功。実provider、物理IME、最終人間UI確認とは別の受入面として扱う。

- 2026-09-07 package事前検証として、`verify-version.sh v0.1.0`、`verify-internal-dependencies.sh v0.1.0`、`cargo package -p katana-acp-client --locked --allow-dirty`、残4 crateの`cargo package --list`、`cargo publish -p katana-acp-client --dry-run --locked --allow-dirty`、`assert-crates-not-published.sh v0.1.0`を成功させた。最終の全crate publishはPR merge後のRelease workflowで確認する。

- 2026-09-07 `katanagent_output_probe` をlocal Ollama `qwen3:4b` でexit 0。実モデルが返したMarkdown file candidateを一時workspaceの`tmp/`へCreateFileで適用し、同じ実モデルの添付Markdown editからdiff candidateをApplyDiff、同一outputをUndoChangeして元本文へ復元した。検証中にmacOSの`/var`と`/private/var`差で既存添付をworkspace外と誤判定したため、実在Markdownのcanonical pathで境界確認する修正と回帰を追加した。

- 2026-09-07 `acp_history_probe` を実Codex ACPでexit 0。保存済みsessionを新Hostへ読み込み、Managed SelectHistory、NewChatのdraft分離、元sessionの再選択、履歴tokenでの会話継続を確認した。Codex RPCのメタデータは明示3 promptだけで、履歴復元によるprompt再送は0件。未選択Claude adapterの終了中トレースは証拠対象外にし、Codex traceだけを解析するようprobeを修正した。

- 2026-09-07 KUCを公開0.3.7へ更新し、`cargo update --package katana-ui-core --precise 0.3.7`をexit 0で完了。KCU composerは公開KUCの測定content boundsでscroll boundsを同期し、scroll offsetをRGBA source cropへ反映するよう修正した。複数行入力のscrollでメタデータだけでなくcomposer領域の実pixel差分が出る回帰を追加した。続いてthreadも実message heightからKUC ScrollAreaのcontent heightを算出し、offsetを末尾でclampするよう修正。long Markdownのscrollでthread領域の実pixel差分が出る回帰を追加し、`cargo test --locked -p katana-chat-ui markdown` が12件、`kuc::runtime::tests`が42件PASS。0013はここで完了とするが、全workspace quality gateと実IMEは別要件として未完了。

- 2026-09-07 KCUのClaude ACP profileに、`KCU_CLAUDE_OLLAMA_MODEL` 指定時だけOllama互換の `ANTHROPIC_*` を子processへ渡す経路を追加。transport/profileのscope testは各PASS。親processにAnthropic設定を置かない `KCU_CLAUDE_OLLAMA_MODEL=qwen3:4b KCU_OLLAMA_ENDPOINT=http://localhost:11434 cargo run --locked --manifest-path tools/host-support/Cargo.toml --bin acp_provider_probe -- claude-code` がexit 0で、initialize/session/new/prompt/完了応答を実確認した。Qwenの応答は105秒を要したためprobe上限を180秒に更新。これはローカルOllama経路だけのClaude adapter実通信証拠であり、Copilot Free実通信や全provider受入を完了扱いしない。

- 2026-09-07 最新transportで `acp_provider_probe` をCodex CLI、Copilot Free、KatanAgent/Ollamaにも実行し、全てexit 0。Copilotは本人完了済みGitHub OAuth/TOTPのFree accountで、trial・有料契約・BYOKなし。KatanAgentはACP adapterではないため、同probeにdirect Ollama catalog/model選択/stream完了経路を追加して検証した。4 providerのtext prompt完了は確認済みだが、実IME、Markdown長文scroll、最終全品質gateは別要件として未達のまま。

- 2026-09-07 KUC 0.3.7 lock更新はrootだけでなくscreenshot、E2E host、manual Egui/Floem/GPUI、host-supportへ適用。E2E hostの直接KUC指定も0.3.7へ更新し、全7 lockfileにKUC0.3.6が残らないことを確認。全workspace quality gateとoutdated再評価はこの後に実施する。

- 2026-09-07 モデル削除に関する追加指示後、親が `ollama list` を再実行し qwen3:4b / 2.5GB / ID359d7dd4bcda の導入を再確認。Claude Codeの単発ローカルenv probeは担当報告exit0、`tmp/claude-ollama-qwen3-4b-probe-20260907-attempt2.log` に KCU_OLLAMA_CLAUDE_OK 応答（モデル認識/context警告あり）。永続KCU設定・ACP実応答・全要件達成は別途未検証。

- 2026-09-07 Copilotログイン: `copilot login --web-flow` session74507で開始。既存GitHubアカウントのConfirm access/TOTP画面で本人入力待ち（Chrome tab1477740410をhandoff保持）。トークン・OTPを収集せず、有料申込なし。Claude通常OAuthは自身のprocessをTERMし46703 exit143で停止確認。親がOllama /api/tagsのqwen3:4b存在を確認。

- 2026-09-07 再開: ユーザーがClaude再認証・Copilot/Ollama環境設定を依頼し、Claude/Copilotの有料契約をしないと明示。課金・有料trial・クラウド有料モデルは使わない。Copilot CLI 1.0.83導入完了、Free認証と実通信は未確認。Ollama qwen3:4b導入とJSON Schema実応答を担当が確認、親によるKCU接続検証は残る。Claudeの通常OAuthログインを中止し、公式Ollama互換APIを用いるローカルClaude Codeを検証する（Anthropic Claudeモデルの検証と混同しない）。旧停止境界監査は過去状態であり、設定作業を継続する。
- 2026-09-07 KUC 0.3.7公開取得を確認。担当の公開API比較ではscroll bounds/content_boundsは0.3.6にも存在し、未公開API待ちではなくKCUのcrop/offset実装不足。別repo編集を要件にせず、KCU公開API利用で修正を進める。依存取り込み・検証は未実施。

- 停止境界監査3回目: 最新入力もgoal継続のみ。Claude再認証、Copilot/Ollama環境指定、KUC別repo許可、実IME直接検証の境界に変化なし。直前2turnとも同条件でno progress、進行中検査なし。完了済み69018/56495/22701と実Codex証拠を維持し、未達を完了へ変更しない。ユーザー入力または環境変更なしに有意な進捗を得られないためgoalをblockedへ更新する。再開時は最新指示と本台帳を再読する。

- 停止境界監査2回目: 前turnは回答待ちのno progress。最新ユーザー入力はgoal継続のみで、認証/環境指定/別repo許可の回答なし。0004/0007/0013等の未達を現台帳で再確認。進行中jobなし、公開/基準縮小/未許可編集を行わない。同じ停止条件の2連続目としてgoal activeを維持する。

- 停止境界監査1回目: 直前turnは69018/56495/22701の終端と新証拠を取得したprogress。本turnは残DoDを再読し全subagent完了を確認。現在進行中の検証なし。Claude再認証、Copilot導入方針、Ollamaモデル、KUC別repo許可は未回答。実OS IMEもCUA失敗後の直接証拠がなく未達。受入を省略して公開せず、同じ成功済みgateを無根拠に繰り返さない。本turnは新たな実装進捗ではなく回答待ちのno progressとして記録し、goalはactiveを維持する。

- 最新release-check69018 exit0（`tmp/model-boundary-release-check.log`）。quality/line82.65/native15・version/internal依存一致・ACP package実buildとpublish dry-run・残4crate package list・全5crate未公開確認PASS。全5crateのpackage実build済みとは表現しない。OpenSpec validate56495 exit0。outdated22701 exit0で7構成調査、親が52488ログと正規化依存行集合を比較して追加0/削除0。新しい直接更新候補なし、保留理由は既存docs/dependency-update.mdのまま。実公開/PRはまだなし。残条件はprovider環境・実IME・KUC公開API/scroll等で、質問への回答待ち。

- Claude login CLI helpで `claude auth login` を確認。Copilot executable不在、Ollama /api/tags models=[]を現環境で再確認。ユーザーへClaude再認証状況/Copilot導入方針/Ollama modelと、Issue先行KUC別repo修正公開許可をasync質問。未回答は許可と扱わない。その間、最新 `just VERSION=v0.1.0 release-check` を69018で `tmp/model-boundary-release-check.log` へ開始。公開実行ではなく事前gate、同handle終端前に重複Cargo/再起動しない。

- provider probeのError/Complete判定追加後45318 host-support-check exit0。Claude実通信25297 exit1（`tmp/acp-claude-existing-environment.log`）、OAuth session expired and could not be refreshed / authentication_failedを確認。CLI有無やACPReadyだけで実利用可能としない。Claude再認証が必要、認証設定は変更していない。全4provider/全要件の未達維持。

- 全体19111 exit0、`just check coverage` PASS（`tmp/model-boundary-full-check-formatted.log`）、native passed/15とline82.65%を親確認。後追加vendor比較の38320 hostcheck/63414 build、実probe46376もexit0（`tmp/acp-saved-settings-real/probe.log`）。保存設定比較の既知P1は独立reviewで解消、新規P0/P1なし。次にClaude既存環境の実応答を確認するtools限定acp_provider_probeを追加。初回4606 hostcheck PASS、親がError statusを本文一致で隠さないcompleted_responseを追加し45318で再検査。認証/設定変更はしない。

- model保持/trace空dir/全message比較の最終reviewは解消確認。保存vendor設定のendpoint/mode/thinking/permission/pending_permission比較不足をprobe限定helperで追加freeze。再探索候補(options/choices/available ids)だけは取得時に変わるため除外し理由を明記。19111全体checkは同handleでlive確認、終了後に最新probeのhostgate/実通信再検証が必要。商用sourceは45331の修正版から不変。監査表0011/0012の古い未取得記載を実Codex証拠へ同期、全達成にはしない。

- 実green probe45331 exit0（`tmp/acp-model-boundary-green/probe.log`）。同一SelectHistoryとNewChatのack時model保持、非空draftからの分離、元session再復元、stable snapshot/全message surface一致、token継続を確認。親RPC集計は明示prompt1+2のみ、Managed接続群0、失敗markerなし。全体check初回はprobeのfmt1箇所でexit1、rustfmt後 `just check coverage` を19111で `tmp/model-boundary-full-check-formatted.log` へ実行中。source修正版、全体gate完了はまだ未確定。

- model保持修正後42144 host-support-check/79660 bin build exit0。実green probe45331を `tmp/acp-model-boundary-green/probe.log` へ実行中、同handleをpollし終了前に再実行しない。現在sourceは修正版、旧53468全体gateは本host商用修正前の証拠として扱い、再検証が必要。

- 追加回帰の行数lint4475失敗を親がexercise_boundariesへ分割、41470 hostcheck/19226 build PASS。修正前実probe19033 exit1（`tmp/acp-model-boundary-red/probe.log`）でSelected model expected Some(gpt-6-astra)→actual Noneを直接再現。保全していたsession_lifecycle修正版を現在sourceへ復元済み（clear_model_selection不在）。同一回帰greenと最終全gateはこれから。

- 独立reviewのログ再利用P1を空trace dir必須化で修正し、74872 hostcheck/37432 bin build PASS。24411は既存trace dirを実prompt前にexit1拒否。比較強化probe5005 exit0（`tmp/acp-managed-real-20260905-4/probe.log`）。ただし同一session/NewChatでclear_model_selection→persistがmodelを消しReadyのdefault再選択で隠れる商用P1を親・独立reviewが確認。修正後sourceを `tmp/session-lifecycle-model-preserved.rs` に保全し、現在session_lifecycle.rsは回帰red確認のため変更前動作へ一時的に戻している。担当がack時model保持/NewChat draft分離のprobe回帰を追加中。実行後は保全した修正版を戻して全gate再検証が必須。

- representation修正後44418 host-support-check/53596通常bin build exit0。実Codex probe53978 exit0（`tmp/acp-managed-real-20260905-3/probe.log`）。親がRPC metadataを集計: Codex初回connection prompt1、Managed履歴選択connection prompt0、復元後connection prompt2（config適用1）、計3。probeがsurface同一表現比較・保存raw messages完全一致・履歴token継続・failure marker不在を確認。0012のCodex Managed無再送の直接証拠として追加、全provider/UI全体には拡大しない。release_gate_auditが計測2fileのfalse-positive/転送境界を独立read-only review中。

- 診断版23904は行数lint失敗、親が診断helperを分離して37092 hostcheck/93648 bin build exit0。実probe65988 exit1、error=None、expected raw message長[(1,92),(2,249)]とactual surface長[(1,92),(2,248)]。親がsurface/message.rsのMarkdown→plain_text再構成を確認し、raw contentとsurface bodyの比較が検証側の誤りと判明。担当へexpected surfaceの同一表現比較と、復元後保存raw messagesの完全一致を併用するtools限定修正を依頼。単純trimや長さ比較への弱化はしない。

- 実probe59047 exit1。`tmp/acp-managed-real-20260905-1/probe.log` はManaged SelectHistory復元不一致。親がrpc JSONLを集計しCodex initialize/sessionnew/sessionprompt各1件（初回）、次Codex initialize1件、Claude初回initialize/sessionnewと次initializeを確認。最初の実Codex promptは成功、履歴復元は失敗して以降の2prompt未到達。担当へhost error/message不一致を本文なし診断で切り分けるtools限定修正を依頼。現時点で商用バグかprobe誤りか未確定、0012未達。

- recorder helper化後の&&slice compile失敗を親が.args(args)へ修正。host-support-check87991 exit0（`tmp/acp-managed-recorder-check-verified.log`）、通常bin build84744 exit0。実Codex Managed履歴probeを59047で開始（`tmp/acp-managed-real-20260905-1/probe.log`、RPC metadataは同dir/rpc）。一時PATHのnpxだけをwrapperへ差替、既存認証は変更しない。終端前に再起動/同じprobeを重複実行しない。無再送/全要件の合格はまだ未確定。

- recorder/probe再検査53448 exit101（`tmp/acp-managed-recorder-check-fixed.log`）。型エラーは解消、既存strict lintの関数行数2件とcollapsible_if1件で停止。担当へ責務分割修正を依頼し、規則・範囲は維持。実通信はまだ実行しない。

- 0012用tools限定recorder/probeを親review。実npxへstdio透過転送しmethod/id/時刻/既知adapter名/PIDだけを記録する。Managed SelectHistoryの保存message一致・Ready待機・明示3prompt総数検証を追加中。初回host-support-check44648 exit101（`tmp/acp-managed-recorder-check.log`）、unused import/u128 JSON変換/借用寿命でcompile失敗。担当へ修正と不正JSON隠蔽禁止/最終failure marker検査/ack同時Ready取りこぼし対応を依頼。実provider未実行。商用source・依存変更なし、0012未達維持。
- OpenSpec validate1849 exit0。v0-1-0-scope.mdの0011/0013に残った古い完了表記を、現在の台帳と同じ未完了へ訂正済み。要件と基準は維持、diff --check PASS。

- 25171 exit0、更新egui通常binary build成功（`tmp/ime-egui-build-updated.log`）。第3実OS試行を `tmp/ime-egui-live-20260905-3` に隔離しCUAで起動。Raiseと座標clickを同一呼出しにすると操作結果は成功したが、実ログは意図したcomposer focusではなくSelectVendor(claude-code) seq1/ack1。以後AX noWindowsAvailable、キー操作はinactiveで拒否、pgrep exit1でapp終了を確認。stderr空。実IME・送信の証拠は得られておらず0004未完了を維持する。無根拠な同一試行は繰り返さない。
- 0012調査: 既存ACP history probeはManaged SelectHistoryを通らず、manual traceはoutbound session/promptを数えない。requirements_remaining_auditへtools限定の実stdio透過recorder/Managed検証probeを依頼。商用hook・mock・認証/prompt内容のログ・Cargo/外部実行は禁止し、親がfreeze後に検証する。全provider環境とKUC別repo許可は依然未回答。

- 53468 exit0、最新 `just check coverage` PASS（`tmp/unselected-provider-full-check-retry.log`）。core132/AST19/3host/E2E/macOS5/native15を通過し、親がnative passed/15を再確認。line coverage82.65%（8134行/未実行1411）、既存77%維持。0003の初回候補消失修正を達成へ戻す。実IME/全provider/scroll/公開は未完了。

- 全体58473 exit101、Floem既存empty registry回帰が空labelを期待して失敗。親が意図した未選択labelへ期待値だけ同期し、空active/空options/送信不可assertを維持。再実行 `just check coverage` は53468で `tmp/unselected-provider-full-check-retry.log` へ。coverageには初回未到達。

- 未選択provider修正freeze後、87435 exit0/core132 tests PASS（`tmp/unselected-provider-core.log`）。empty/unknown active候補保持・runtime/unknown候補除外・KUC typed選択回帰を含む。自動選択はしない。最新 `just check coverage` を58473で `tmp/unselected-provider-full-check.log` へ開始、終了前のCargo重複禁止。0003は全検証終了まで未完了を維持。

- 実host診断: tmp/ime-egui-live-20260905-2/tmp/harness-/last-session.jsonにactive_vendor_id空・available_vendor_ids=[claude-code,codex-cli]を親確認。VendorControlRenderModel::unknownがoptionsを空にするP1を確認。0003を未完了へ戻し、release_gate_auditへcontrols/関連tests限定修正を許可。自動選択/fallbackや旧native headerを根拠にしない。Cargoなし、保存freeze後に直接回帰と全体gate。

- bundle内binary配置の第2試行tmp/ime-egui-live-20260905-2でもCUA座標clickはnoWindowsAvailable。AX/画像/Tab keyは成功、実IME送信は未実施。起動wrapper外binaryだけが原因という仮説は支持されない。provider欄空/ack0更新4件の原因をrelease_gate_auditがread-only調査中。CUA Cmd+Q後getAXはtimeout、終了状態はprocess確認を優先。ログを保存し起動試行を無根拠に繰り返さない。

- egui build6107 exit0。CUAで検証appを起動、実process13509とevents.jsonlのhost更新を確認。CUA app取得/AX/画像は成功するが座標clickはnoWindowsAvailableで失敗。入力/送信は未実施。Close button操作後のgetAXはprocNotFound（操作後状態を次に確認）。wrapperがbundle外binaryをexecする構成を見直す。ログはtmp/ime-egui-live-20260905-1に保持し上書きしない。

- 実IME用binary build6107はlive（`tmp/ime-egui-build.log`）。起動用 `tmp/ime-egui-live-20260905-1/KCU IME egui.app` を作成、専用cwd/新規events.jsonl/stdout/stderrへ隔離する。まだ起動なし。CUA初期API読込済み、UI操作はcua_replだけを使用（キーはpressKey、paste/typeTextのUnicode注入を実IME証拠にしない）。build6107終端確認後、このapp pathをcua.getAppで開く。

- manual gate60204 exit0、trace追加後の3host strict lint/check/tests PASS（egui7/Floem53/GPUI6、`tmp/live-trace-host-check-final.log`）。親がshared trace testsへ日本語/改行JSONLとread-only実File書込失敗stickyの2回帰を追加し全hostで通過。実IME検証用egui通常binaryを6107で `tmp/ime-egui-build.log` へbuild開始。OS IME/実provider送信はまだ実施前。

- manual gate再実行51848 exit101、eguiはPASS、Floemのtrace追加でscheduler/dispatchのtoo_many_arguments3件（`tmp/live-trace-host-check-fixed.log`）。担当へtrace/on_effect観測責務をまとめる修正を依頼、規則無効化禁止。Cargo停止済み、GPUI gateは未到達。

- manual gate75856 exit101、egui strict lintでtraceのserde_json::json!内部unwrap/disallowed_methodsとcollapsible_ifを検出（`tmp/live-trace-host-check.log`）。担当へ明示Value/Map構築と条件式修正を依頼。規則無効化/依存追加は禁止、freeze後に親再実行。Cargoは停止済み、Floem/GPUIは未到達。

- trace修正freezeを親確認、send成功seqを返しupdate適用を継続する。writer失敗はsticky保持とKCU_TRACE_ERROR stderrで検証失敗を識別。3host gate `just manual-ui-check` を75856で開始（`tmp/live-trace-host-check.log`）。終了前のCargo重複禁止。依存52488の親/独立照合で新規直接更新候補なし、docs/dependency-update.mdの最新証拠を39967/59086/52488へ同期。

- outdated52488 exit0（全7manifest調査、`tmp/thinking-width-outdated.log`）。release_gate_auditが既知保留理由との差分をread-only照合中。trace親reviewで、記録エラーがsend成功seq/受信updateを捨てる問題を検出。担当へtrace失敗の明示・sticky保持と通常send/update維持へ修正を依頼。まだtrace compile前、Cargo実行なし。

- 最新依存再確認 `just outdated` は52488で実行中（`tmp/thinking-width-outdated.log`）。manifest/lock変更なし、終了前のCargo重複禁止。
- coverage59086 exit0、line82.64%（8129行/未実行1411）、既存77%基準維持（`tmp/thinking-width-coverage.log`）。manual-host trace担当は実装中のためmanual gateはfreeze後。workspace商用sourceは39967/59086から未変更、traceはtools限定。

- 0004実IME検証のため、requirements_remaining_auditへmanual-host3種とtools/manual-host-trace共通module限定で任意JSONLログを実装許可。既存Managed send/try_recvを維持し、実seq/ack/draft/messagesを記録する。KCU_TRACE_PATHは絶対path/create_new、fixture代替送信なし、core/adapter/host-support/依存/画像基準変更禁止。coverage59086実行中のためCargo禁止、保存freeze後に親検証。OS設定のread-only確認ではSelectedにGoogle日本語の記録あり、Enabled一覧とは異なるため実IME稼働証拠とは扱わない。

- OpenSpec validate90843 exit0、diff --checkもPASS。coverage59086は同handleでlive/依存compile中。0004実OS IMEの既存machine-readable導線は未整備。requirements_remaining_auditがmanual-host本来のManaged send/try_recvを保持した検証ログ案をread-only設計中。fixture Submit代替・商用core/adapter/host-supportへのtest hookは採用しない。まだ編集/アプリ起動なし。

- 39967 exit0、最新 `just check` 全体PASS（`tmp/thinking-width-full-check.log`）。親がnative summary passed/15を再確認。0005/0006限定独立reviewで新規P0/P1なし、両要件を達成へ更新。既存77%基準の `just coverage` を59086で開始（`tmp/thinking-width-coverage.log`）、未終端・Cargo重複禁止。実provider/IME/scroll/公開の未完了は維持。

- thinking-only幅をcontent/frame.rsの補助表示幅算出で修正。64853 exit0、layout15 tests PASS（`tmp/layout-requirements-fixed.log`）、usage ring/長短本文/3状態除外の全追加回帰を含む。最新 `just check` を39967で `tmp/thinking-width-full-check.log` へ実行中。strict lint/core129/AST19/共有host/3manual host/E2E/macOS5画像まで通過し、Linux15画像へ進行。終了前のCargo重複禁止。画像基準は85035後から変更していない。

- 追加layout回帰79661 exit101、usage ring実画素/長短agent本文幅の2件PASS、thinking/execution/processing幅除外1件FAIL（`tmp/layout-requirements-regression.log`）。空本文thinkingのbody_width fallbackが残存。requirements_remaining_auditへcontent/frame.rs限定の内容幅修正を許可、商用変更後の再検証は親が実行する。基準は緩和しない。

- native必須gate85035 exit0、親がsummary status=passed/15画像・基準hashを確認（`tmp/native-message-padding-gate.log`、`target/native-harness-screenshots/summary.json`）。0005/0006の残り直接回帰をrequirements_remaining_auditがlayout tests限定で追加中、商用コード/基準変更/Cargoは禁止。freeze後に親検証する。

- native採取85290 exit0、15case全exit0・IHDR寸法一致・同scenario3host SHA256一致を親が確認。standard/narrow/tall/thinking実画像を確認し、本文padding差分をaarch64基準12hashへ反映（empty3hashは不変）。必須 `just harness-native-linux-matrix-check` を85035で実行中（`tmp/native-message-padding-gate.log`）。終端取得までCargo/source変更を重複しない。

- 0005/0006追加照合: control下揃え/枠画素/狭幅/送信左usage位置/実本文上下中央/user右寄せ/agent=composer幅は現126 tests内で証明済み。円グラフ画素、agent長短本文で幅不変、thinking/execution/processing全状態の幅除外は直接回帰を補う。native採取中はsourceを固定する。
- 0012 read-only追跡: Managed SelectHistoryのboundaryはprovider probeを起動する。bind限定回帰を全経路無送信証拠にしない。実認証providerのrequest記録と合わせたSelectHistory検証が残る（test都合の商用hook/Mock/source文字列検査は導入しない）。

- 29316 exit0、本文padding/model復元修正後 `just manual-ui-check host-e2e harness-screenshot-check` PASS（`tmp/message-model-host-matrix.log`）。macOS5画像の更新後基準も通過。Linux native15caseを既存artifact-only採取経路で85290へ開始（`tmp/native-message-padding-capture.log` / `tmp/native-message-padding-review`）。採取の成功と基準gateを区別する。

- 再開時29316のliveをpollで確認。本文padding/model復元修正後の `just manual-ui-check host-e2e harness-screenshot-check` を `tmp/message-model-host-matrix.log` へ実行中。終端取得前にCargo/native採取を重複開始しない。
- model復元修正後 `just host-support-check` 37752 exit0、strict Clippyと73 tests PASS（`tmp/restored-model-host-check-final.log`）。保存model保持・欠落/失敗時の無断fallback禁止の回帰を含むが、実provider適用成功と0012全Managed経路の無再送証拠には拡大しない。
- macOSのstandard/narrow/tall/thinking画像を親が確認、全5画像の寸法も確認。本文padding変更による4hashを基準へ反映、emptyは一致のため変更なし。基準gateの結果は29316待ち。Linux native基準は未変更、採取・レビュー・必須gateはこれから。

- standard-chat採取72663はexit1（旧hashとの不一致）、実画像2880x1800を親が確認。新hash de808422926884553aebaec61a5ca465bb86a82e5f31d159a82f9634a87cfed0、まだ基準未更新。残4scenarioを`tmp/message-padding-review`へ採取開始し、各returncodeをcapture-results.jsonへ保存する。採取wrapper成功と基準gate合格は区別する。
- host-support-check38249 exit101、katan_poll::handle_katan_eventがClippy too_many_lines 34/30。担当へCatalogReady処理のhelper分離を依頼（このfileのみ/Cargo禁止）。model新unit実行はlint通過後。画像runnerとhost-support依存は分離済み。
- 採取再実行handleは72663、pollでlive。次turnは同handleの終了取得を優先し、Cargo/cleanupを重複しない。画像未確認・baseline未変更のまま。hostモデル修正はrelease_gate_auditのfreeze後に親review/host gateを行う。
- standard-chat採取29891はexit101、release buildでNo space left on device（`tmp/message-padding-capture.log`）。結果画像/基準判定には未到達。全Cargo終了確認後、task-owned `target/llvm-cov-target`をcargo clean dry-run→39317 exit0で4.1GiB整理。ログ/画像/ソースは維持、現在空き18GiB。同採取を`tmp/message-padding-capture-retry.log`へ再開する。モデル担当は明示model成功後のMissing解除の親指摘を修正中、host testはまだ未実行。
- standard-chat採取の実行handleは29891、20秒pollでlive確認。次turnは同handleの終端を確認し、実行中にCargoを重複起動しない。スクショrunnerはcore+egui依存のみでhost-support担当と分離している。model修正はrelease_gate_auditが実行中、保存freeze前にhost-supportのcompileは開始しない。
- 本文padding修正後core全unit41805 exit0、126 tests PASS（`tmp/message-padding-core.log`）。モデル担当はhost-support修正中で未freeze。独立したin-process standard-chat画像を旧baseline維持のまま`tmp/message-padding-review`へ採取開始（`tmp/message-padding-capture.log`）。採取/基準差分/実gateを区別し、基準はまだ更新しない。
- 本文padding修正後85138 exit0/layout12 tests PASS。親がvisual回帰をさらに実thread上端とbody hit位置の差分まで測る形へ補強し、97451 exit0/3 tests PASS（`tmp/message-padding-verified.log`）。本文の実上下余白が共通paddingに一致することを確認。現在core全unitを`tmp/message-padding-core.log`へ実行開始。画像基準/全体gateはまだ再検証前。
- モデル復元実装の親reviewで、不正な明示選択でもpending解除・欠落/適用失敗後のReadyでdefault復帰・通常Ready選択更新の抑止を検出。release_gate_audit担当が失敗保持/選択成功時のみ解除/復元中だけ選択保持へ修正中。まだmodel回帰の再実行前で、0011未完了を維持する。
- 本文境界へ修正した回帰46200 exit101、実本文上20px/下4pxを再現（`tmp/layout-body-regression.log`、他2tests PASS）。requirements_remaining_audit担当に公開KUCによる共通上下配置の商用修正を許可し、content/frame・関連layout回帰・conformance内の旧status座標依存だけを担当させた。別repo/内部描画コピー/Cargoは禁止。画像基準はまだ更新しない。修正後に本文/複数行/Markdown回帰と画像gateを再検証する。
- model消失88045の再現後、release_gate_audit担当がprovider_managerと必要なprovider_integration範囲で保存model保持/候補確認後の実適用/適用前送信禁止/無断fallback防止/古いpending漏れ防止を実装中。親のmodel回帰を維持し、Cargoは禁止。両担当は実行中で書込責務を分離。現在の全体PASS証拠18167はこの新修正前のものとして扱い、保存freeze後に親が回帰・品質gateを再実行する。
- 親が履歴回帰をprobeなしのstate SelectHistory→bind_session_identityへ修正。88045 exit101、保存model Some("ollama:test-model")がbind後Noneになる不具合を直接再現（`tmp/history-model-regression.log`）。provider通信は起動していない。release_gate_audit担当は再接続時のmodel適用/fallback禁止を含む修正設計をread-only調査中。layout担当は本文nodeを測る回帰へ修正中。両担当へfollowup_taskで実行を開始済み、Cargoは現在なし。
- layout visual22164 exit101: control下揃え/SVG枠pixelの2回帰PASS、status nodeの12px固定回帰FAIL。ただし親レビューで本文ではなくstatusを測る誤りと未指定12px期待値を検出したため、この失敗だけを実装欠陥の根拠にしない。担当が実本文の上下余白へ修正中。全Cargo終了、freeze後に再実行する。
- 0012担当の新規Managed SelectHistoryテストはbegin_session_boundary→probeで外部providerを起動するため、親が実行前に差し止めた。実保存→state復元→Manager bind_session_identityだけのmodel保持回帰へ変更を依頼。bindのclear_model_selectionで保存modelを消す現sourceを確認し、0011を未完了へ戻した。0012無送信の全経路証拠とは混同しない。商用変更なし・新規回帰はまだ未検証。
- 0002/0003/0011の番号別監査を親がsource/18167ログで再確認し[x]へ更新。0005/0006はcontrol下揃え/二重枠/文字縦位置/paddingの直接回帰が不足し、requirements_remaining_audit担当がruntime/tests/layout配下へ追加中（Cargo禁止、商用修正は禁止）。0006はsource上のpadding不足を疑うが、実描画での失敗確認前は確定扱いしない。0012はrelease_gate_audit担当が実Managed offline復元回帰をdispatch/testsに追加中。submit observer注入は行わず、送信回数計測済みとは表記しない。両担当freeze後に親が回帰を実行する。
- CI OS分離を保存freeze後に親確認。macOSの全品質/coverage/package検査とubuntu-24.04-armのnative gateを必須needsで結合、全checkoutをmerge_commit_sha/github.shaへ固定。公開時package再buildのOSもmacos-latestへ揃えた。親のjust --dry-run release-check/diffcheckはexit0、Rubyで固定ref/needs/runnerと集約shellのsuccess/failure/cancelled/skipped全16組合せを実行してexit0。既存gate内容/77%基準/画像基準/Rustソースに変更なし。実CIは未実施、ローカル全検査の実行証拠は直前18167 exit0を維持する。
- release-check18167はexit0完了（`tmp/pure-kuc-release-check.log`）。workspace193/AST19/共有host64/manual各host/E2E/native15/coverage82.28%/package/先頭crate publish dry-run/全5crate同版未公開確認までPASS。親がnative summary passed/15を再確認。これ以降18167のpollや同一検査の無根拠な再実行は不要。未達機能/実provider/CI/公開は別条件として残る。
- CI runner監査でrelease/preflightのubuntu-latest単一jobとmacOS画像基準・Linux/arm64 native gateの矛盾を検出。release_gate_audit担当がworkflow/Justfile/docsのみ修正中。既存local release-check/checkの検証内容は維持し、各OSの成功を公開の必須依存にする。Rust/画像基準は変更禁止。修正保存後に親が構造とdry-runを確認する。
- 実provider検証環境（Claude未認証/Copilotなし/Ollamaモデルなし）の接続先・モデル・認証導入方針をasync質問した。未回答は環境導入や認証の承認と扱わない。既存KUC別repo編集の承認も未回答のまま。
- release_gate_auditの最終read-only再レビューで公開順序/公開省略/部分公開後のソース混入の3指摘は解消、新規公開阻害なし。親の一時実Git検証と併せてworkflow修正を確認済み。release-check18167は20秒pollでlive、まだ終端未取得。次turnは同handleを確認し、Cargoの重複起動を避ける。全OpenSpec要件/CI/commit/push/公開は未完了。
- 公開順序の再レビューで部分公開後の別ソース混入を検出し、最終順序をsource固定tag→全5crate公開/可視化→GitHub Releaseへ修正。既存tag^{commit}とHEAD不一致はpublish前に失敗する。省略inputは除去したまま。実workflowから抽出したtag scriptを一時実Git repo/ローカルbare remoteで実行し、新規tag/同HEAD再実行の成功と別HEAD再実行拒否を確認exit0。YAML順序検査/diff --checkもexit0、実GitHub Actions/公開は未実施。下記のtag後置案はこの修正で置換した。
- 公開手順のread-only監査で、crate公開前にtag/GitHub Releaseを作る経路とpublish_crates=falseによる公開省略を検出。release.ymlを全5crate公開・可視化確認→tag→GitHub Releaseに変更し、省略inputを除去。docs/release.md同期、Ruby YAML構造検査（公開step無条件・順序・省略input不在）とdiff --checkはexit0。再レビュー待ち、CI/実公開は未実施。release-check18167は同handleでlive確認し、依存build継続。今回の変更はworkflow/docsのみでRust差分なし。
- OpenSpec status54459/apply instructions36602はexit0。statusのisCompleteはartifact整備状態であり実装完了ではない。applyは34タスク中24完了/10未完了。既存scope0001〜0017と追加changeのproposal/design/2spec/tasksを再読し、scroll上限欠落を現sourceでも確認。全要件の完了判定は保留する。
- release-check18167はliveでworkspace検査まで進行、結果未取得。実provider環境を再確認: codex/claude/ollama実行ファイルあり、copilotなし、localhost Ollama /api/tags models=[]。`claude auth status`1783 exit1でloggedIn=false/authMethod=none。全4provider実接続を満たす環境ではなく、fixture/gate合格で代替しない。
- release-check実行handleは18167（`tmp/pure-kuc-release-check.log`）。次turnは同handleの終了確認から再開する。現在はビルド中で公開dry-runは未到達。commit/push/PR/実公開はまだ行っていない。無関係`.serena/project.yml`と各`.DS_Store`を引き続き対象外に保つ。
- coverage9021 exit0、line82.28%（8080行中1432未実行）、既存77%閾値/対象を維持（`tmp/pure-kuc-current-coverage.log`）。直近標準UI/添付API/出力ボタン/Linux gateのread-only再レビューは新規P0/P1なし。既知scroll/全provider環境の未達は別途残る。`just VERSION=v0.1.0 release-check`を`tmp/pure-kuc-release-check.log`へ開始、line-tables-only/JOBS2/INCREMENTAL0を引き継ぐ。公開dry-runまでの検証であり実公開ではない。
- coverage9021はliveで依存compile継続（終端未取得）。内部依存version検証は0.1.0でexit0。GitHub open PRは[]、Release一覧も空を再確認。KUC sparse registry92970では最新0.3.6/yanked=falseのままで、scroll API境界を解消する新公開版は出ていない。active change tasksのcheck/ASTだけを現証拠に基づき[x]へ更新、coverage/全要件/リリースは未完了。
- coverage実行handleは9021。次turnは同handleをpollし、終了前にCargoやtarget cleanupを重複開始しない。coverage結果はまだ未取得。
- 最新差分`just check`99379 exit0（`tmp/standard-scope-full-check.log`）。fmt/strict lint/workspace193/AST19/共有host64/3manual host/E2E/in-process5画像/native15画像を含む。native summary passed/15を親確認。coverage前にtask-owned target/debugをcargo clean --profile devでdry-run33291→実行16058 exit0、40.1GiB回収。ソース/tmpログ/target画像/release runnerは保持。coverageを`tmp/pure-kuc-current-coverage.log`へ開始（CARGO_BUILD_JOBS=2、DEV_DEBUG=line-tables-only、INCREMENTAL=0、既存77%基準/対象維持）。
- 全体check63606は共有dispatch回帰の整形後に33/30行となりhost-support Clippyでexit101。添付データ準備をmarkdown_attachment helperへ分離し、検証内容を維持。99379で全体`just check`再実行中（`tmp/standard-scope-full-check.log`）。終了前にCargo再起動/target cleanupを行わない。ディスク空き4.4GiB、coverage前にtask-owned debug cacheの整理を検討する。
- 全体check初回は共有dispatch回帰のload_host改名後1assert整形差分でexit1。host-support cargo fmt後、63606で`just check`を再実行中（`tmp/standard-scope-full-check.log`）。次turnは63606の終端確認から再開し、同Cargoを重複開始しない。
- 更新済みnative基準の必須gate65317 exit0、`target/native-harness-screenshots/summary.json` status=passed/15画像を確認。Linux画素回帰もPASS。最新標準scope全体の`just check`を`tmp/standard-scope-full-check.log`へ開始（root共通target）。残るscroll境界/実provider/coverage77/release公開条件は未完了のまま。
- native artifact採取56792 exit0、全15caseのexit0・IHDR要求寸法一致・同scenario3host SHA256完全一致を親が検証。全5scenarioの画像を確認し、添付除去/ボタン128幅を反映したaarch64基準15hashを更新。`just harness-native-linux-matrix-check`65317を`tmp/native-standard-scope-gate.log`へ実行中（Linux画素回帰必須化後の初回全gate）。採取成功とgate成功を区別し、次turnは65317をpollする。
- macOS5画面41931は採取終了（各旧baseline不一致exit1）。親が全PNGを確認し、添付アイコン除去・128×32出力ボタン・usage/sendを確認、IHDRがviewport×2であることを検査して5hash更新。Linuxは公式artifact-only（native-baseline引数なし）で15caseを56792で採取中、`tmp/native-standard-scope-review.log`と同dir/review-results.jsonへ記録。採取成功は0015 gate PASSではない。終了後全caseの寸法/host間hash/画像を確認し、native基準採否とmandatory gate実行を行う。次turnは56792をpollする。
- `just host-support-check manual-ui-check`57272 exit0（`tmp/standard-scope-host-check.log`）。共有host64testsとegui/Floem/GPUIの各strict lint/build/testを通過。標準scope変更後のmacOS5画面を`tmp/standard-scope-images`へ採取開始、旧baselineチェックは維持する。採取画像を確認してから基準更新/native15caseへ進む。
- 両担当freeze後、親がprogrammatic添付の実dispatch回帰をreview（helper名shadowをload_hostへ修正）。`just host-support-check manual-ui-check`57272を`tmp/standard-scope-host-check.log`へ実行中。host-support strict lintと64testsはPASS（添付追加→新host復元→削除→新host復元を含む）。現在manual host gates継続中。次turnは57272の終端を取得し、未完了中に同Cargoを再開始しない。
- 標準scope修正後の初回54613はcore旧Attach前提2testでexit101。親がAttach不在assertへ置換し、usage/send/control配置検査を維持。再実行`just lint unit-test`27046 exit0（`tmp/standard-scope-unit.log`）。削除済みFloem picker専用helperを参照する旧テストも除去し、同等以上のprogrammatic AddAttachment/RemoveAttachment契約を共有worker dispatchで検証する新回帰を担当が追加中。親のmanual-ui-check/host-support-checkは担当freeze後に実行する。
- slash担当は保存freeze。composer.rsのlauncher投影を除去、tree fixtureのvisible=false回避を除去し、実visible状態でもquery/actionが標準treeに無い回帰を追加。親reviewで不要Result戻り値だけ簡素化。添付担当はfooter/host adapterと回帰を編集中（agent standard_attachment_scope running）。次turnは同担当freeze確認後、親review・fmt/lint/unit・manual-ui-checkを実行する。全Cargoは停止している。
- 幅128修正後macOS`just fmt-check lint unit-test`72268 exit0/192 tests PASS（`tmp/output-width-check.log`）、Linux38591もPASS。全Cargo終了。native gateはrun-checks.shでLinux画素回帰を必須実行してからXvfb/matrixへ進むよう更新済み、shell構文検証済み（実gate未実行）。
- 追加監査でscope§3除外のOS file picker/DnDとslash launcherが標準経路に残ることを親も確認し採用。standard_attachment_scope担当はfooter/Floem main/egui appのOS添付経路を除去、requirements_remaining_audit担当はcomposer.rs/tree testsの標準slash投影を除去中。programmatic Markdown入力/core command registry契約は保持する。両担当freeze後に親review・回帰実行し、画像はこの変更も含め再採取する。
- native狭幅でOpen preview文字の横はみ出しを発見。採取41878は意図停止しexit143取得（5caseまでの途中採取でありgate PASSではない）。Linux/aarch64の同画素回帰58387 exit101でx=96がwidth96の外に描かれることを再現。出力actionを128×32へ修正、Linux38591 exit0で全7ラベル画素包含を確認（`tmp/linux-output-label-{regression,fixed}.log`）。macOS fmt/lint/unitを`tmp/output-width-check.log`へ実行中。前の128未満幅の3会話画像baselineは最終証拠に使わず、修正後のmacOS/native採取・採否・全gateが必要。
- OpenSpec validate3445 exit0。Linux必須native gateへ同画素回帰を追加する作業をmatrix担当へ依頼（shellのみ、Cargo禁止）。requirements_remaining_audit担当は既知scroll/実provider環境以外の現在コードをread-only監査中。
- native matrixの例外終了時にsummaryがrunningのまま残る不具合を修正。failed・診断・成功済みimagesを保存して元の例外を再送出する。親がhelper分離後をreviewし、Python unit5件PASSを再確認。native採取41878は継続中（empty-chat egui/Floemは旧hash一致）。修正後matrix本体のnative gate実行は採取終了・基準採否判断後。
- 更新済みin-process基準で`just harness-screenshot-matrix-check`63062 exit0（`tmp/output-button-screenshot-gate.log`）。3adapter34tests・runner12testsと既存5画面hashを通過。native15caseの採取41878を`tmp/native-output-button-review.log`へ開始、`tmp/native-output-button-review/review-results.json`は採取結果だけでgate合格ではない。native旧hashを維持したまま全caseを採取し、変更後のhost間一致と画像を確認する。次turnは41878の終了を確認し、実行中に同Cargoを再起動しない。
- 残4画面採取53925は終了（各caseは旧hash不一致exit1、wrapper exit0）。親が全5画面を確認し、純KUC移行後のheader/composer/controlと出力ボタン32pxを基準へ採用。5 JSONの変更はhash各1行のみ、全PNGのIHDRは要求viewport×2に一致。`just harness-screenshot-matrix-check`63062を`tmp/output-button-screenshot-gate.log`へ実行中。native baselineは未変更、続いて15caseを採取する。
- tall-chat修正後採取52945は旧baseline不一致でexit1（`tmp/output-button-capture.log`）、新hash624108f9c7b1eed6db0508a1af2b25bf1998a34a43b1dff73d8e97c99839578e。画像`tmp/output-button-review/tall-chat.png`を親が確認し、Open preview/Apply diffの文字が背景内に収まることを確認。自動画素回帰の裏付けでありprovider E2Eではない。fmt/lint/unit36300は192件PASS、全Cargo作業終了。次工程は残4画面とnative15caseを修正後に採取し、意図した移行差分としてbaseline採否を判断する。baselineはまだ変更していない。
- 出力ボタン/GPUI操作変更後の`just fmt-check lint unit-test`36300 exit0（`tmp/output-button-gpui-check.log`）。全7出力actionラベルの画素包含回帰とGPUI履歴/Stop回帰を含む。旧baseline未更新のままtall-chatを`tmp/output-button-review`へ再採取し、修正後画像を確認する。
- screenshot88556はexit1（empty-chat旧hash不一致）。全5case採取28326も各case exit1で、`tmp/pure-kuc-baseline-review/results.json`は採取結果のみ。baselineは未更新。画像の出力ボタン文字は高さ20pxの背景外(y=20)へ描かれることを新しい実KUC画素差分unitで再現86408 exit101。出力action高さ32pxへ修正し84715 exit0。全7actionラベルへ回帰を拡張、最新fmt/lint/unitを`tmp/output-button-gpui-check.log`へ実行中。
- GPUIの実KUC action座標→simulate_click→ToggleHistory/SelectHistory/Stop callback回帰を追加。初回19732はtest helperのVisualTestContext借用型不一致で失敗、親修正後61740 exit0/1 test PASS。provider E2Eとは区別する。出力高さ変更によりnative/in-process画像は再採取・検証が必要。KUC scroll別repo承認は未回答。
- screenshot gateのhandleは88556（`tmp/pure-kuc-screenshot-check.log`）、root共通targetを使用。次turnは同handleをpollし、終了前に別cargoやbaseline更新を開始しない。
- IME/操作/依存更新後`just fmt-check lint unit-test`33594 exit0、workspace190 tests PASS（ACP17/core121/egui13/Floem10/GPUI10/AST19）、strictclippy PASS（`tmp/ime-actions-workspace-check.log`）。GPUIはvirtual text/marked/selected、explicit virtual rangeの継続preedit/commitを実TestWindowで検証。egui/Floemのhistory/Stop実pointer回帰79796もexit0。全担当freeze。既存in-process screenshot gateを`tmp/pure-kuc-screenshot-check.log`へ開始し、純KUC移行後の旧baseline照合を進める。KUC scroll API別repo承認は未回答のまま。
- egui/Floem unitの実行handleは79796（`tmp/adapter-history-stop-tests.log`）、現在build中。担当host_actorは完了freeze、GPUIのvirtual→base変換のみkuc_boundaryが編集中。次turnは79796の終端確認を優先し、同時Cargoを開始しない。
- GPUI IME test46523 exit101（9PASS/1FAIL）。失敗は指定1文字後caretを先頭glyph内部と誤期待した回帰で、first glyph右端/次glyph左端の正確な位置へ修正する。別途virtual document上のexplicit rangeをbaseへ未変換でcommit/継続preeditする欠陥を親/担当で確認、kuc_boundaryが共用変換と回帰を修正中。egui/Floem history/Stop回帰は保存freeze済み、親が両crate unitを`tmp/adapter-history-stop-tests.log`へ実行中。次turnはそのhandleの終了を取得し、GPUI担当freeze後に再検証。
- 更新後`just outdated`30552 exit0（`tmp/dependency-outdated-after-update.log`）。base64/redox_syscallの更新候補は解消。workspaceのindexmap1.9.3/schemars0.9.0は`cargo tree --locked --target all --all-features -i`でも使用経路なし、serde_withの互換用optional依存を含むlock表示と実使用を区別する。Windows/raw-window-handle別系列等の残候補は無条件major置換しない。全Cargo停止、IME/操作テスト両担当が実装中、freeze後にunit/lintを実行する。
- base64更新後ACP単独test80886 exit0/17 PASS（`tmp/acp-base64-update-test.log`、warnings-as-errors/locked）。全Cargo停止。kuc_boundary担当はGPUI virtual preedit documentとmarked/selected range契約を実InputHandler回帰で修正中、host_actor担当はegui/Floemのhistory/Stop実pointer経路unitを追加中。担当範囲は分離。両担当freeze前に全workspace gateは開始しない。KUC別repo承認は未回答。
- base64更新45071 exit0、全7lockでkatana-acp-client→base640.23.1をTOMLとして検証。GPUI lockの不要windows-*4packageがresolverにより除去された。compile/testはIME担当freeze待ちで未実施。次turnは依存差分reviewとIME完了通知を確認してから個別gateを開始する。
- `just outdated`80021 exit0で全7構成を再調査（`tmp/dependency-outdated-current.log`）。4lockのredox_syscall0.9.3→0.9.4を20400 exit0で更新。base64最新版0.23.1のAPI/MSRVをregistry sourceで親も確認し、ACP manifestを0.23へ更新。7manifestへlock再解決45071を実行中（Cargo updateのみ、build/testなし）。次turnは45071の終了取得・lock差分確認から開始。IME担当はまだ作業中のためcompile gateは保存freeze後に実行する。KUC別repo承認は未回答のまま。
- scroll修正の境界: sparse registryでKUC最新公開0.3.6を再確認。実描画のmeasured_scroll_node_height/text_contextはpub(super)、公開hit APIはcontent_heightを入力に使い正確なextent APIではないことを親/担当で確認。KUCに実描画と同じ公開extent APIを追加・公開してKCUで採用する案について、別repo編集の明示承認をasync質問中（未回答は許可ではない）。回答前はKUC編集/Issue作成を行わず、KCU内IME修正・検証を継続する。全Cargo停止、kuc_boundary担当保存待ち。
- 読取監査で0004のIME caret単位不整合を確認（Floem cursor=Noneでchars count、GPUIでUTF16 count、KUC契約はUTF8 byte）。親も実source確認。check31211を意図停止exit143、cargoなし確認。host_actorは共通scroll extent/clamp/回帰、kuc_boundaryはFloem/GPUI IME caret/selected rangeとGPUI held CmdEnter回帰を実装中。担当freeze完了までCargoを開始しない。3adapterのhistory/Stop実click受入はまだ未検証。
- 追加読取監査でscroll上限欠落を確認。runtime/input/scroll.rsはu32飽和のみ、tree/frame.rsはcontent_height=viewportheight固定、KUC canvasはraw offsetで描画するため末尾超過でthreadが空白化する。親も該当sourceを確認。全check31211終了後、実content extentと最大offset clamp、長文末尾/先頭復帰/短文不動の回帰を実装する。未対応P1として残す。
- 全check40740はGPUI buildのNo space left on deviceでexit101。egui/Floemのhost gateまではPASS。停止確認後、今回生成した4host専用targetだけをcargo cleanし約11.5GiBを整理（各clean exit0）、root target・ログ・画像・ソースは維持。`CARGO_TARGET_DIR=/Users/hiroyuki_furuno/works/private/katana-chat-ui/target just check`を31211で再開（`tmp/openspec-full-check-shared-target.log`）。次turnは31211をpollし、終了前に別cargoを始めない。kuc_boundaryは入力契約、host_actorはscroll契約を読み取り監査中。
- Actions依存を公式APIで親も確認（checkout/upload-artifact latest=v7.0.1）。3workflowのcheckout8箇所とupload1箇所を@v7へ更新。既存ref/fetch-depth/artifact条件を維持、3YAML parseとdiff-check PASS。最新runner条件と採否は`docs/dependency-update.md`へ記録。CI実行は未確認。全check40740はまだhost側build中、次turnは同handleをpollしてから次cargoを開始する。
- 再開した全体checkのhandleは40740（`tmp/openspec-full-check-activity-fixed.log`）。現在workspace test中、未終了。host_actor担当はworkflow usesの最新公式release/runner移行条件を読み取り調査中。次turnは40740の終了取得とActions調査の統合から再開する。
- 0017のACP ToolKind情報損失を修正。Read/Edit/Delete/Move/Search/Fetch/Execute/Think/Otherをtyped activityへ写像しbridgeまで保持、kindなしupdateはUnchanged。Plan/nontextは既存Processingを維持。`just host-support-check`14767 exit0、strictclippyと63 tests PASS（`tmp/acp-activity-host-check.log`）。新規回帰は全種写像・update有無・Plan/nontext・7種session反映を検証。実provider E2E達成とは区別する。全体checkを`tmp/openspec-full-check-activity-fixed.log`へ再開、次turnは実行handleをpollする。
- README/designのACP通信未提供という古い記述を現transport実装・未検証境界へ修正。純KUC adapter3crateもREADMEへ明記。kuc_boundary担当は0017修正を実装中、親はRustを編集せずCargo停止を維持している。各providerが実行しないtool状態を捏造する必要はなく、標準ACP ToolKind情報の損失を解消する。保存完了通知後に親reviewとhost-support-checkを行う。
- 全check53722はfmt/strict lint、workspace183 tests、追加AST19 testsまでPASS。0017の欠落（ACP ToolKind→title→Processing固定）を最新実装で確認したため、host-support-check中に親が意図停止しexit143取得、cargo停止を確認。全check合格ではない。kuc_boundary担当がtyped activity写像・bridge伝播・回帰を修正中（Cargo禁止）。親は保存完了後に差分review、host-support-check、最終全体gateを実行する。log=`tmp/openspec-full-check-current.log`。
- proposalの旧「公開対象外」を最新リリース指示へ統一し、OpenSpec validate19718 exit0、diff-check PASS。native gateの独立読取レビューは最新ファイル再確認後に指摘0件。全体check53722はコンパイル継続中（終了未取得）。kuc_boundary担当が0017のACP Activity payload破棄を読み取り監査中。次turnはcheck53722をpollし、監査結果を最新コードで確認する。
- native必須gate再実行6980はexit0。`target/native-harness-screenshots/summary.json`はstatus=passed、aarch64の15caseを記録。Python契約検査追加後の再検証を完了した。全体`just check`53722を`tmp/openspec-full-check-current.log`へ出力して実行中。次cargoは同handleの終了を取得してから開始する。
- 3host native必須gateを`just check`へ追加、CIは実測と同じ`ubuntu-24.04-arm`で独立job・失敗時もartifact保存。基準は`linux/baselines/aarch64.json`にhost/requestごと管理。親reviewでrunner二重WM起動、不要release profile、request path固定、空scenario成功を修正した。
- `just harness-native-linux-matrix-check`58635 exit0、summaryの15case検証済みを確認。実captureへ誤hashを渡した35771は期待どおりexit1（`tmp/native-baseline-negative.log`）。baselineを減らす/host欠落/不正hash/viewport不正のunit4件PASS。通常gate/CI組込みは実装済みだがCIの実行・required check設定は未確認。新Python検証の追加後にgate再実行が必要。
- 再検証86575 exit0。最新visual修正のstrict lintとhost E2Eが通過（`tmp/visual-fixes-lint-e2e.log`）。全cargo停止。次工程は上記3host画像の必須gate化と残OpenSpec証拠の充足であり、リリース完了にはしていない。
- visual修正後lint88373はGPUI error reportのnested ifだけでexit101。意味を変えずlet chainへ直し`just lint host-e2e`86575で再実行中（`tmp/visual-fixes-lint-e2e.log`）。次turnは同handleの終了取得から再開し、終了前に次cargoを開始しない。
- 正式Mesa25/Openbox環境の3host×5scenario65397 exit0。全15PNGのIHDR寸法がrequestに一致し、同scenarioの3host SHA256が完全一致（`tmp/three-host-matrix-evidence.json`）。standard/thinking/narrow等を目視しOutput重複とheader欠落の解消を確認。WM wrapperの子exit17伝播も実測。baseline採用/通常check・CI組込み/4provider実E2Eの証拠とは区別する。
- 次工程: native baselineをhost/request/architecture付きで管理し、既存in-process screenshot gateを残して新しい3host必須gateを追加する。実測はlinux/aarch64のため、未検証amd64で同hashを仮定しない。全cargo停止、全担当freeze。
- Openbox付きGPUI通常採取48704 exit0。起動後リサイズなしでKUC画面が表示され、empty-chat hashfc1ba602…がegui/Floemと一致した。初回animation要求の試行は撤去済み。Mesa25+Openboxを正式Dockerfile/仮想display wrapperに反映しimage55052 exit0。
- 正式環境で3host×既存5scenarioの採取65397を実行中（`tmp/three-host-matrix.log`）。全cargoはこのDockerのみ。終了後15PNGの寸法・描画・同scenario比較を行う。baseline採用/required CI gate/最終品質gateは別途未完了。
- GPUI初回request_animation_frame追加2250もexit1で効果なし、追加を撤去した。WM不在の影響を確認するOpenbox比較image96158はbuild exit0。通常KUC再採取はegui/Floem修正後matrix59530（`tmp/native-matrix-fixed.log`）の終了後に実行する。全cargoはmatrixのみ、同handleをpollしてから次cargoへ進む。
- Mesa25通常GPUI採取38482は初期frame単色でexit1。一方、同じ通常KUC binaryへ実X11 resizeを与える90749はexit0/colors215で描画成功（`tmp/gpui-kuc-mesa25-resize.log`、`tmp/native-linux/gpui/probe.png`）。Mesa25だけで初期frame問題が全解消したとはしない。初回描画後の再描画通知/表示初期化を次に検証する。全cargo停止。
- Output/Header修正後`just fmt unit-test`26214 exit0/183 PASS（core121を含む）。Mesa25正式imageで通常GPUI sourceを再build・採取する38482を実行中（`tmp/native-gpui-mesa25.log`）。全cargoはこのDockerのみ。同handle終了取得後に3host全scenarioを再採取し、修正後画像を検査する。
- 正式Docker Mesa25 image69234 exit0。Output重複修正はColumn/Text分離と430/900幅実frame非重複・typed action回帰を保存freeze。Header幅修正と合わせ`just fmt unit-test`を実行中（`tmp/matrix-layout-unit.log`）。次はこの終了取得後、Mesa25上の通常GPUI KUC画面と全3host matrixを検証する。
- Mesa25比較image28890 exit0。同じGPUI標準quad診断binaryをMesa25.0.7-2~bpo12+1で動かすprobe16119がexit0、緑背景/赤矩形をPNGで確認。Mesa22.3.6では同binaryが黒一色だったため、ドライバー更新で表示が復旧することを実測した。正式DockerfileへDebian backportsの検証済み25.0.7固定を追加。通常KUC sourceを再buildした実画面の確認は次工程（既存Docker binaryは診断用）。
- egui/Floem既存5scenario実採取75547 exit0。全10PNGの寸法がrequestに一致し、各scenarioのegui/Floem PNG hashも一致（`tmp/native-matrix-evidence.json`）。ただし目視でOutput本文/ボタン重複と430px header履歴欠落を発見し、基準画像には採用しない。Output担当が修正中。
- Header欠落は回帰50977でRED確認、タイトル幅をprovider/2action/gapの残幅へ制限し31520 GREEN。全体gate/画像再検証は未実行。Xvfb画面を許可viewport最大4096角へ広げ、1400px縦scenarioがdisplay外へ出ないようにした。
- GPUI環境はMesa22.3.6を実測。Debian公式backports Mesa25.0.7との比較用imageを28890でbuild中（`tmp/mesa25-image.log`）。製品sourceの診断変更は撤去済み。次cargo前にOutput担当freeze確認が必要。
- GPUI標準quadだけの診断49654も30秒黒一色exit1（`tmp/gpui-quad-probe.log`）。KUC imageを外しても同症状のため、KUC画像変換だけを根本原因にはできない。temporary trace/緑背景/赤矩形はすべて除去し標準KUC mountへ復元、diffcheck PASS。全cargo停止。Docker内manual-host-gpui binaryは診断quad版のままなので、次回は必ずsourceからbuildしてから製品画像を採取する。次の診断はGPUI/Blade/Vulkan/X11表示経路とMesa環境を確認する。
- Lavapipe ICD明示probe75721も単色でexit1。temporary prepaint/paint traceは除去済み。fixture背景を緑にする診断56429も黒一色exit1で、次にKUC imageを一時的に外してGPUI標準の緑背景+赤矩形だけを描く診断49654を実行中（`tmp/gpui-quad-probe.log`）。これは製品実装や達成証拠ではない。終了後、`tools/manual-host-gpui/src/fixture.rs`のRenderを必ず`div().size_full().child(self.view.clone())`へ戻す。
- viewport後3manual gate97169 exit0（egui5/Floem52/GPUI4 tests、strictclippy/check各PASS）。GPUI実X11 resize probe7191は単色でexit1。temporary描画trace96935では1440x900 prepaint/paintに到達しエラー無しだが、captureは単色でexit1。描画未到達・0寸法ではない証拠を得た。`element.rs`のtemporary eprintln2行は診断終了後必ず除去する（正式成果物に残さない）。全cargo停止。
- GPUI診断49579はviewport Fixture戻り型不整合でexit1。親がmain::fixture_or_exitをOption<fixture::Fixture>へ修正し42236で再実行、build/spawn成功後も30秒単色でexit1。追加したredraw/paint_imageエラーはログに出ていない（`tmp/native-linux-gpui-diagnostic.log`）。これだけでGPU/capture原因と断定せず、実paint到達・X11 expose・Vulkan presentの境界を次に切り分ける。全cargo停止、全担当freeze。
- Floem実採取51428 exit0。標準mountのsize指定修正後、既存empty-chat requestの1440x900でheader/composer/controlが描かれ、白画面が解消したことをPNG目視で確認（`tmp/native-linux-floem-layout.log`、`tmp/native-linux/floem/native-floem.png`、hashfc1ba602…）。baseline未指定のartifactであり0015 matrix合格ではない。
- GPUI診断版を実採取49579で実行中（`tmp/native-linux-gpui-diagnostic.log`）。全cargoはこのDockerのみ、次turnは同handleの終了を取得する。viewport parserは幅320/高さ240の下限に揃え保存freeze、3manual unit/strictlintは未実行。
- 全unit4665 exit0/180 PASS（ACP17/core118/egui12/Floem8/GPUI6/AST19）。Floem標準layout回帰とGPUI描画エラー処理を含む。native viewport parserは高さ下限240と幅下限320を揃える最終修正中。現時点で全cargo停止、次はLinux実画像で確認する。
- Floem標準mountへsize_full/min-sizeを追加し公開Style→Taffyの非zero layout回帰を保存。GPUIはredraw/paint_image失敗をstderrへ報告し、失敗時の旧image採用を停止。親reviewでEntity::update(&mut App)の戻り値を直接Resultとして修正した。全unitを`tmp/native-layout-unit.log`へ実行中。
- native viewport伝播が保存されたが、親reviewで3manualのf32 parserがNaNを受理する点と旧surface単体引数の互換性を指摘し修正中。狭幅430x900/高縦900x1400を縮小せず実clientへ反映する。最終compile/画像はまだ未検証。
- GPUI実採取62297はbuild/spawn/window検出後、30秒黒一色でexit1。画像を目視確認し、全cargoは停止した。GPUI担当がadapterのredraw/paint_image失敗の観測を追加中、Floem担当は標準viewの非zero layoutを修正中。host担当は既存5scenarioのviewportをscreenshot→3manualへ伝播する変更中。担当範囲は分離し、次cargoは保存後freezeを確認して開始する。
- ディスク空き約2.4GiB。今回のMac targetのKCU4packageのみcargo cleanで289.1MiB回収（exit0）。Docker実採取cache、検証ログ、画像、他repoは保持した。
- Floem実採取19117は30秒単色でexit1、標準TinySkia指定77021もexit1。白背景だけでchatが描かれないことを目視確認。RasterChatViewの標準layoutが0寸法の可能性を担当が調査修正中。GPUI実採取62297を開始（`tmp/native-linux-gpui.log`）。全cargoはこのDockerのみ。終了取得前に次cargoを起動せず、次turnはこのhandleをpollする。
- OpenSpec validate69473 exit0、diffcheck exit0。scope0014〜0016の古い「完了」表記を検証中/実装中へ訂正し、DoDは保持した。既存5scenarioは1440x900/430x900/900x1400、native固定1280x900ではmatrixを満たさず、viewport伝播の追加が必要（read-only調査済み）。
- strict lint49083 exit0、git diffcheck exit0。Linux egui再採取6723 exit0。`tmp/native-linux/egui/native-egui.png`を目視し、header/provider/composer/control/usage/sendの実描画を確認（hash823ed3b2…）。単色起動frameは再試行される。1scenarioのartifactであり、baselineや0015の合格にはしていない。Floem実採取を開始した（`tmp/native-linux-floem.log`）。
- 全unit94580 exit0、179 tests PASS（ACP17/core118/egui12/Floem7/GPUI6/AST19）。GPUI非原点TestWindowのpointer/character index/IME boundsとFloemrepeat回帰が通過。fmt完了後lint38914はtestの不要String変換1件で失敗し修正、49083で再検証中。Linux描画待ちは保存済み、親reviewでclosureのResult不足を修正した。次はLinux実採取の再実行。
- 本文到着後activity表示の回帰はRED77383（2 FAIL）から修正後core57360 exit0/118 PASSへ進んだ。stateのactivity自体は保持し、本文がある場合の表示を抑制する。Floem repeat・GPUI非原点修正は保存済みで、全unit94580を実行中（`tmp/p1-unit-current.log`）。
- Linux egui採取72604 exit0で実PNGを取得したが、目視で黒一色の起動前frameと判明。0015の証拠には使わない。子process生存確認と実画素の非単色観測を組み合わせた描画待ちへ修正中。期待hashによる品質gateは別途必要。

- Linux workspace52017 exit0。manual92690はegui4/Floem51 PASS後GPUI Render import不足でexit101、AppContext不足も追加修正しmanualGPUI53430 exit0/3 tests PASS。runner Mac strictclippy4123 exit0（先行21752のtest expect/Linux専用helper deadcodeを修正）。
- Linux egui native88183は実window検出後xwd欠如でexit1。Dockerへx11-apps追加し64143 imagebuild exit0。初回画像はまだ無し。native runnerはCARGO_TARGET_DIRを尊重しhost stderrを保持、LinuxPNG metadataをstripしてhashを再現可能化。
- 独立レビューでP1三件を確定: 0017本文到着後activity併記、Floem CmdEnter repeat、GPUI非原点mount座標。Floemは修正保存済み未検証、GPUIは担当が修正中。親はactivity表示回帰を追加し77383でRED確認中（`tmp/activity-regression.log`）。cargoはこの1本のみ。

- Linux workspace checkをDocker session52017で実行中（`tmp/linux-workspace-check.log`）。`--workspace --all-targets --all-features --locked`、source read-only、専用volume `kcu-v010-linux-target`/`kcu-v010-linux-registry`、debug0/jobs2。次turnは同handleをpollし終了前に他cargoを起動しない。container名kcu-v010-linux-check。
- manual-ui-check21283 exit101（egui DroppedFile path API/fixture startup error型）。担当が修正保存済み、3manual不正fixture引数を非zero化、GPUI main関数分割済み。未再検証、全担当freeze。
- native_captureレビュー修正は保存済み。Linux IsViewable/PID/title/PNG IHDR厳密照合、不正IDはErr、expected hash指定時は実capture一致までpoll。hashなしはartifactのみと明示。MacはCGWindow physicalサイズ+shadow除外、子PID一致へ修正。親CLI --native-request接続済み、runner/3manualのcompileと実画像は未達。

- Linux build image78666 exit0。`scripts/screenshot/linux/run-xvfb.sh`を追加し、displayfd+xdpyinfoで実readyを確認する起動へ変更。DockerでGL/Vulkan probe exit0（`tmp/headless-xvfb-wrapper.log`、XDG errorなし）、子exit17がrunner exit17へ伝播することを実測。shell構文PASS。3hostアプリのLinux build/採取は未実行。
- 3manual fixture親reviewでnestedResult/mut不整合・描画失敗伝播を指摘し担当が修正中。native capture親reviewでPNG IHDR寸法/IsViewable/ID parse失敗/初期描画待ちを指摘し修正中。全cargo停止。

- `just lint`6959 exit0。先行47883/70754で検出したegui expect/test関数長、core test関数長を基準維持で修正。manual-egui/gpuiへfixture専用serde依存を追加しoffline lock同期85797/後続ともexit0。
- Linux実測: Docker linux/aarch64が利用可能。Xvfbの実displayにxdpyinfo成功、glxinfo/vulkaninfoでllvmpipe OpenGL4.5/Vulkan CPU adapter確認（`tmp/headless-gpu-live.log`）。xvfb-run wrapperは起動待ちで停滞したため、稼働中Xvfbへ直接probeした後、当タスクcontainerをstop22341 exit0（元80736 exit137）。これを3host描画成功とはしない。
- Linux capture backendと3manual --kcu-surface入口を追加しreview中。親は--native-requestから同一serialized surfaceを生成する接続を追加。未compile/未native採取。Linux hostbuild用Docker imageを78666で構築中（`tmp/headless-build-image.log`）。ソースは各担当保存後freeze、次cargo前に編集区切りを確認する。

- full check68435 exit101、core lint2件（vendor.rs manual_clamp、vendor_ui/surface.rs from_profile37/30lines）。core担当が基準維持で修正中、全cargo停止。3host Linux capture backendをscreenshot担当へ分離し実装開始、同一surface fixture入口はread-only設計中。

- `just fmt`59291 exit0、git diffcheck exit0。`just check`を68435で実行中（`tmp/pure-kuc-check-current.log`）、現在lint依存check段階。同handle終端まで次cargoを起動しない。全unit170/E2E9 PASSからfull gateへ進行。

- `just unit-test host-e2e`45491 exit0。全workspace170 tests（ACP17/core115/egui9/Floem5/GPUI5/AST19）とE2E9 tests（conformance8/hostflow1）PASS、E2E strictclippy PASS（`tmp/narrow-unit-e2e-current.log`）。狭幅/dropdown/egui0.36 APIとtexture後処理修正後、全担当source固定状態で確認。全cargo停止。full check/manual/画像/coverage/releaseは未完了。

- E2E90469 exit0、strictclippyとconformance8/hostflow1 PASS。実行中に追加core修正が保存されたためこれを最終差分の全gate証拠にせず、eguiTexturesDelta対応と合わせ`just unit-test host-e2e`45491で再実行中（`tmp/narrow-unit-e2e-current.log`）。全担当にsource保存停止を通知、終了前に次cargoを起動しない。
- KUC Buttonの明示height指定時に描画とhitの縦送りが異なるため、dropdown optionは指定heightを外し公開default20px+gap8へ統一、menu実高28px/項目へ同期しhit包含回帰を追加。usageはIconTextButton/Iconで矩形背景を出さず固定28x28を維持。eguiheadlessはtextureサイズ検証後に差分を明示消費する新APIへ移行。

- unit84182 exit101、ACP17/core115全PASS、egui5PASS/4FAIL。狭幅usage/dropdown回帰は解消。egui4件は0.36のTexturesDelta未消費Drop panicで、担当がtexture内容検証後の明示消費へtest移行中。egui非依存の`just host-e2e`90469を実行中（`tmp/host-e2e-current.log`）、次cargo前に同handle終了取得。
- 3host screenshotはLinux Xvfb上の実window採取が上流編集なしの候補。mac専用captureのLinux対応、同一scenarioの標準mount、Floem CPU renderer制約の実測が必要。UI fixture画像は実provider E2Eの代替にしない。調査をdocs/headless-three-host-plan.mdへ保存中。

- 要件監査の最新証拠欄を現KUC/3adapter/依存更新状態へ修正し、旧Floem git pin説明を履歴へ区別。OpenSpec validate69711 exit0、git diffcheck exit0。仕様構造の検証であり全要件達成ではない。

- egui更新後unit9775 exit101、IME deprecated/Preedit構造/RawInput.modifiers変更の計5件を検出、担当が追加移行中。core単体6200 exit101、114PASS/1FAIL。usage到達性と非重複はPASS、dropdown overlapが残りcore担当が再診断。全cargo停止。
- 3host画像調査ではhost texture/paint入力の取得は可能だが、それだけを0015の最終headless screenshot証拠にはしない。Floem/GPUIのoffscreen render/readback公開APIを追加調査中。

- 狭幅のusageを固定28×28の公開KUC Button primitiveに変更し、dropdown重なり回帰を実入力高へ同期。egui/eframe/egui_kittestを0.36.1へ更新、IME必須fieldsを追加しroot/manual-egui/screenshot lock更新は全exit0。`just unit-test host-e2e`を9775で実行中（`tmp/narrow-unit-e2e-current.log`）。終了前に次cargoを起動しない。
- 空き123MiBのためビルド停止を確認し、root targetのKUC/KCU/egui関連package cacheのみcargo clean47083 exit0、1.6GiB回復。ソース・画像・ログは保持。先行core80610は依存更新によるlocked不足でexit101、更新後上記9775へ統合。

- 狭幅unit/E2E42207はunitでexit101、core112PASS/3FAIL（`tmp/narrow-unit-e2e-current.log`）。usage hit幅がsend領域へ延びる2回帰、縦controls化後のdropdown重なりfixtureが失敗。E2Eには未到達。core担当が実hit/raster境界を修正中、cargo全停止。

- manualFloem86185 exit0、strictclippy/check/51 tests PASS（`tmp/manual-floem-current.log`）。旧未使用entry整理後、現actorと既存回帰を維持。
- 狭幅修正収束: attach/usage/sendを幅に応じた明示left座標へ配置、controlsは不足幅で縦配置しcomposer高さ/入力bounds/popupアンカーを同期。120/320/560の実frame回帰追加、unit→E2E連続gateを開始（`tmp/narrow-unit-e2e-current.log`）。
- 最新公開依存のread-only再監査ではegui/eframe0.36.1が未採用、他の調査対象とACP CLI pinは公開最新版一致。egui更新は旧native hashを不採用根拠にせず現構成で再評価する。

- manual-Floem67705はAPI/import7件を解消後、旧cfg(test) provider/stateの未使用entry/helpers9件でstrictlint失敗（exit101、`tmp/manual-floem-current.log`）。担当が現共通actorとの責務対応を維持して旧未使用コードを整理中、lint抑制はしない。
- coreはaction行の固定spacerを公開KUC絶対配置へ置換、狭幅controlsの縦配置とcomposer bounds/dropdownアンカー追従を実装中。cargo全停止。
- 独立依存監査を再開し、各manifest/lockと公式公開最新版をread-only照合中。旧native hashだけでegui更新を不採用にはしない。

- manual-GPUI26276 exit0、strictclippy/check/3 tests PASS（`tmp/manual-gpui-current.log`）。egui実hostと合わせ接続gate通過、Floem実hostの修正・再検証は残る。現在cargo全停止。

- manual-Floem56179はlock不足でexit101、offline同期exit0。空き973MiBに低下したためcargo全停止を確認しrelease build cacheのみclean4599 exit0（2.0GiB回復）、画像/log/sourceは保持。
- manual-Floem80518はAPI/import不整合7件でexit101（`tmp/manual-floem-current.log`）、担当が修正中。rootはmanual-GPUI gateを実行開始（`tmp/manual-gpui-current.log`）。

- host-e2e30970はstrictclippy PASS後conformance7/8 PASSでexit101。320px narrowでchat.composer.usageの実node hitが欠落（`tmp/host-e2e-current.log`）。footerの固定444px spacerが狭幅でも維持される不具合をcore担当が修正中。120/320/560でusage/send/controls到達性を維持する回帰を要求。E2E基準は変更しない。
- cargo全停止。次gateはcore狭幅修正後のunit/host-e2e、画像matrix。

- host-e2e84921はE2E helperの不要f32 cast2件でstrictclippy失敗（exit101）。root修正後30970で全gate再実行中（`tmp/host-e2e-current.log`）、現在test依存compile段階。同handleをpollして終了確認する。root target debug8.3GiB/release2.1GiB、filesystem空き1.6GiBを確認し、cargoは1本に限定。

- host-e2e63390はlock同期不足でexit101。`cargo update --manifest-path tools/e2e-host-app/Cargo.toml --offline` exit0で同期し、同gateを再実行中（`tmp/host-e2e-current.log`）。次turnは現在handleを確認し終了前に再起動しない。

- 画像35451はexit1、PNG生成後旧baselineとの差異（actual c9997db8d050b0bb185d21b2e288b4acf235c0d7fcc47a34682242fe196b69ee）。rootが実画像を確認し、composer黒背景/二重scale/Attach Send raw矩形は解消。5scenario全画像と3host最終確認は残る、baseline未変更。
- E2Eはreview修正（frame参照・center/left・Markdown内容・message幅）収束、次にhost-e2e gate実行。

- `just unit-test`51265 exit0、計169 tests PASS（ACP17/core114/egui9/Floem5/GPUI5/AST19、`tmp/pure-kuc-unit-current.log`）。disabled抑止と高DPI・dropdown修正後の全workspace unit/AST結果。manual全host/全E2E/画像/coverage/release gateは別途未達。

- manual-egui32969は旧respond_permissionメソッド呼出でexit101。rootがManagedHostへRespondPermission typed actionを送る現経路へ修正し、23279 exit0でstrictclippy/check/4 tests PASS（`tmp/manual-egui-current.log`）。
- core4件はテストのみをopen popup overlayの実tree/hit契約と設定済fixtureへ移行、library変更なし。`just unit-test`51265で再実行中（`tmp/pure-kuc-unit-current.log`）。次cargo前に終了を取得する。

- `just unit-test`21512はexit101、core110PASS/4FAIL（`tmp/pure-kuc-unit-current.log`）。disabled Submit抑止、retina logical layout/physical density回帰はPASS。dropdown composer重なり、旧子option期待2件、controls未設定fixture1件をcore担当が修正中。初回85065/34917の加算型不整合はroot修正済み。
- pointer enabledはhost_action.enabledとancestor disabledの両方を要求する。E2Eは現KUC実frame/dispatch検査へ移行後review中。rootがcurrent_frame参照、action rect左端/中心区別、Markdown内容検査を追加要求した。
- cargo全停止。次はcoreとE2E担当収束後にunit/host-e2e/画像検証。

- 3adapter gate74518 exit0（egui9/Floem5/GPUI5、`tmp/three-adapter-current.log`）。先行3702/37448はmodule/import/visibility/lifetime不整合、45949/68005はoption.common(default)がhost_actionを消すため失敗し、rootが階層修正と個別width/height builderへ修正した。
- ただしrootの追加reviewでdisabled Submitでもpointer actionが実行される不備を発見。core担当がenabled判定と回帰を追加中、74518を最終品質証拠にしない。Floem fixtureもdraftを実sessionへ設定してsend_enabledを実状態へ合わせた（未再実行）。
- 現在cargo全停止。次はcore disabled/dropdown/DPI回帰収束後に全gate/画像再実行。

- active change tasks.mdの旧native最終check/coverage、旧自己レビュー、依存最終採否を現構成未完了へ訂正し、過去結果を履歴と明示。変更後OpenSpec validate10759 exit0、diffcheck PASS。
- eguiキーsequenceはpress/repeat/releaseとRawInput.modifiersを実入力へ合わせ修正済み、未実行。core高DPI/dropdown修正中のためcargo全停止。root reviewでoverlay優先hitとcomposer outside-click閉鎖を追加要求。
- e2e-host-appの旧include_str conformanceを共通KUC実frame/dispatch検査へ移行中。3host固有の実headless gateとの要件対応を明記し、共通検査を3host実行証拠とはしない。

- OpenSpec list12260 exit0、activeは今回のpure-kuc change1件のみ（27/34 tasks、in-progress）。旧v0.1〜v0.6の6changeは指定`2026-05-09-superseded-*`archiveに存在。0001の旧change移動は直接確認したが、元DoDの全v0-*不在という文字面と今回新change名の関係は監査上明記して残す。

- egui93118はexit101、Submit count0の失敗が残る（8pass1fail）。直前EnterのKeyUp不足によるrepeat正規化を含め診断中。cargo全停止。
- core担当が公開KUC0.3.6に基づき高DPI blitのlogical area修正と、root絶対配置の公開Button/Columnによるdropdown描画・hitを実装中。TextSurfaceのgrapheme boundsは既にlogicalであり、座標の再縮小は不要と確認。画質を落とさない。
- 要件監査の最新証拠を58host tests/実ACP履歴probe/現adapter不合格へ更新。3adapterのpublic moduleがrasterのみであることをrootも確認。

- 実Codex ACP履歴probe73141はexit0（`tmp/acp-history-real.log`）。実prompt応答→Manager経由保存→新Host読込→bind/Ready→同一provider再選択2回→質問に値を含めない履歴token回答→次ACKpromptまでPASS。全4provider/3hostの実操作完了には拡大しない。
- egui fixture同期修正後のlibrary全testsを93118で実行中（`tmp/egui-input-current.log`）。次cargo前に終了を取得する。

- host-support53055はprobeのprivate persist呼出でexit101。Manager.pollがevent毎に保存する実経路を確認し冗長呼出を除去、81698はexit0でstrictclippy/58 tests PASS（`tmp/host-restore-current.log`）。
- 実Codex履歴probeをsession73141で実行中（`tmp/acp-history-real.log`）。終了を取得するまで未検証。
- Floem provider選択失敗はopen SelectBox childrenのaction-hit欠落と判明。公開registry KUC0.3.6での確認と、公開popup等のprimitive構成によるKCU側修正を調査中。adapter側独自hit生成/別repo編集はしない。

- host-support98274はexit101、worker::run cognitive complexity11/10でstrictclippy停止。担当が初期化責務分離を修正中。cargoは全停止済み、上記98274実行中記載は終了履歴へ更新。probeは未compile/未実通信。

- Floem/GPUI68018はFloem provider dropdownの`missing Floem action hit: codex-cli`でexit101（4pass1fail）、担当が診断中。eguiはcallback後surface再生成という実host契約へfixture修正済み・未再実行。
- host-support全gateをsession98274で再実行中（`tmp/host-restore-current.log`）。次turnはこのhandleの終了を取得する。
- composer文字サイズ不整合はphysical text rasterをlogical canvasで再拡大する二重scaleが原因候補。scale1固定で画質を落とす案は採用せず、公開KUC型でlogical layout変換できるか調査中。別repo編集なし。

- 3adapter53810はFloem test helperのlifetime不足でexit101、rootが実Viewのstatic callback契約へ合わせ修正。再実行55306はegui IME/Enter通過後Submit count 0（期待1）でexit101、固定surfaceのdraft状態反映を診断中。
- egui診断と分離してFloem/GPUI library全testsをsession68018で実行中（`tmp/floem-gpui-current.log`）。終了取得前に次cargoを起動しない。

- core palette/icon仕上げとFloem/egui実入力回帰が収束。`just headless-adapter-check`をsession53810で実行中（`tmp/three-adapter-current.log`）、同handleをpollしcompile継続を確認。終了取得前に次cargoを起動しない。
- 実ACP履歴probeを`tools/host-support/src/bin/acp_history_probe.rs`へ追加済み。root reviewで復元質問自体のtoken値を除去し、履歴なしに正答できない質問へ修正。cargo/実通信未実行。3回目ACKは通常継続確認であり履歴の再注入不在の証拠とはしない。

- host E2E entryのKUC mount失敗をexpect panicせずResult伝播へ修正し、binary pathをCargo compile-time `env!`で取得するよう修正。fmt/diffcheckのみ、host-e2e gate未実行。
- 次検証: core UI差分収束後にhost-support/3adapter gateを再実行。host担当は実Codex prompt→persist→再読込→再接続→次promptの履歴継続probeを公開APIで実装中、egui担当は実Context入力→typed Submitの回帰追加中。

- `just ast-lint`はsession65847 exit0、19 tests PASS（`tmp/pure-kuc-ast-current.log`）。
- 起動時bind/history context修正後の`just host-support-check`12978はexit101。core UI palette編集中のunused fieldsで止まり、host検証未到達。core/host両担当の収束後に再実行する。全cargo停止済み。
- Floem headlessテストはplatform/fixture分離まで収束、cargo未実行。旧`tools/e2e-host-app/tests/adapter_conformance.rs`は旧native include_str検査が残り、現純KUC経路の実証として不十分なため対応を監査中。

- `scripts/openspec validate v0-2-0-pure-kuc-multivendor-acp`はsession9654 exit0。change構造がvalidである証拠であり、既存0001〜0017の実装・全品質検証完了を意味しない。
- rootの再レビューでworker初期化がprobe後にsession identity/history contextをbindしない点を確認。起動復元→接続→次promptの履歴継続と二重注入防止をhost担当が調査修正中。

- 画像59632はexit1、PNG生成後baseline不一致（actual f23db6db5a2b6735388c60e6a5e8034795b1ba9aa48e7e53914ddd5f1185463b）。実画像でheader buttons/primary右寄せは改善したが、composerのみ黒背景・大文字、Attach/Sendのraw矩形、pulldownの識別性が未達。baselineを変更せずcore担当が0003/0005/0016の仕上げと回帰を継続する。

- GPUI headless検証はSDK API不整合2件を修正後、session88153 exit0、5 tests PASS（`tmp/gpui-headless-tests.log`）。実TestWindowのpaintとpointer→Cmd+EnterでSubmitが1回だけ発生する回帰を含む。全3host/全品質gateの証拠には拡大しない。
- header/icon/controls修正後のempty-chat画像をsession59632で再生成中（`tmp/pure-kuc-image-layout.log`）。終了状態と実画像を確認するまで未検証。Floem headlessテストは追加済みでファイル分割を整理中、cargo未実行。

- `harness-screenshot-matrix-check`の前提に`headless-adapter-check`を追加し、全3adapter library testsを必須化した。`just --show`でrecipe構文を確認。全gate自体は未実行。docs/kuc-migration.mdに現接続状況と旧設計記録の境界、egui PNGのみでは3host証拠にならない点を追記した。
- GPUI40798は同handleをpollし稼働継続を確認。test-support依存のcompile中で、再起動していない。floem_bridgeはFloem固有のEvent→dispatch→callback/image提示のheadless回帰を作業中。

- GPUI headless testは初回locked不足でexit101、root lockにtest-support由来9 packageを同期しsession40798で再実行中（`tmp/gpui-headless-tests.log`）。次turnはこのhandleをpollして最終結果を取得する。

- `just host-support-check`はsession9663 exit0、strictclippyと57 tests PASS（`tmp/host-boundary-output-check.log`）。新規/履歴session境界、Markdown context/output、cancel/drop、失敗undoの永続理由と外部編集保持を含む。path parserのdot区切り誤認と日本語quoteのUTF8境界、diff末尾改行期待値を修正して全host-support gateを再実行した。
- GPUIの実headless TestWindow回帰のためdev依存にgpui test-support featureを追加し、`cargo test -p katana-chat-ui-gpui --lib --locked`を開始（`tmp/gpui-headless-tests.log`）。終了前に次cargoを起動しない。core header/icon/controls追加修正は収束済みだが再画像未検証。

- composer行分離・light canvas背景・output状態表示追加後、core106 tests PASS（session14964 exit0、`tmp/kuc-layout-output-tests.log`）。最初の71312は新テストsetupのProviderMissingでexit101、configured setupへ修正して全core再実行した。
- 画像session80576はexit1、PNG生成後baseline不一致（`tmp/pure-kuc-image-layout.log`）。実画像で不要panel/黒背景は解消したがheader provider selector/newchat/historyが可視ボタンにならず、controls区切りとprimary配置も不十分。kuc_boundaryが次修正中。baseline変更なし、画像品質gate未達。
- rootがChatUiOutputSurfaceにstatusを保存互換default付きで追加し、KUC output nodeに成功/取り消し/失敗理由を表示。core106件にそのsession→surface→node回帰を含む。hostのoutput action種類/状態検証と不正ApplyDiffでFileCandidateを書き込まない実ファイル回帰は追加済み・未実行。
- GPUI実TestWindowのheadless入力/paint回帰はfloem_bridgeが追加し、禁止panicと実API整合をレビュー修正中。host_actorは新規/履歴境界のprovider再接続・identity同期と次prompt履歴contextを修正中。現在cargoは全停止。

- 空stateのhistory/capabilities/output/permission可視条件を修正し、coreテストをsession7846で実行中（`tmp/kuc-visibility-tests.log`）。終了取得前に次cargoを起動しない。composer配置/light背景の修正はこのgate後に再開予定。
- rootがoutput apply/undo失敗を`OutputStatus::Failed(reason)`へ記録・永続化する経路と、外部変更を保存して失敗理由が再起動後も残る実ファイル回帰を追加した（`output_failure_tests.rs`）。未検証。
- host_actorはNewChat/SelectHistory時に旧remote session・stream・permissionを新sessionへ流入させない境界を修正中。floem_bridgeは標準actor接続の実装を収束し、現在0015の実3adapter headless検証方法をread-only調査中。旧Floem回帰はcfg(test)で保持、新seq/error/effect/drain/weak寿命回帰を追加したが未実行。

- 画像再検証session49232はexit1、PNG生成成功後baseline不一致（`tmp/pure-kuc-image-retest.log`、`target/harness-screenshots/empty-chat.png`）。実画像を確認するとpermission無しの確認panel、Capabilities/Output/No history常設、header低contrast、composer controls重なりがある。基準更新せずkuc_boundaryが共通KUC可視条件/style/layoutと数値回帰を修正中。スクリーンショット生成成功は品質達成ではない。
- egui/Floem actor移行、Katan request→structured output接続、モデルlabel伝播、Ollama cancel/drop世代判定は実装済だが未統合検証。host_actorが添付本文による操作誤判定・無空白日本語path・構造化response prompt契約をレビュー修正中。floem_bridgeが新経路回帰継承と禁止expect/wildcard修正中。現在cargoは全停止、次gateは担当収束後。

- `just manual-ui-check-gpui`はlock同期後session46050 exit0、strictclippy/check/3 tests PASS（`tmp/gpui-actor-check.log`）。ただし後続core model label/共通Markdown統合差分は含まない。egui session71026は編集中のkatan_request module不在でexit101、収束後に再実行する。現在cargoは全停止。
- 実Ollama `http://127.0.0.1:11434/api/tags` は成功し`models: []`。Claude OAuth更新と利用するOllama model名をユーザーへ非同期質問した。これは全作業のblockerではない。
- Floem actor移行は実装後レビューでdraft seq未保持、drain無制限、window破棄後timer存続等を修正中。旧moduleの未コンパイルで旧testsが検査から落ちるため、新経路の要件検査への継承を監査中。単なる対応表をPASSの証拠にしない。
- 依存egui0.36の旧native画像hash不一致を理由にした過去の不採用判断は、純KUC画像経路が成立した後に再評価する。新経路の品質gateを維持した最新互換版を採用する。

- 共通host actor/direct managerの統合後、`just host-support-check`はsession40996 exit0、strictclippyと33 tests PASS（`tmp/host-actor-check.log`）。コンパイル不整合・actor複雑度を修正し、KatanAgent cancel completionの短絡評価でexecutorを復帰しない不具合も修正した。Ollama通知permitの次prompt持越し回帰は追加修正中であり、このPASSには含まない。
- GPUI actor接続の`just manual-ui-check-gpui`をsession19175で開始（`tmp/gpui-actor-check.log`）。egui actor移行は実装済・未検証、Floem actor移行は作業中。Markdown候補共通moduleはレビュー修正中・未接続。
- 画像matrix session43486はexit1で終了。empty-chatが`KUC viewport exceeds the host texture limit`。headlessが2880px textureにegui既定2048制限を渡していたためharness capability4096の明示と事前検証、実texture upload/超過拒否の回帰を追加済み。baseline変更なし、画像gate再検証は未実行。

- screenshot lockを最新化後、runner strictclippyと9 testsはPASS（`tmp/pure-kuc-screenshot-check.log`）。画像生成release compileは空き740MBまで低下したためsession4407をCtrl-Cでexit130に停止、未完了。全build停止後root dev cache15.7GiBをcargo cleanし空き16GiB。以後通常build/testは`CARGO_PROFILE_DEV_DEBUG=0`・incremental0・jobs2で統一し、検査範囲/coverage基準は維持する。
- 同じ画像matrix commandだけsession43486で再開（`tmp/pure-kuc-image-matrix.log`）。baseline変更なし。終了状態を確認してから次cargoへ。
- floem_bridgeのKatanAgent direct manager統合は収束・未検証。host_actorが背景host処理を追加しrootがGPUI state/main接続を実装、ManagedHostのerror/effect消失・shutdown・禁止testmacroをレビュー修正中。host-support lib exportとTokio rt/time features追加済み、ここはまだcompile未検証。

- workspace `just unit-test ast-lint`はsession68620 exit0でPASS（`tmp/pure-kuc-unit-ast.log`）。workspace lintと合わせ新runtime経路の通常Rust/AST gateを通過。残るmanual全host/E2E/画像/coverage/release gateは未達。
- 共通ProviderManager + Ollama executor追加後の`just host-support-check`は30 tests/strictclippy PASS（`tmp/host-manager-check.log`、session29816 exit0）。manual-eguiをmanagerへ接続し、API署名/host effectの統合修正後3 tests PASS（`tmp/manual-manager-check.log`、session53818 exit0）。全cargoは停止済み。
- 次の実装はfloem_bridge担当のOllama→manager統合と、kuc_boundary担当のOllama cancel通知lost-wakeup/prompt future drop時のstate残留修正。両者へ編集開始通知済み。rootは当該ファイルを編集しない。
- 空き容量低下に対し、open fileなしを確認した旧`tools/host-support/target` 1.5GiBと、全build停止後の旧root release cache2.0GiBをcargo cleanで削除した。source/公開成果物は削除していない。root debug cacheは維持、後続build前に空き容量を確認する。

- workspace `just lint` はsession57323 exit0でPASS（`tmp/pure-kuc-lint.log`）。core/Floem/GPUI、probe example、ACP testsの各指摘を無効化なしで修正した。依存block/proc-macro-error2のfuture-incompatibility通知は残るため依存採否で確認する。
- 最新差分の`just unit-test ast-lint`をsession68620で実行開始（`tmp/pure-kuc-unit-ast.log`）。結果が揃うまで関数分割後の全動作検証は未完了。

- ACP typed SDK構築/Boxイベント化後の`cargo test -p katana-acp-client --all-targets`は17 tests + example compile PASS（`tmp/acp-typed-wire-retest.log`）。最新binaryでCodex実2prompt/同一sessionもPASS（`tmp/acp-current-real-retest.log`、session46538 exit0）。
- host-support gate15500はpermission条件のcollapseとtest行数/Path参照の4 lintでexit101。root修正済み。再実行92355はcore編集中の未定義関数で停止。core担当の収束後に再実行が必要。
- core runtime関数分割は収束。workspace lintをsession2614で再実行中（`tmp/pure-kuc-lint.log`）。ProviderManager実装はfloem_bridge担当でhost-support内のみ進行中、3host配線とdirect Ollamaは後続。

- ACP lint修正後のworkspace lint21450はcoreの不要lifetimeと関数行数3件でexit101。kuc_boundaryが該当runtime3ファイルを責務分割中。ACP14件はこの実行で再発なし、全lint通過は未達。
- host-support lifecycleのResult型変換をrootが修正し、provider起動profile/権限clearを含む24 tests PASS（`tmp/host-lifecycle-retest.log`、session26225 exit0）。共通profileは3 ACP providerの公式commandを生成するが、3host selectorへの接続は残る。
- `just host-support-check` はsession15500（`tmp/host-support-check.log`）で実行中。後続cargo前に最終状態を確認する。

- `just lint` session19113はACP層14 diagnosticsでexit101。SDK型の直接構築、wildcard import排除、connect責務分割、SessionUpdate Box化とconsumer追従を修正。再実行session21450（`tmp/pure-kuc-lint.log`）は実行中。
- Claude adapter0.74.0（npm viewで確認）を既存probe binaryで実起動。initialize/sessionnew成功、promptはOAuth期限切れでexit1（`tmp/claude-acp-real-probe.log`）。実会話は未達。認証更新後に最新binaryで再検証が必要。他作業の全体blockerにはしない。

- ACP config保持を含む17 tests PASS（`tmp/acp-config-retest.log`）、permission投影/validationを含むhost-support19 tests PASS（`tmp/host-permission-retest.log`）。その後追加中のlifecycle testsは未検証。
- manual-egui統合3 tests PASS（`tmp/manual-egui-acp-retest.log`）。permission応答・model選択をbridgeへ接続し、with_rootテストで実agentを自動起動しない。実agentの権限往復はまだ未検証。
- GPUIは画像に加えてUTF16 IME→実KUC draft effectを含む4 tests PASS（`tmp/gpui-input-retest.log`）。session42966/37277はexit0取得済み。
- `just fmt-check`は一度全manifest PASS。その後lifecycle編集中に`just check`がfmtで停止したため、収束後に全checkを再実行する。現在`just lint` session19113（`tmp/pure-kuc-lint.log`）でworkspace lintを実行中。稼働を確認してからpollし、重複cargoを起動しない。

- 共通KUC coreは100 tests、Floemは4 tests PASS（`tmp/kuc-floem-retest.log`、process exit 0）。逆方向選択のコピーとpermission modal背景遮断を含む。全OpenSpec達成の証拠にはまだ不足する。
- 共有hostは実ファイル・履歴等13 tests PASS（`tmp/host-support-retest.log`、process exit 0）。追加したpermission validation testsはこの結果には含まない。
- ACP permissionのrequest ID/option表示名を保持し、応答時に保留要求と選択肢を検証するよう修正。model configの現在値保持と、設定通知から不要なassistant streamを作らない修正も実装。新差分の統合検証待ち。
- GPUI最新testはsession42966、`tmp/gpui-runtime-retest.log`で実行中。終了状態を取得してから後続cargo gateへ進む。
- 原要件0001〜0017、4provider実経路、3host全操作、品質gate、公開確認は未完了。監査文書の旧native実装を指す記述は移行前の証拠であり、最新の達成判定に流用しない。

前turnの依存更新・共通treeは未コミット。既存経路の`just check`とcoverage79.09%は通過したが、純KUC提供の達成証拠ではない。共通runtime実装、既存要件監査、公開状態調査を分離して進行中。

- 共通runtime、egui/Floem/GPUIのOS・画像bridgeを分離して実装中。新egui描画をscreenshot実行経路へ接続済みだが未検証。
- Floemのみが持っていた履歴・file/diff/undoを`tools/host-support`へ抽出中。既存core contractのみを理由に「物理IO未実装」と誤判定しない。
- 公開対象をACP/core/egui/Floem/GPUIの5crateへ修正し、全crateの公開後可視性確認を追加した。shell構文と内部依存version検査はPASS。
- 公開可能にするためFloemをregistry 0.2.0へ切替中。必要な旧native API互換を直し、新KUC bridgeへ移行する。
- 一時的な容量不足に対処し、稼働buildなしを確認後に当repoの再生成可能`target/debug`だけをcargo cleanし、33GiBを確保した。今後は共通の`CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2`で検証する。
- ACP公式wire変換の12 unit testsはPASS（`tmp/acp-wire-tests.log`）。その後stdio transport追加中の統合buildは7 compile errorsで終了し、担当が修正中。transport完成や実agent通信の証拠ではない。
- egui公開`EguiChatView`をstatefulなKUC raster入口へ変更し、history/output等typed action callbackを追加。旧native modulesは公開入口から外したが、残存source整理と新経路gateは未完了。
- host-support新規sessionのConfigured/Missing維持、利用不可provider/model選択拒否を修正し回帰テストを追加。物理outputのcreate/edit/undo内容照合とsymlink境界も修正中。
- GPUI bridgeレビュー未対応P1: subview viewport/origin、render/dispatch error伝播、draftとsurface同期、実selectionとUTF16 IME範囲、candidate bounds、clipboard/scroll/focuslost、送信repeat抑止。描画実装があるだけでは0014達成としない。
- ACP stdio transportのcompile修正後、`cargo test -p katana-acp-client --lib`は16件PASS（`tmp/acp-transport-tests.log`）。実agentへのinitialize/session/prompt/cancelは未検証。ローカル実行可能CLIはcodex/claude/ollama/npx、copilotとACP adapter直接コマンドは未検出。
- `just check`へ`host-support-check`を追加し、共有hostのlint/実ファイルテストも必須化。`just --list --unsorted`でrecipe構文を確認済み。新egui/controllerとhost control回帰テストの実行はcore配置修正後に行う。
- 実`@agentclientprotocol/codex-acp@1.10.0`でprotocol1 initialize/sessionnewと同一session内の2 prompt/stream応答をPASS（`tmp/acp-real-handshake.log`, `tmp/acp-real-prompts.log`、example `acp_probe`）。回答前に診断textが届くため終端回答を検証。全provider/cancel/config/permissionの実通信はまだ未検証。
- manual-eguiは共有ChatHostStateへ接続。mock response/sample outputを通常経路から除外し、未接続送信はdraftを保持して失敗する。履歴読取エラーもexpectでpanicせず起動エラーとして返す。ACP/Ollama実executorのhost配線は残る。
- outputのcreate_new、編集原文照合、Appliedのみundo、undo時現内容照合、tmp symlink拒否を実装済み。実ファイルtests初回はcore visibility compile errorで停止（`tmp/host-support-tests.log`）；core担当修正済み、再実行待ち。
- egui新KUC経路は8 tests PASS（`tmp/egui-runtime-tests.log`）。実texture upload、RGB/alpha、IME cancel、CmdEnter repeat、typed history/output転送、過大viewport拒否を検証。GPUI画像の透明alphaをrootが修正し実RenderImage bytes回帰を追加、未実行。
- 共通coreは初回98 tests中92PASS/6FAIL。ScrollArea viewport未指定、composer可視labelによる位置ずれ、選択Cut/dragを修正済みと担当通知。最新差分での再実行が必要。
- Floem公開旧render経路を外しKUC mountへ移行、manual Floemとe2e callsiteも接続。既存実Ollama/Markdown host機能は維持。現在native2adapter `cargo check` はsession30032、ログ`tmp/native-kuc-check.log`で実行中。
- 共有hostの実ファイル・履歴・provider control tests 8件PASS（`tmp/host-support-tests.log`）。create/edit/undo原文照合、外部変更保護、tmp symlink拒否、履歴保存復元、新規draft分離を確認。追加のatomic/履歴JSON回帰は担当が追加中で別途検証。
- native check30032/51220は終了。Floem private import修正後はFloem errorsなし、GPUI27件の型/import/API errorsを修正中。rootはimage型importとRenderImage Frame配列化を修正済み。
- core再testも92/98 PASSで6FAIL（`tmp/kuc-runtime-tests.log`）。composer実幅がKUC built-inの132になっており、確定layoutと実frame/hit領域の整合が残る。未達をテスト期待値変更だけで処理しない。
- rootがcomposer入力slot高さ88、確定layout由来のTextSurface描画範囲とnodehit同期へ修正。coreは96/98PASSまで改善。残2件はagentカード幅と外側drag selection（kuc_boundary担当）。ログ`tmp/kuc-runtime-tests.log`。
- Floem headlessは3/4PASS（画像RGBA、mount、native pointer/IME→KUC effect）。refreshテストのみlayout未確定viewportによるMissingComposerBoundsを調査中（`tmp/floem-runtime-tests.log`）。session11331は結果確認済み、最終process exitのpollを次に行う。
- ACP permission型追加途中でagentが停止したためrootがVendorUiState→VendorUiSurface→ChatUiVendorBarSurface→KUC permission panel/RespondPermission actionまで結線した。modal click routingはkuc_boundaryに依頼、host bridgeの応答往復・model ID選択は未完。
