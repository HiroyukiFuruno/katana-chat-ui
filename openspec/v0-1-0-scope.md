# v0.1.0 新スコープ

この文書は、v0.1.0 を作り直すための単一の作業基準である。
旧 `openspec/changes/v0-*` は広すぎるため、`2026-05-09-superseded-*` として archive へ退避した。

2026-10-24の公開判定: PR #4の正式固定版と同じmerge SHA `64b29b421ec65bb8f1ff0bf54a595fbbb62d4c72` をActionsでv0.1.0として公開した。全5 registry artifactと独立consumer終了0、公開KUC0.4.4、3 adapter公開型compileと共通描画、専用Secret名の実診断を確認した。後続6 changeもmain specへ同期してarchive済み（main5・active0・strict5/5）。最終PR #5のreview/CI/mergeと後片付けは親台帳で継続する。詳細は [要件監査](../docs/release-requirements-audit.md) と `release-tasks.md` を参照する。

## 1. 目的

`katana-chat-ui` は、利用側が独自に AI エージェントチャット画面を作らなくても使える標準 UI を提供する。

v0.1.0 は「とりあえず会話できる」ではなく、次を満たす最小の実用品質に絞る。

- チャット指示で Markdown ファイルを生成できる。
- チャット指示で Markdown ファイルを編集できる。
- 生成、編集した変更を元に戻せる。
- 新規セッションを開始できる。
- 過去セッションを履歴から復元できる。
- 最後に開いていたセッションを再起動時に復元できる。
- provider と model を UI から切り替えられる。
- 同一セッション内で model を切り替えても会話を継続できる。

## 2. v0.1.0 に含めるもの

### 動作

- Markdown ファイルの生成。
- Markdown ファイルの編集。
- Markdown ファイルの生成、編集で発生した変更の取り消し。
- 生成、編集の結果を構造化 output として返す。
- `+` 操作による新規セッション開始。
- 履歴操作によるセッション一覧表示。
- セッション選択による復元。
- 最後のセッションの自動復元。

### UI / UX

- 下部固定の composer。
- `Enter` は改行。
- `Command + Enter` は送信。
- 送信中は送信ボタン位置を停止ボタンに切り替える。
- provider 選択はタイトル左側に置く。
- composer 内には model と provider ごとの可変 control を置く。
- context 使用率は composer 内の送信ボタン左側に円グラフで表示する。
- user message は右寄せで、本文量に応じた幅にする。
- agent response は composer とほぼ同じ幅に固定し、本文量で幅を変えない。
- thinking / execution / processing などの状態表示は本文幅ルールの対象外とする。
- Markdown 表示は code fence が途中で途切れないことを必須とする。

### provider

- 初期対応 provider は次の 4 つとする。
  - Codex CLI
  - Claude Code
  - GitHub Copilot
  - KatanAgent
- KatanAgent は `katana-chat-ui` 側が所有する agent 抽象である。
- KatanAgent の初期 runtime は Ollama を利用する。
- Ollama は provider selector に出さない。Ollama は KatanAgent の local model runtime として扱う。
- goose は KatanAgent core ではなく、拡張可能なら runtime adapter 候補として採用する。

## 3. v0.1.0 から外すもの

- 左端 hover の debug / harness panel。
- 右端 hover の output debug panel。
- 右上の設定ボタン。
- 設定画面。
- provider 接続詳細設定。
- secret 管理。
- provider usage / account usage。
- OS file picker。
- drag and drop 添付。
- clipboard 画像添付。
- 絵文字パレット対応。
- `/` command launcher。
- prompt / skill / workflow / command / hook / MCP の詳細 UI。
- file / diff / tool result の高度な詳細 view。
- 送信済み message 編集。
- 処理中の割り込み投稿。
- chat 内検索。
- 履歴横断検索。

## 4. 手動確認枠

- `harness-up` は標準 UI を載せるだけの枠にする。
- Floem、egui、GPUI は同じ標準 UI / UX を表示する。
- 完全な見た目一致までは必須にしないが、操作、配置、責務は同じにする。
- 手動確認枠は独自の会話表示、debug panel、output panel を作らない。
- screenshot 検証は headless を基本にする。
- native window を開く確認は最終確認だけにする。

## 5. 000X タスク

### 0001-archive-old-scope

旧 v0.1.0 から v0.6.0 の OpenSpec change を active から外す。

Status: 完了。2026-10-18に当時のchangeもarchiveし、その時点の一覧は空だった。後続のKUC互換・ACP修正など6 changeも、2026-10-24に公開・独立consumer確認後のmain spec同期とarchiveを完了した（active0）。旧v0-* changeのarchiveとは区別して証跡を保持する。

DoD:

- `openspec/changes/v0-*` が active に残っていない。
- archive 先が `openspec/changes/archive/2026-05-09-superseded-*` で残っている。
- `openspec list --json` で旧 change が active に出ない。

### 0002-remove-debug-and-settings-surfaces

左右 debug 機能と設定ボタンを削除する。

Status: 実装・検証完了。標準 UI には debug / settings / output hover を常設しない。

DoD:

- 左端 hover で harness / debug panel が出ない。
- 右端 hover で output debug panel が出ない。
- 右上に設定ボタンが出ない。
- 標準 UI の入力、送信、provider 切り替えを妨害する overlay がない。
- 回帰テストが debug / settings surface の再混入を検知する。

### 0003-provider-selector-header

provider 選択を title 左側の UI に固定する。

Status: 実装・検証完了。provider 候補は agent provider だけに絞り、Ollama は runtime として provider 候補から除外する。

DoD:

- composer 内に provider selector がない。
- title 左側に現在 provider と pulldown 表示がある。
- provider 候補は利用可能なものだけ出る。
- Ollama が provider 候補に出ない。

### 0004-composer-input-contract

composer の入力契約を安定化する。

Status: 自動回帰の初回範囲は完了。placeholder と draft の分離、Command+Enter 送信、送信後 draft clear、日本語IME rangeをEgui/Floem/GPUI native adapter回帰と2026-10-18 release-checkで固定した。一方、最新の人間指示に基づくエージェントの実IME追加受入は未完了で、専用macOS runnerとprivate Linux displayで追跡する。標準NSTextViewの実確定・取消は確認したが、KCU各hostとGPUI公開依存の残条件を[要件監査](../docs/release-requirements-audit.md)へ記録する。過去のconsumer integrationまでの延期を、今回の追加検証を省くwaiverとして流用しない。人間による手動受入は引き続き要求しない。

DoD:

- placeholder は入力値ではない。
- 日本語 IME 入力が壊れない。
- `Enter` は改行する。
- `Command + Enter` は 1 回だけ送信する。
- 送信後に draft が消え、session は消えない。

### 0005-composer-control-layout

composer 内 control の配置を固定する。

Status: 完了。context 使用率を送信ボタン左に置く順序、control の下揃え、SVG icon button の二重枠禁止を回帰テストで固定済み。

DoD:

- model selector と provider ごとの control が下揃えで並ぶ。
- context 使用率が送信ボタン左側に円グラフで表示される。
- SVG icon button に不要な二重枠が出ない。
- 横幅が狭くても入力欄が消えない。

### 0006-message-layout-contract

会話表示の幅と位置を固定する。

Status: 完了。user/agent の幅契約、上下 padding、thinking-only state を agent response 固定幅から除外する契約を回帰テストで固定済み。

DoD:

- user message は右寄せで、短文なら短い幅になる。
- user message の文字は上下中央に揃う。
- agent response は composer とほぼ同じ幅で表示される。
- agent response は本文量で幅が変わらない。
- thinking / execution / processing は agent response 幅ルールに含めない。

### 0007-provider-runtime-model

provider と runtime の境界を実装する。

Status: 実装・実受入完了。KatanAgentはKCU所有のprovider、Ollamaはruntime/model catalogとして扱い、利用不可providerを選択せずfallbackもしない。2026-10-23の4経路の実応答確認に加え、2026-10-24の正式`2160f356`（merge `64b29b42`と同一tree）でCodexの履歴・取消後の再送信、実Claude/KatanAgentの添付・保存output参照、KatanAgentモデルの再選択・切替復帰時のUI/保存一致が成功した。承認context・確定model・provider別文脈の恒久回帰と全品質gate、PR #4のmanual/Ready後reviewも完了した。公開工程は`release-tasks.md`で別途追跡する。Storybook fixtureの選択肢は実可用性の証拠にしない。

更新前のStatus（2026-10-24以前の履歴）: 実装済み・4経路の実接続確認済み。KatanAgent を kcu 所有の provider として追加し、Ollama は KatanAgent の runtime/model catalog として扱う。利用不可 provider は選択肢から除外し、fallback は行わない。2026-10-23にCodex・Copilot・KatanAgent・Claude Codeからlocal Ollamaの各実応答を確認した。最終reviewの承認context・model確定応答修正は `release-tasks.md` とactive changeで追跡する。Storybook fixtureの選択肢は実可用性の証拠にしない。

DoD:

- Codex CLI、Claude Code、GitHub Copilot、KatanAgent が provider 候補になる。
- KatanAgent は Ollama runtime を持てる。
- Ollama は provider ではなく runtime として扱われる。
- 利用不可 provider は選択できない。
- fallback で別 provider に切り替えない。

### 0008-katanagent-markdown-file-generation

KatanAgent で Markdown ファイル生成を扱う。

Status: 完了。Markdown 生成結果は chat 本文へ混ぜず、file candidate output と `CreateFile` action として host へ渡す契約を固定した。manual host 側は `cwd/tmp/` 配下だけ物理作成を許可し、許可外 path は拒否する。2026-09-07にはlocal Ollamaの実KatanAgentで候補生成と隔離`tmp/`への明示適用を確認した。

DoD:

- Markdown 作成指示から file candidate output が生成される。
- `./tmp` 配下など許可された場所だけを書き込み候補にできる。
- 許可外 path は失敗理由を返す。
- chat 本文と output contract が分離している。

### 0009-katanagent-markdown-file-editing

KatanAgent で Markdown ファイル編集を扱う。

Status: 実装・実受入完了。Markdown添付本文に加え、標準composerから明示した許可内の既存`tmp/*.md`本文もModify contextへ渡し、添付なしでdiff候補を生成できる。既存添付・Createの優先と物理tmp境界を維持し、outputは元ファイル、変更後本文、unified diff、概要を持つ。正式`2160f356`で実KatanAgentの添付なしCreate→Apply→Modify→ApplyDiff→Undoが終了コード0となり、画面に提示する操作の存在と実ファイル内容を確認した。全kind/status・保存payload・標準hostの実行可能操作だけを表示する回帰も成功した。

更新前のStatus（2026-10-24以前の履歴）: 完了。Markdown 添付本文を元内容として扱い、KatanAgent の変更後本文から diff candidate output を生成する契約を固定した。output は元ファイル、変更後内容、unified diff、概要を持つ。2026-09-07にはlocal Ollamaの実KatanAgentで、既存Markdown添付からdiff候補を生成して明示適用した。

DoD:

- Markdown 編集指示から diff または file update candidate が生成される。
- 元ファイル、変更後内容、差分概要を output に持てる。
- 物理書き込みは host / adapter 境界で行う。
- chat 本文に debug JSON を混ぜない。

### 0010-markdown-change-undo

Markdown 生成、編集で発生した変更を元に戻す。

Status: 完了。適用済みの file / diff output は `UndoChange` action を返し、manual host は `cwd/tmp/` 配下の生成ファイル削除と Markdown 編集内容の復元を行える。2026-09-07にはlocal Ollamaの実KatanAgentが生成したdiff候補を適用後、同一outputだけをUndoして元本文へ戻した。

DoD:

- 生成した Markdown ファイルを取り消し対象として記録できる。
- 編集した Markdown ファイルの変更前内容または逆差分を保持できる。
- 取り消し操作で対象変更だけを元に戻せる。
- 取り消し結果を output contract に持てる。
- 取り消しできない状態では理由を返し、別の変更へ fallback しない。

### 0011-session-history-store

セッション履歴を保存する。

Status: 完了。session snapshotを`./tmp/harness-${provider}`へ保存し、再起動したhostで最新sessionとmodelを復元する。2026-09-07には実Codex ACPで保存、再起動、履歴選択、NewChatとのdraft分離、元sessionの再選択、会話継続まで確認した。最新証拠は`release-tasks.md`を参照する。

DoD:

- session id、title、provider、model、message、output reference を保存できる。
- harness では `./tmp/harness-${provider}` 配下に保存される。
- 新規セッション開始で現在 session と draft が分離される。
- 再起動時に最後の session を復元できる。

### 0012-session-history-ui

履歴一覧と復元 UI を実装する。

Status: 完了。KUCの右上履歴actionとsession一覧・選択actionを共通surfaceへ描画する。2026-09-07には実Codex ACPでManaged履歴選択を通し、復元接続中の余分なpromptがなく、明示した3 promptだけが送信されたことをトレースメタデータで確認した。最新証拠は`release-tasks.md`を参照する。

DoD:

- 右上の履歴操作から session 一覧を開ける。
- 一覧に title、provider、updated at、preview が出る。
- session を選ぶと会話が復元される。
- 復元時に provider へ再送信しない。

### 0013-markdown-rendering-minimum

Markdown 表示の最低品質を満たす。

Status: 完了。Markdown ASTと共通KUC描画のunit / adapter conformanceに加え、長文MarkdownのKUC ScrollAreaが実content heightとclamp済みoffsetを使い、描画pixelが変化する回帰を追加した。`cargo test --locked -p katana-chat-ui markdown` は12件、`kuc::runtime::tests` は42件PASS。最新証拠は `release-tasks.md` を参照する。

DoD:

- code fence が途中の ``` で途切れない。
- 見出し、段落、list、code block が崩れない。
- raw HTML は描画せず text として扱う。
- renderer は将来差し替え可能な境界を持つ。

### 0014-three-host-ui-conformance

Floem、egui、GPUI の表示差を許容範囲に収める。

Status: 完了。3 hostの標準mountは共通KUC runtimeを使用し、adapter conformanceと5 scenario×3 hostの実画像一致を確認した（29316/85035）。IMEや実providerの要件は別番号で継続する。最新証拠は `release-tasks.md` を参照する。

DoD:

- 3 host が同じ render model を使う。
- 3 host で header、thread、composer の責務が同じである。
- 3 host screenshot の比較で、標準 UI として同じ操作対象に見える。
- host 側に独自 debug / output UI がない。

### 0015-headless-screenshot-gate

目視前の screenshot 検証を自動化する。

Status: 実装・再検証完了。公開KUC0.4.4の最終修正で完全release-checkが実終了コード0となり、新規native15/headless5画像のSHA・寸法・時刻、基準画像との一致、private Xvfb Storybook smokeと506 source不変をrootが照合した（`pr4-review15-host-output-actions-final-root-accepted.json`）。正式`2160f356`のCI全6jobとrelease-preflight全3jobも成功した。前面windowを開かず、画像差分と操作・配置の恒久回帰を維持している。

更新前のStatus（2026-10-24以前の履歴）: 実装済み・最新依存と起動経路の再検証待ち。Linux仮想display上で3 host×5画面の実画像・寸法・基準hashを必須検証し、通常checkへ追加した。過去の15件は85035で通過したが、公開KUC 0.4.4採用後の画像matrix、変更後のprivate Xvfb Storybook smoke、実CIと最終全体gateは未完了である。最新証拠は `release-tasks.md` を参照する。

DoD:

- headless screenshot で 3 host の主要レイアウトを検証する。
- native window を通常検証で開かない。
- screenshot 差分で composer 欠落、provider 誤表示、debug 混入を検知する。

### 0016-final-ui-polish

v0.1.0 の最後に UI 調整だけをまとめて行う。

Status: 実装・最終依存での再検証完了。正式`2160f356`のprivate Xvfbで入力・送信・scroll・履歴選択・provider切替を実操作し、9枚の新規画像のSHA・1280×900寸法・時刻と操作終了コード0を照合した。rootは末尾会話、履歴overlay、履歴選択後、provider menuを目視し、基準9画像との一致も確認した。配置・入力の恒久回帰、3 host matrix、完全gate、manual/Ready後review、CIも成功した。公開・consumer・archiveは完了し、最終PR #5のreview/CI/mergeと最終hygieneを親台帳で継続する。現在未利用の初回リリースではユーザー手動確認を要求せず、物理macOS IMEの既存waiverも維持する。

更新前のStatus（2026-10-24以前の履歴）: 実装済み・最終依存での再検証待ち。2026-10-18にagentが実Storybook artifactの5画面を目視し、共通KUCの配置契約、3 hostのheadless matrix、Linux native matrixを当時の依存で検証した。現在の公開KUC更新後の画像・native smoke・完全gateは `release-tasks.md` で追跡する。現在未利用の初回リリースではユーザー手動確認を要求しない。

DoD:

- button、spacing、text vertical align、message width を最終調整する。
- 既存の動作テストを壊さない。
- 人間の手動確認に出せる状態である。

### 0017-agent-activity-and-placeholder-contract

agent が今何をしているかを常に分かるようにし、composer placeholder を入力値から完全に分離する。

Status: 実装・自動検証完了。typed activity 7種、本文到着時の切替、完了・失敗時のclear、placeholder/IME分離を回帰で検証し、実KatanAgent/Ollamaでも`Processing`表示から完了時clearまで確認済み。

DoD:

- `thinking` は思考ログだけを表し、作業状態表示とは混ぜない。
- `thinking: false` でも、応答待ち中は `処理中` の状態表示が出る。
- 作業状態は `処理中`、`生成中`、`編集中`、`読み込み中`、`検索中`、`web検索中`、`実行中` を表せる。
- 作業状態は本文幅ルールに含めず、内容最小幅で表示する。
- 応答本文が届いたら、作業状態表示ではなく本文表示へ切り替わる。
- 完了、失敗時に作業状態が残らない。
- composer placeholder は editor の入力値ではなく、入力が空の時だけ見える overlay とする。
- placeholder 表示が cursor 位置、IME、日本語入力、`Command + Enter` 送信に影響しない。
- user message と agent response の上下 padding が同一契約で、文字が上寄せに見えない。

## 6. 完了判定

v0.1.0 は、0001 から 0017 までが完了し、次の検証が通った時だけ完了とする。

- `just fmt`
- `just lint`
- `just ast-lint`
- `just unit-test`
- `just host-e2e`
- `just harness-screenshot-matrix-check`
- `just harness-native-screenshot-check`

手動確認は通常最後に行う。現在未利用のv0.1.0初回リリースでは、ユーザー明示によりユーザー手動確認を省き、agentが実Storybook 5画面を目視した。物理macOS日本語IME受入は最初のconsumer integrationで実施する。
