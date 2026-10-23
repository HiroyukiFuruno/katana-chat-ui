# リリース要件監査

## 現在の判定（2026-10-24）

Draft PR #4の初回reviewでOllamaのUTF-8 chunk分割・stream内errorのP1と、ACP更新config ID不反映のP2を取得した。Ollama修正commit `7ff5c70` は旧版2ケース実失敗と現行各ケース成功、復元SHA一致を確認した。ACPはUI通知と受理IDの共有状態を更新し、削除済みIDの非致命拒否・session分離・通知先closed時不変も回帰へ追加した。ACP34件、既存lint・AST19・unit gateはexit0である。CIで検出した新Rustの冗長変換4件とLinuxリンク依存不足も修正した。

修正後の完全release-checkは実child exit0、502 source SHA不変、行coverage84.26%（基準77%）、既存5画像とLinux native15画像のSHA・寸法一致、Storybook smoke、package gateを確認した（`pr4-final-release-root-accepted.json`）。最新native対話・provider切替の実画像はroot目視済みで、現行binaryの実Codex履歴とKatanAgent/Ollamaもexit0。Codexの明示prompt3/new3/config2とcapture failure0をrootが独立照合した（`pr4-final-real-visual-root-accepted.json`）。修正後cloud reviewとCI、公開・downstream・archive・hygieneは引き続き未完了である。以下の旧成功gateは修正前の経緯として保持する。

公開KUC v0.4.4を直接manifest2箇所と全7lockfileへ採用した。独立workspaceで公開registryからの実downloadを確認し、全7構成の `update-safe` / `outdated` はexit0である。KUC #104/#105/#107は公開証拠を記録してclose済み。KCUではcore162/162、選択/copy/Arabic下線のfocused5件、strict lint・AST19・fmtが成功した（`tmp/verification/kuc044-final-gates.log`、`kuc044-seven-locks-root.json`）。

ACPの承認context・model応答・復元時確認待ち・回復可能な要求拒否の修正は、ACP21件、host84件、restore16件、strict lintと実Codex ACP履歴probeで確認した（`tmp/verification/acp-history-confirmed-real.log`、`acp-history-confirmed-summary.json`）。4providerの無料またはlocal構成での実接続証拠も保持し、追加契約を前提にしない。

旧native Rust source52件は、通常hook付き事前commit後にignored trashへ退避した。rootが全52 SHA、52 deletionだけの差分、使用中source・assets等136ファイル不変を照合した（`legacy-native-quarantine-root-accepted.json`）。整理後の完全release-checkもexit0で、499ファイルのhash不変を独立確認した。整理差分の正式保存は次工程である。

公開KUC0.4.4、Linux GPUIのlibc0.2.189互換制約、cache同時整理防止とsource祖先拒否を含む完全`just check`がexit0となった（`tmp/verification/kuc044-full-just-check-libc189.log`）。Git環境隔離修正と旧source整理後の完全release-checkもexit0である（`final-release-check-kuc044-cleaned.log`）。rootが499ファイルのhash不変、行coverage83.83%（基準77%）、既存5画像、Linux native15画像のSHA・寸法、Storybook実window smokeを独立確認した（`final-release-check-root-accepted.json`）。package検証はACP実buildとpublish dry-run、残4crateのpackage一覧、全5crate未公開確認であり、全5crateの実publish検証とは区別する。

最新binaryのprivate Xvfb実対話で入力・送信・固定応答・scroll・履歴選択を実行し、rootが実PNGを目視した。4provider候補とClaude CodeからCodex CLIへの切替も確認した（`storybook044-provenance.json`、`storybook044-provider-provenance.json`）。host前面操作・provider通信は行っていない。残るのは整理差分の正式保存、Draft PR・最低2回cloud review・CI・merge、5crate公開、独立registry consumer受入、OpenSpec archiveとbranch hygieneである。以下の過去記録を現在の完了証拠として扱わない。

## 2026-10-23 の更新（公開0.4.4採用前の記録）

全7lockへ公開KUC 0.4.3を採用し、ACP 3.5.0、egui/eframe/egui_kittest 0.37.0を使用している。120/320/560pxの本文幅・pointer境界、本文System描画不変、codeのmodel意味情報、5種の段落区切りcopyの局所回帰、および厳格lintが成功した。code専用macOS font候補とthinkingの高さ計算を修正し、5画像は既存hashと個別に一致した。agentは実画像も確認したが、これは完全gateの代替ではない。

追加した実consumer回帰で、soft wrap copyへのLF混入とArabic glyphへの4096px幅漏出を検出し、[KUC #104](https://github.com/HiroyukiFuruno/katana-ui-core/issues/104)・[#105](https://github.com/HiroyukiFuruno/katana-ui-core/issues/105)として追跡している。上流 `release/v0.4.4` の修正後は1059テストが通り、完全release-checkを実行中である。KCUは公開registry版だけを採用し、その後に全体gate・PRレビュー・公開・downstream受入を完了する。まだリリース完了ではない。

Codex ACP 2.2.2の同一sessionでの2応答と、更新後SDKでの履歴保存・新Host復元・明示3 prompt以外の再送信なしを実通信で確認した。同日の再build probeではCopilot・KatanAgentも成功し、Claude Agent ACP 0.94.0も専用config・通常default permission mode・ローカルqwen3:4bで厳密応答検査に成功した。追加契約・認証・download・global設定変更はない。証跡は `tmp/verification/{copilot,katanagent}-current-real.log`、`claude-local-default-mode-real.log`。承認操作のcontext破棄とmodel確定応答の破棄を最終reviewで検出したため、active change `acp-permission-model-confirmation` で修正・再検証する。残条件は `openspec/release-tasks.md` とactive changeを参照する。

以下は2026-10-18時点の監査記録である。

監査日: 2026-10-18。判定対象は `openspec/v0-1-0-scope.md` の0001〜0017、archive済みchange `2026-10-18-v0-2-0-pure-kuc-multivendor-acp`、および「純KUC・multi-vendor・ACP影響確認・依存更新・リリースまで」という最新指示である。OpenSpec の `Status` や tasks のチェックだけで判定せず、現行コード、実provider probe、品質gateを直接照合した。

## 結論

純KUC・multi-vendor・ACP transportの実装と、KUC 0.3.7時点の直接品質gateは揃った。0004の物理macOS日本語IME受入は、現在未利用の初回リリースではユーザーが明示的に延期した。multi-vendor runtimeは既存認証または導入済みモデルを使える場合だけproviderを利用可能にし、認証・課金・Ollamaモデル取得を自動実行しない。2026-09の4経路成功は過去の接続証拠であり、現在の可用性を示すものではない。KUC 0.4.1とACP 3.4.0への依存更新では、KUCの公開Card APIが既存のmessage幅契約と5画面基準を両立できず、最終release-checkがscreenshot matrixで停止した。したがってv0.1.0はPR・CI・公開artifactへ進めず、[katana-ui-core#83](https://github.com/HiroyukiFuruno/katana-ui-core/issues/83) の公開修正版を待つ。egui 0.37.0は採用済みだが、KUC互換版と組み合わせた最終release gateは未実行である。

## 2026-10-18時点の検証状態（過去記録）

| ID | 現在の判定 | 直接証拠と残条件 |
| --- | --- | --- |
| 0001 | 達成 | 旧v0.1.0〜v0.6.0に加え現changeを`openspec/changes/archive/2026-10-18-v0-2-0-pure-kuc-multivendor-acp/`へarchiveした。2026-10-18時点の一覧は空で、旧`v0-*`はactiveにない。現在は後続のKUC互換・ACP修正のactive changeを別途追跡する。 |
| 0004 | 達成（初回リリース） | native adapter回帰と2026-10-18 release-checkで日本語IME range、Enter改行、Cmd+Enter一回送信、draft clearを検証した。物理macOS日本語IME受入は、現在未利用の初回リリースではユーザー明示により最初のconsumer integrationへ延期する。 |
| 0007 | 実装済み・現行実接続は検証待ち | 2026-09にはCodex ACP、Claude Code adapterからlocal Ollama `qwen3:4b`、Copilot Free ACP、KatanAgentからdirect Ollamaの4経路でcatalog、session、prompt、完了応答を確認した。現在はClaude認証なし、Copilot CLIなし、Ollama catalog空であるため、各providerは利用可能一覧へ入らない。既存認証の再設定、課金・trial、Ollamaモデルの自動取得は行わず、導入済み環境での再接続だけを将来の検証条件とする。 |
| 0008〜0010 | 達成 | 実KatanAgent/OllamaがMarkdownを生成・添付編集し、tmp内のCreateFile/ApplyDiff/UndoChangeで物理ファイルを作成・復元する `katanagent_output_probe` が成功。 |
| 0011・0012 | 達成 | 実Codex ACPで保存、新Host復元、Managed SelectHistory、model保持、履歴会話継続を確認。JSON-RPCメタデータは明示3 promptだけで、履歴選択による再送信は0件。 |
| 0013 | 達成 | 構造化Markdown/生HTML文字列/code fenceの12回帰、KUC ScrollAreaの長文pixel scrollを含むruntime 42回帰が成功。 |
| 0014・0015 | 達成 | 3 hostがKUC runtimeを使うE2E conformance 8件とhost flow 1件、headless matrix 5 scenario、Linux/aarch64 Xvfb native matrixが成功。 |
| 0016 | 再検証待ち | KUC 0.3.7時点では3 manual host gate、headless matrix、Linux native matrix、agentによるStorybook 5画面目視を完了した。KUC 0.4.1ではCard child indentにより既存画像基準とwide幅契約が衝突するため、KUC #83の公開修正版後に同じ画像・native検証を再実行する。 |
| 0017 | 達成 | typed activity 7種、本文到着時の切替、完了/失敗時clear、placeholder/IME分離の回帰に加え、実KatanAgentが`Processing`を表示して完了時にclearすることを確認。 |

- KUC 0.3.7時点の`rtk proxy env CARGO_INCREMENTAL=0 just VERSION=v0.1.0 release-check`は成功し、fmt、strict clippy、workspace/native/E2E、manual Egui 7・Floem 53・GPUI 6、Storybook 6、5画面screenshot matrix、coverage 81.85%（基準77%）、package/publish dry-run、Linux Docker matrix、未公開crate確認を含んだ。これはKUC 0.4.1移行後の最終証拠ではない。
- KUC 0.4.1とACP 3.4.0を公開registryから全7 lockfileへ更新し、strict clippy、workspace/native/E2E、manual host、Storybook smokeまでは成功した。最終release-checkの5画面matrixでは、Card child indentによる既存基準不一致を検出した。KCUはbaselineを更新せず、KUC #83の公開修正版を待つ。egui/eframe/egui_kittest 0.37.0は公開registryから採用し、Egui adapter 13件、manual Egui host 7件、Storybook 6件、screenshot executor 12件、native smokeを通過した。KUC互換版と組み合わせた画像・release gateは未実行である。
- ACP仕様影響として、child process単位の環境引渡しを追加し、Claudeのlocal Ollama profileが親processへAnthropic認証情報を残さないことを確認した。
- 現在のprovider可用性は過去probeと分離する。ClaudeのOllama互換profileは`KCU_CLAUDE_OLLAMA_MODEL`を明示した子processだけに渡り、Copilot/Claudeは`initialize`と`session/new`後のReady投影まで選択肢に入らない。Ollamaは`/api/tags`が空なら`Ollama model catalog is empty`として利用不能となり、`/api/pull`や`ollama pull`を実行しない。

## 移行後の経過（過去の証拠）

以下は過去時点の記録であり、現在の完了判定には上記と `openspec/release-tasks.md` を優先する。

- 0015の更新: Linux/aarch64の実egui/Floem/GPUIで5 scenarioずつ、計15画像の寸法・基準SHA256一致を必須検証する。`just harness-native-linux-matrix-check`再実行6980 exit0、summary status=passed。誤hashの実captureはexit1、baseline契約unit4件PASS。通常checkとCI jobへの組込みは実装済みだが、CI実行・required check設定および最新全体gateは未確認。下記の単一画像のみという記録はこの証拠で更新する。
- 0004/0017の追加P1: 本文到着時のactivity非表示、Floem Cmd+Enter repeat抑制、GPUI非原点のpointer/IME座標を修正した。全workspace179 tests PASSとstrict lint PASS（`tmp/p1-unit-current.log`、`tmp/p1-lint-current.log`）。GPUIは実TestWindowを(48,36)へ配置した回帰を含む。4providerの実接続やOS日本語IME全体の証拠には拡大しない。
- 0015: Linux Xvfb上の標準egui mountから実1280x900画像を取得し、header/composer等を目視確認した（`tmp/native-linux/egui/native-egui.png`）。初回の単色起動frameを検出して再試行する。空チャット1件のartifact段階であり、3host matrix・狭幅・画像差分gateは未完了。
- 純KUC境界: 3adapterの現 `src/lib.rs` はraster entryだけをmodule化・公開する。旧native widgetは通常compile/public APIに含まれない。E2E conformanceは共通KUCの実frame/dispatch検査へ移行済みだが、3host固有の画像証拠とは区別する。
- 0008〜0012: 共通 `tools/host-support` のstrict clippyと58 testsが通過（`tmp/host-restore-current.log`）。tmp内の生成・編集・undo、外部編集保持、失敗理由永続化、session境界を含む。実KatanAgent入力から物理outputまでの結合は未検証。
- 0011・ACP: 実Codex3 promptが通過（`tmp/acp-history-real.log`）。保存・新Hostへ再読込した後、質問文に含まない過去tokenを回答でき、その後も会話を継続した。全provider/全host UIの復元証拠には拡大しない。
- 0004・0014: 依存更新前に3adapter unitと3manual host個別gateが通過。egui 0.36.1更新後の全workspace170 testsとhost E2E9 testsが通過（`tmp/narrow-unit-e2e-current.log`）。manual各hostの更新後全gateは残る。
- 0003・0005・0016: 高DPI二重scaleとdropdown経路は修正済み。固定usage hitとdropdown描画/hitの縦送り不整合を修正し、core115 testsとE2E9 testsが通過。画像baselineは更新していない。
- 0007: Codexは実通信の証拠あり。直近probeではClaudeはOAuth期限切れ、Copilot CLI不在、Ollama catalogは空であり、全4provider実会話は未達。
- 依存更新: egui/eframe/egui_kittest 0.36.1のmanifest/API/3lock更新完了、全unit170/E2E9 tests通過。
- リリースgate: 最新の全workspace check/coverage/release-checkと3host画像品質は未通過。実行中handleと最新ログは `openspec/release-tasks.md` を参照する。

## 移行着手時の0001〜0017の照合（旧実装の調査記録）

以下の表は移行前のnative widget経路の記録である。最新実装の判定には上記の証拠と台帳を用い、未達項目は直接検証が揃うまで完了にしない。

| ID | 要件 | 実装場所と直接証拠 | 判定・未対応 |
| --- | --- | --- | --- |
| 0001 | 旧 v0 change を archive へ移し active から外す | `openspec/changes/archive/` は存在する。一方、active に `openspec/changes/v0-2-0-pure-kuc-multivendor-acp/` がある。 | **要確認**。DoD の文字どおりの `openspec/changes/v0-*` 不在には反する。旧 change を指すなら `openspec list --json` の結果で区別する証拠が必要。 |
| 0002 | debug/settings/output hover を標準 UI から除く | `crates/katana-chat-ui/src/session/render_ops.rs` は settings render model を依然構築する。標準 host の表層に常設されないことを検査するテストはある。 | **部分的**。core に設定 surface が残ること自体は不合格理由ではないが、3 host の実操作・overlay 不在の直接検証がない。 |
| 0003 | header 左の利用可能 provider 選択、Ollama 非表示 | `ChatUiSurface` と vendor state、egui/Floem の header は候補を描画する。GPUI `src/view/header.rs` は表示用 `Div` で、選択 event を持たない。 | **部分的**。候補のデータ契約はあるが、CLI 利用可否の probe がなく、GPUI を含む3 hostで選択できない。 |
| 0004 | IME、Enter 改行、Cmd+Enter 一回送信、draft clear | Floem `src/widget/composer/editor.rs` は native `TextEditor` と Cmd+Enter handler を使う。egui は `TextEdit::multiline`。GPUI `src/view/composer.rs` は `String` を表示するだけ。 | **未達**。Floem の単体・ソース検査はあるが、日本語 IME を含む3 hostの入力契約を満たさない。 |
| 0005 | composer control の配置 | 各 host に composer 描画と配置テストがある。 | **部分的**。静的描画はあるが、共通 KUC 実装でなく、GPUI は操作不能。狭幅・実入力を3 hostで確認していない。 |
| 0006 | message 幅・位置・padding 契約 | Floem `src/widget/thread_layout.rs`、egui `src/message.rs`、GPUI `src/view/thread.rs` に個別実装がある。 | **部分的**。各実装が別で、同一責務の実動比較がない。 |
| 0007 | Codex/Claude/Copilot/KatanAgent、Ollama runtime、不可 provider 除外、fallback なし | manual Floem `src/provider.rs` は PATH/`gh` で CLI を検出し、KatanAgent は Ollama model catalog を問い合わせる。検出した Claude/Codex/Copilot/OpenCode は `Unavailable` とし、実行可能なのは `KatanAgentOllamaAgent` だけである。 | **部分的**。Ollama runtime、不可 provider 除外、fallback なしは根拠あり。Codex/Claude/Copilot は実行経路を持たず、multi-vendor の利用可能 provider 要件は未達。core `AcpProcess` は process 設定型だけで実行しない。 |
| 0008 | Markdown 生成と許可 path の物理作成 | core `src/output.rs` は意図だけを返す。host `tools/manual-host-floem/src/state.rs` の `ManualFileCreateAction::apply` は `tmp` 内に `fs::write` し、外部 path を拒否する。state test は生成後の実ファイルを検査する。 | **Floem host では達成**。KatanAgent+Ollama の実行から candidate を受ける経路がある。ただし実 agent/実 transport fixture による生成→保存の結合試験はなく、egui/GPUI には同じ host service がない。 |
| 0009 | Markdown 編集、元/後/diff/概要、host physical write | `tools/manual-host-floem/src/provider/markdown_ops.rs` が添付 Markdown から diff candidate を作り、`ManualDiffApplyAction::apply` が `updated_content` を `fs::write` する。state test は内容が `after` になることを検査する。 | **Floem host では達成**。core intent と host I/O の責任分離は scope に一致する。実 agent 結合試験と egui/GPUI 共有実装がない。 |
| 0010 | 生成・編集の対象限定 undo | `ManualFileCreateAction::undo` は対象 file を削除し、`ManualDiffApplyAction::undo` は original content を書き戻す。state の2 test が実 filesystem と `OutputStatus::Reverted` を検査する。 | **Floem host では達成**。対象 output の復元は確認できる。一方で共通 host service でなく、egui/GPUI は action executor を持たない。 |
| 0011 | session id を含む履歴永続化、last session 復元 | `tools/manual-host-floem/src/history.rs` は `tmp/harness-${provider}/sessions/<id>.json` と `last-session.json` を書く。`state.rs` は起動時 `load_latest`、session id と snapshot を復元する。history tests と state の再生成 test がある。 | **Floem harness では達成**。core snapshot が I/O を持たないのは責任分離として正しい。egui/GPUI には同等 service がなく、3 host 共通の要件としては未達。 |
| 0012 | 履歴一覧、選択復元、再送信なし | Floem `history.rs` と `state.rs::open_history/restore_history_session`、`widget/history.rs` の callback。state test は保存済み session の一覧表示・選択復元・停止状態を検査する。 | **Floem host では達成**。復元関数は snapshot を直接反映し provider job を作らない。egui は `open_history` が表示状態を持たず、GPUI は static view のため、multi-vendor共通の履歴機能は未達。 |
| 0013 | Markdown 最低表示、raw HTML は text、差替え境界 | `src/markdown.rs` と parser tests は構造化 block と code fence を保持する。KUC `src/kuc/markdown.rs` は UiTree 写像を持つ。 | **部分的**。構文・KUC tree は根拠があるが、KUC tree は host 未接続。3 native renderer の実表示同等性、raw HTML の最終描画を確認する実 host test がない。 |
| 0014 | Floem/egui/GPUI の同一 model・責務・操作対象 | 全 host は `ChatUiSurface` を受け取る。しかし Floem は native editor、egui は `TextEdit`、GPUI は静的 `Div` を別々に実装する。 | **未達（Critical）**。同一 render/action KUC 経路ではなく、GPUI の入力・action がない。 |
| 0015 | headless で3 host、composer/provider/debug を検知 | `scripts/screenshot/src/executor_harness.rs` は `EguiChatView` と `egui_kittest::Harness` だけを使う。`justfile` の `harness-native-screenshot-check` は同 headless matrix に委譲する。 | **未達（Critical）**。5 scenario の hash は egui のみ。3 host を検査せず、native 3 host は opt-in visible check で通常 gate 外。 |
| 0016 | 最終 polish と手動確認可能状態 | egui screenshot baseline と個別 adapter tests はある。 | **未達**。0014・0015 と実操作の未達により、人間の手動確認へ出せる共通 UI の証拠にならない。 |
| 0017 | thinking/activity 分離、7 activity、本文到着時切替、clear、placeholder/IME/padding | `src/session/stream_ops.rs` は開始時 Processing、finish/fail 時 clear を実装。`AgentActivityKind` は7種類。Floem `thread_layout.rs` は state を content-sized にする。 | **部分的**。core state test はあるが、activity source は注入 event のみ。GPUI placeholder は入力値として文字列表示し、3 hostで overlay/IME/Cmd+Enter を満たさない。scope 自身も `実装中`。 |

## 移行着手時のactive change照合（旧実装の調査記録）

`openspec/changes/v0-2-0-pure-kuc-multivendor-acp/tasks.md` は、KUC UiTree/action mapper と依存更新、ACP 境界文書を完了としている一方で、次を未完了にしている。

- 「KUCのみを使用する共通UI」
- egui/Floem/GPUI の共通 UI への接続
- 入力、送信、cancel、history、vendor 選択、output の既存契約維持
- OpenSpec 照合で Critical がないこと

これは現行実装とも一致する。`src/kuc.rs` の `ChatUiTreeBuilder` と `ChatUiActionMapper` は存在するが、3 native adapter は KUC tree を描画・操作していない。したがって純 KUC / multi-vendor は未完了であり、UiTree の生成成功を移行完了と扱えない。

ACP は core `agent.rs` では process configuration と注入用 `ChatAgentEvent` までである。manual Floem は `katana-acp-client` の `OllamaProvider` を直接使い、KatanAgent の streaming を実行するが、Codex/Claude/Copilot の ACP process spawn、JSON-RPC framing、initialize、session/update、prompt、cancel、error mapping はない。今回 ACP 通信を新設しない方針は適切だが、CLI provider を「利用できる」と主張する根拠には使えない。将来の結合検証は mock IT ではなく、実 agent と実 transport fixture subprocess で行う必要がある。

## Non-Goals が既存 P0 を消していないか

過去の広い P0 が含んでいた settings 等は、`v0-1-0-scope.md` 自身が単一の作業基準として明示的に対象外へ再定義している。従って active change の Non-Goals がそれらを新たに消したわけではない。

一方、active change の Non-Goals を 0001〜0017 の未達を除外する根拠にはできない。特に provider runtime、Markdown の物理生成・編集・undo、履歴、3 host、headless3 host は v0.1 scope の P0 相当の完了条件である。active change の「release は対象外」という旧記述も、最新の「リリースまで」指示により優先されない。

## 移行着手時の実装分担案（履歴）

1. **共通 non-UI host service**: manual Floem の `ManualHistoryStore`、output executor、provider registry/execution を framework 非依存 crate/module に切り出す。session snapshot と host I/O の責任分離を維持したまま、egui/GPUI に同じ永続化・path policy・undo・provider可否判定を提供する。
2. **実 provider fixture**: KatanAgent+Ollama の生成→保存、添付編集→復元、再起動復元を実 transport subprocess で検証する。CLI provider を候補に残すなら、各 CLI の probe と実行/ACP境界も同じ方式で検証する。
3. **KUC common renderer と host bridge**: UiTree と typed action を実際の描画・OS event 経路へ接続し、各 host の独自 chat widget を置換する。IME/clipboard/focus は host が event bridge を担当する。
4. **3 host screenshot gate**: common renderer 接続後に、headless 各 host の同一 scenario を取得し composer/provider/debug を構造・画像両面で検査する。現在の egui-only hash は代替にならない。

agent CLI probe・process/ACP transport・イベント変換は、Floem の KatanAgent+Ollama host path と独立に実装できる。ただし実通信を今回の ACP 非新設方針で保留するなら、Codex/Claude/Copilot を利用可能 provider と表示せず、0007 の multi-vendor達成とは扱わないことが必要である。

## Floem 旧 host テストの移行対応（2026-09-05）

標準バイナリは `tools/manual-host-floem/src/main.rs` から `ManagedChatHost` と KUC を mount する。旧 `state.rs`、`provider.rs`、`history.rs` は `#[cfg(test)]` で既存回帰を維持し、標準実行入口には含めない。下表は旧テスト名を実在する共有実装または新経路のテスト名へ対応付けた。対応先がないものは未検証とする。

| 旧 Floem テスト | 新しい直接証拠 | 判定 |
| --- | --- | --- |
| `history_store_restores_latest_completed_session`、`history_ui_opens_saved_sessions_and_restores_selected_session` | `tools/host-support/src/tests.rs::typed_actions_persist_restore_and_select_history` | 検証済み |
| `manual_file_create_allows_only_tmp_under_cwd`、`manual_file_create_can_undo_created_tmp_file`、`manual_file_create_rejects_parent_path`、`manual_diff_apply_can_restore_original_content` | `tools/host-support/src/tests.rs::file_and_diff_actions_write_and_undo_only_under_tmp`、`tools/host-support/src/output/tests.rs::{create_new_and_undo_preserve_existing_or_changed_files,diff_requires_original_content_and_restores_only_its_applied_edit,symlink_under_tmp_cannot_escape_the_output_root}`、`tools/host-support/src/output/atomic_tests.rs::{atomic_create_does_not_clobber_an_existing_file,atomic_edit_keeps_the_existing_file_permissions}` | 検証済み |
| `select_vendor_rejects_unavailable_provider`、`empty_provider_registry_exposes_no_vendor_selector_options` | `tools/host-support/src/provider_manager/inventory.rs::only_ready_slots_are_available_for_selection` | 検証済み |
| `stopped_provider_result_is_not_applied_to_next_message` | `tools/host-support/src/provider_manager/routing.rs::non_active_permission_is_cancelled_instead_of_reaching_the_chat`、`tools/host-support/src/provider_manager/commands.rs::active_stream_or_permission_blocks_provider_switch` | 部分的。停止済み stream 本文の直接回帰は未検証 |
| `katanagent_agent_provider_keeps_ollama_as_runtime_model_label` | `tools/host-support/src/provider_manager/katan_poll.rs::direct_catalog_models_do_not_select_a_model_implicitly` | 部分的。実 catalog のラベル反映は未検証 |
| `selected_file_can_be_attached_and_removed`、`selected_file_attach_preserves_live_draft_text` | `tools/manual-host-floem/src/managed_state/tests.rs::attachment_keeps_the_file_uri`、`stale_draft_ack_keeps_the_newer_composer_text` | 部分的。添付の削除操作は共有 actor の直接回帰が未検証 |
| 新しい actor 同期境界 | `update_plan_preserves_error_and_effect_without_clearing_them`、`update_drain_stops_after_the_tick_limit`、`scheduling_stops_when_the_host_is_dropped` | manual Floem strict clippy/check/51 tests PASS（egui依存更新前） |
| `provider_events_finish_streaming_state`、`provider_event_marks_error_state`、`provider_output_is_exposed_as_handoff_not_thread_body` | 対応する共有 actor の直接テストは未確認 | 未検証 |
| mock 応答、synthetic provider dump、旧 Markdown candidate 生成 | mock は純 KUC/共有 host の根拠に使わない。実 transport から構造化 output への結合回帰は未実装 | 未検証 |

`ManagedChatHost` または Floem bridge を変更した場合は、共有 actor の check と標準 Floem 経路の回帰を再実行する。実 Markdown structured output の transport 結合は、この対応表で完了扱いにしない。
