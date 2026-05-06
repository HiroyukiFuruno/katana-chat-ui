# Tasks: v0.1.0 — chat UX foundation

> kcu を framework-neutral core と標準 UI 提供 crate の組として固定し、標準的な AI chat 入力と表示を実装する。API-only は customization path であり、完了条件ではない。

> Status reset: 現在の実装や過去の完了マークは、`docs/chat-ui-spec.ja.md` への適合証明として扱わない。実装は仕様整合後に改めて検証し、合格条件を満たした項目だけを完了扱いにする。

## Branch Rule

本 change では、以下のブランチ運用を適用する。

- 統合ブランチ: `v0-1-0-chat-widget-floem`
- task ブランチ: `v0-1-0-chat-widget-floem-task<N>`
- feedback ブランチ: `v0-1-0-chat-widget-floem-feedback`

base branch を固定で決めず、PR 作成時に `/create_pull_request` で確認する。

---

## 0. Foundation Cleanup

### Definition of Ready

- [x] `git status --short` で既存差分を確認している
- [x] active OpenSpec がこの change を source of truth とすることを確認している

### Definition of Done

- [x] `Cargo.toml` と `crates/katana-chat-ui` から `egui` dependency と `egui::` public API が消えている
- [x] README が特定 host app / egui host 前提ではなく、host-agnostic な説明になっている
- [x] `rg -n "egui::|<<KATANA" crates/katana-chat-ui README.md` が一致しない
- [x] `/self-review` を実行し、指摘を解消している
- [x] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [x] ユーザーへ結果を報告し、コミット前に停止している

---

## 1. Framework-neutral Chat Core

### Definition of Ready

- [x] Task 0 の実装、自己レビュー、検証、報告が完了している
- [x] base branch が最新で、今回の書き込み範囲が明確になっている
- [x] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [x] `ChatSession` / `ChatMessage` / `MessageRole` / `MessageStatus` / `ChatRenderModel` を実装している
- [x] user / assistant / tool / system を視覚差分として表現できる render model がある
- [x] streaming 中、送信不可、provider 未設定、エラーの state を持つ
- [x] unit test が turn 管理と role 表示 contract を検証している
- [x] `cargo tree -p katana-chat-ui | grep -E "floem|egui|vello|eframe"` が空である
- [x] `/self-review` を実行し、指摘を解消している
- [x] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [x] ユーザーへ結果を報告し、コミット前に停止している

---

## 2. Input, Attachment, and Markdown

### Definition of Ready

- [x] Task 1 の実装、自己レビュー、検証、報告が完了している
- [x] base branch が最新で、今回の書き込み範囲が明確になっている
- [x] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [x] `ChatInputDraft` が multiline text、attachment tray、submit / cancel intent を持つ
- [x] file attachment、image attachment、path drop を `Attachment` として表現している
- [x] path drop は host callback で読み取り、kcu core が勝手に filesystem を読まない
- [x] ACP の text / image / embedded resource content block へ変換できる contract がある
- [x] Markdown subset は `comrak` を暫定採用し、見出し、段落、強調、取り消し線、link、自動 link、inline code、code block、blockquote、通常 list、番号付き list、task list、table を扱う
- [x] 画像 Markdown は画像描画せず安全な alt/link 表示にし、raw HTML は text として扱う
- [x] HTML block、script、raw style を描画しない test がある
- [x] `/self-review` を実行し、指摘を解消している
- [x] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [x] ユーザーへ結果を報告し、コミット前に停止している

---

## 3. Theme, SVG Icons, and Usage Surface

### Definition of Ready

- [x] Task 2 の実装、自己レビュー、検証、報告が完了している
- [x] base branch が最新で、今回の書き込み範囲が明確になっている
- [x] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [x] `ThemeTokens` が color / spacing / typography / status color を受け取れる
- [x] button icon はすべて SVG asset id で表現され、host override できる
- [x] context token usage を used / size / percentage / status として表示できる
- [x] account usage は取得できない場合も `Unavailable(reason)` として表示状態を持つ
- [x] unit test が theme merge、icon override、usage percentage を検証している
- [x] `/self-review` を実行し、指摘を解消している
- [x] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [x] ユーザーへ結果を報告し、コミット前に停止している

---

## 4. Floem Standard UI Implementation

### Definition of Ready

- [x] Task 3 の実装、自己レビュー、検証、報告が完了している
- [x] base branch が最新で、今回の書き込み範囲が明確になっている
- [x] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [x] `katana-chat-ui-floem` が API descriptor だけでなく、実際に描画できる標準 Floem chat widget を提供している
- [x] 標準 widget は message list、IME 対応 composer、attachment tray、usage meter、settings trigger、stop / send action を表示する
- [x] button は `ChatIconSet` の SVG を描画し、runtime override 後の SVG を使う
- [x] UI 文言は text catalog を通し、MVP が英語のみでも locale 追加ができる
- [x] vendor / provider profile と capability に応じて model、mode、thinking、permission などの UI affordance を出し分ける
- [x] output は widget 内で file 書き込みや diff 適用 UI として所有せず、host へ渡る data / action intent として扱う
- [x] Floem crate にも `egui` / `eframe` / host app 固有 dependency がない
- [x] 標準 UI は E2E harness に依存せず、通常 crate として利用できる
- [x] smoke test は標準 UI crate 内の最小範囲に留めている
- [x] `/self-review` を実行し、指摘を解消している
- [x] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [x] ユーザーへ結果を報告し、コミット前に停止している

---

## 4.5 Output Handoff Contract

### Definition of Ready

- [x] Task 4 の標準 UI 実装方針が確定している
- [x] 生成物や差分を chat-ui が所有せず、host へ渡す方針を設計に反映している
- [x] Zed の Agent Panel / Prompt Editor / UI component pattern を参考情報として確認している

### Definition of Done

- [x] `ChatOutput` / `ChatOutputKind` / `HostActionIntent` を実装している
- [x] text / code / file candidate / diff candidate / tool result / permission request を区別できる
- [x] file 書き込み、diff 適用、tool 実行、権限承認を kcu core が実行しない
- [x] render model が output list と host action intent を返す
- [x] unit test が file candidate、diff candidate、permission request の host handoff を検証している
- [x] 標準 UI の外側で output handoff の JSON / data structure を目視確認できる
- [x] `/self-review` を実行し、指摘を解消している
- [x] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [x] ユーザーへ結果を報告し、コミット前に停止している

---

## 5. External Host E2E Verification

> この task は Final Verification の直前に実施する。kcu を実際に取り込む外部 host 視点の確認であり、`crates/` 配下に検証用 app を置かない。

### Definition of Ready

- [x] Task 4 の実装、自己レビュー、検証、報告が完了している
- [x] base branch が最新で、今回の書き込み範囲が明確になっている
- [x] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [x] `tools/e2e-host-app/` のような非公開 host fixture がある
- [x] fixture は workspace member ではなく、配布対象 crate に含まれない
- [x] fixture は kcu を downstream dependency として実際に取り込む
- [x] E2E は kcu 標準 UI surface と Floem widget construct、入力、添付、path drop、send、stop、usage 表示を検証する
- [x] E2E 結果とスクリーンショットまたは実行ログをユーザーに提示できる
- [x] `/self-review` を実行し、指摘を解消している
- [x] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [x] ユーザーへ結果を報告し、コミット前に停止している

---

## 6. User Review and Feedback Tasking

> ユーザーレビューの指摘は `[/]` で閉じる。通常 task の `[x]` と混ぜない。

- [x] 6.1 Task 5 の E2E 結果、スクリーンショットまたは実行ログ、残課題をユーザーへ提示する
- [x] 6.2 ユーザーからのフィードバックを、この section の `User Feedback Tasks` に `[ ]` で task 化する
- [x] 6.3 defer 指定がないフィードバックをすべて解消する
- [x] 6.4 解消済みフィードバックは `[/]` に更新し、通常 task の `[x]` と混ぜない
- [/] 6.5 実動作検証用 UI harness は `crates/` に含めず、外部 host fixture として kcu を実際に取り込む E2E にする

### User Feedback Tasks

- [/] 外部 host E2E を v0.1.0 の計画に含め、Final Verification 直前に実施する
- [/] E2E 結果をユーザーに提示し、ユーザー FB を tasks.md に task 化する構成にする
- [/] UI/UX は自動検証だけでは不十分なため、人間が起動して入力・添付・送信・停止・usage 表示を触れる手動確認用 UI host を用意する
- [/] 手動確認用 UI host は egui / Floem / GPUI の3種類の取り込み側 MVP として起動できる
- [/] MVP でも output contract は必須のため、生成物・差分・tool result・permission request を host に渡す contract を v0.1.0 に含める
- [/] chat-ui 全般の設計で Zed の Agent Panel / Prompt Editor / GPUI component pattern を参考にする
- [/] 手動 LLM 検証は local Ollama を使い、自動検証では有料 LLM API token を消費しない
- [/] `katana-chat-ui` は UI 込みで提供する。API-only 実装だけを v0.1.0 完了扱いしない
- [/] API-only 利用は標準 UI に満足できない利用者向けの customization path として扱う
- [/] manual host は標準 UI を載せる枠であり、host 側で独自 chat UI を作らない（v0.1.0 の人間確認対象は egui / Floem / GPUI すべて同等）
- [/] button は SVG で表示し、input された SVG override を runtime で反映できる
- [/] UI 文言は kcu 側の text catalog を通し、katana 対応 locale と override JSON を受け取れる
- [/] vendor ごとの UI affordance を vendor profile と capability で出し分ける
- [/] vendor ごとの UI 表示/非表示は公式ドキュメント根拠を持つ `VendorFactRegistry` から決める
- [/] Ollama は endpoint / model / thinking / tools / token usage を表示し、公式根拠がない permission mode は表示しない
- [/] `VendorControlRenderModel` を標準 UI の読み取り口にし、widget 内の vendor ID 直書き分岐を禁止する
- [/] Supported capability に公式URLがない場合は検証または lint で失敗させる
- [/] API-only 完了扱いが起きた原因を harness report と lint へ落とす
- [/] katana の coding rule、通常 lint、AST lint、lefthook gate を kcu に移譲する
- [/] katana の `just update-safe` / `just update` を kcu に移譲し、取り込み後に依存関係を最新化してから検証へ進める入口にする
- [/] 各レイヤーの境界を依存性逆転の原則（Dependency Inversion Principle）に沿った interface として固定する
- [/] panel は slot interface だけを受け、thread / composer / output extension の具象実装を知らない構造にする
- [/] vendor 側のアップデート追従と新 vendor 追加を `VendorFactRegistry` と vendor control interface で吸収する
- [/] Floem composer は行番号付き editor ではなく、通常の chat 入力としてカーソル位置と IME 候補位置を崩さない
- [/] 送信は Command + Enter で行い、Enter 単体では送信しない
- [/] streaming 中は送信ボタンと別に停止ボタンを並べず、送信ボタンの位置を停止に切り替える
- [/] SVG icon button は text catalog 由来の tooltip を持つ
- [/] vendor / model / thinking / permission など可変値は、header ではなく composer control row の selector として表示し、固定 text 表示にしない
- [/] active vendor 自体も `VendorFactRegistry` 由来の候補から選択できる
- [/] manual host の output JSON は標準 chat UI に常時混ぜず、`debug: true` の output handoff hover で確認できる
- [/] manual host の検証用表示は、標準 chat UI の toolbar / thread / composer レイアウトを崩さない
- [/] Floem composer は日本語 IME 入力を維持し、行番号付き editor 表示に戻さない
- [/] window resize 時に標準 chat UI の composer が消えず、thread 側が優先的に縮む
- [/] SVG button は Floem 標準 button の二重枠を出さず、SVG atom として描画する
- [/] vendor / model / thinking の pulldown は二重枠や重なりを出さず、短い選択表示にする
- [/] Ollama model は hardcoded list ではなく `/api/tags` から取得し、取得失敗時は provider unavailable として扱い fallback しない
- [/] pulldown は Floem overlay の重なりや二重枠を出さず、クリックで安定して選択できる popup selector にする
- [/] 送信ボタンと Command + Enter は現在の入力 draft を送信できる
- [/] 添付ボタンを押しても message thread と入力 draft が消えない
- [/] manual host の resize で標準 chat UI と検証用表示が重ならない
- [/] 利用可能 provider が0件の場合、`unavailable` などの fake vendor を selector に表示しない
- [/] 送信可否は draft text だけで判定せず、provider ready、添付あり、stop 状態を含む surface contract で判定する
- [/] Command + Enter は stop 状態を迂回せず、送信は1回だけ発火する
- [/] stop 後に遅れて返った provider result が次の assistant message に混入しない
- [/] `just update-safe` で root / e2e / manual host の lock を更新し、`--locked` の手動 host 実行失敗を残さない
- [/] file 添付は固定サンプルではなく、OS のファイル選択から追加できる
- [/] 添付済み file は composer の添付チップから取消できる
- [/] thinking 表示は静的 text だけにせず、動作中だと分かるアニメーションを持つ
- [/] provider 応答が完了したら応答本文を thread に反映し、実行中 indicator が Thinking のまま止まらない
- [/] 左上の title は active provider の SVG icon と一緒に表示する
- [/] 最初の provider 応答から会話 title を更新する
- [/] ユーザー向け表示では vendor ではなく provider（提供元）として扱う
- [/] provider / model / thinking / permission の pulldown 同士がめり込まない spacing を持つ
- [/] P3 の劣後観点は `openspec/roadmap.md` に分離し、v0.1.0 P0/P1 と混ぜない
- [/] manual host は resize 時に標準 chat UI を優先し、検証用表示で chat 幅を破壊しない
- [/] provider 応答が完了したら Floem thread の再描画キーが更新され、実行中 indicator が本文とは別の考慮ログとして残る
- [/] composer の `Ask anything` は入力本文ではなく text catalog 由来の placeholder として描画し、override 可能にする
- [/] Floem thread の assistant response は固定文字数で縦長に潰さず、利用可能幅に応じて自然に折り返す
- [/] thinking は応答本文へ混ぜず、動作中 indicator と後続の詳細表示 contract を分離する
- [/] file 添付ありの送信で provider request が破綻しないよう、manual host は添付本文を上限付きで prompt 化する
- [/] output JSON は `debug: true` の output handoff hover で確認でき、標準 chat UI を押し潰す右分割や常設表示にしない
- [/] provider icon は `+` などの仮アイコンではなく、provider を識別できる SVG に差し替える
- [/] Floem composer は送信後の draft clear で editor document を直接編集せず、IME 操作中に rope state を壊さない
- [/] 標準 chat UI の右上に用途不明な settings / debug button を表示しない
- [/] 標準 toolbar に用途不明 button を戻す変更は AST lint で検知する
- [/] manual host の output JSON は標準 chat UI を押し潰さない output handoff hover で確認できる
- [/] thinking 表示は provider / model / endpoint / prompt size / status の dump を通常の chat bubble に出さない
- [/] 標準 chat UI は固定 `CONTENT_WIDTH` で横幅を潰さず、thread と composer が利用可能幅へ伸びる
- [/] assistant response は provider 完了を待って一括追加せず、chunk ごとに streaming 表示として増える
- [/] 送信 SVG は標準の上向き矢印として固定し、provider 再検出や manual host の処理で別 icon に変化しない
- [/] Floem composer は Floem の Command+Enter 実コマンド `new_line_below` を送信として扱い、Enter 単体は送信しない
- [/] Command+Enter と送信 icon の回帰は unit test で検知する
- [/] Command+Enter は判定関数だけでなく、Floem editor の `pre_command` 配線を unit test で検知する
- [/] 検証用 host が標準 `send` icon を上書きする退行は AST lint で検知し、icon override 機能は library unit test に分離する
- [/] 標準 widget への用途不明な settings surface、常設 output JSON、toolbar debug button 混入は AST lint で検知する
- [/] manual host 固有の `debug` button / popup を表示しない退行を AST lint で検知する
- [/] built-in provider icon は全 provider を列挙し、missing / generic code / plus placeholder への退行を unit test で検知する
- [/] `debug: true` は crates 側の `ChatUiSurface` option として output handoff hover を表示し、manual host 固有実装に閉じ込めない
- [/] `debug: false` では output handoff hover を表示しないことを unit test で検知する
- [/] `Thinking: false` の場合は thinking surface を作らず、Ollama へ `think: false` を渡す
- [/] thinking は provider / endpoint / model / prompt size の dump ではなく、provider の thinking chunk または明示された実行段階だけを考慮ログとして扱う
- [/] Ollama stream の `message.thinking` と `message.content` を別イベントとして扱い、本文へ混入させない
- [/] Markdown の longer fence 内にある ``` を code block 本文として保持する回帰 test を追加する
- [/] assistant response bubble は本文の短さ/長さで幅が変わらない固定 percentage contract にする
- [/] output は host へ返すだけでなく、標準 UI が本文とは別の output extension surface / host action intent として扱える
- [/] `debug: true` の output handoff は tooltip 任せではなく hover で実体表示し、標準レイアウトを押し潰さない
- [/] Zed / Codex 型に合わせ、file edit / terminal / permission は agent tool output として扱い、実処理は host が所有する
- [/] Ollama は編集できる agent provider ではなく local chat backend として仕様へ明記し、permission UI を出さない
- [/] 手動確認起動は `just harness-up [egui|floem|gpui]` に統一し、引数省略時は Floem を起動する

### To-Be Feedback Tasks

> 以下は仕様化済みだが、実装と回帰テストで合格確認するまで `[ ]` のまま扱う。

- [ ] OS file picker と drag and drop の両方で file を添付でき、添付済み file を composer 内で取り消せる
- [ ] drag and drop は OS file、host logical resource、virtual attachment を区別し、host callback で解決する
- [ ] Ollama は agent provider selector に表示せず、agent provider が使う local runtime 設定として扱う
- [ ] `options: { debug: true }` は crate 側標準機能として右端 output hover を表示する
- [ ] `debug: false` では output hover と検証用 JSON を表示しない
- [ ] 右上 settings toggle から標準設定画面を開ける
- [ ] 標準設定画面は theme、locale、placeholder、SVG icon override、provider 表示順、composer behavior を扱い、後続設定を section / slot で追加できる
- [ ] settings JSON reference が未指定の場合は invalid configuration になり、in-memory fallback をしない
- [ ] settings merge は typed settings を指定 JSON へ merge する intent として扱い、secret value を含めない
- [ ] composer 内の `/` 入力で slash launcher が開き、host-provided prompt / skill / workflow / command entry を `CommandLaunchIntent` として選択できる
- [ ] slash launcher は draft、IME、cursor、attachment tray を壊さない
- [ ] 現在の context usage は used / max / percentage / status として表示できる
- [ ] output extension interface は file、diff、tool result、permission request の詳細 UI を後から追加できる
- [ ] file / diff / tool result の詳細 UI は劣後可能だが、標準 thread に混ぜず output extension slot で扱える
- [ ] Markdown code fence は parser の block で開始と終了を判定し、途中の ``` で途切れない
- [ ] egui / Floem / GPUI は v0.1.0 で同等の標準 UI 手動確認対象として起動できる
- [ ] Zed の `conversation_view.rs`、`acp_thread.rs`、`agent.rs`、`message_editor.rs`、`mention_set.rs`、`agent_settings.rs`、`diff.rs`、`terminal.rs` を責務分離の参照元として実装レビューに使う

---

## 7. Final Verification

- [x] 7.0 `just update` が通り、workspace と外部 host の lock が更新される
- [x] 7.1 `just check` が通る
- [x] 7.2 `just ast-lint` が通る
- [x] 7.3 `npx -y @fission-ai/openspec validate "v0-1-0-chat-widget-floem"` が通る
- [x] 7.4 `/openspec-verify-change` で Critical がない
- [x] 7.4.1 `just manual-ui-check` が標準 UI を載せる host frame として通る
- [x] 7.4.2 `just release-verify` が通る
- [x] 7.5 PR 作成が必要な場合は `/create_pull_request` を使う（GUI 確認前のため未実行、手順のみ固定）
- [x] 7.6 merge 後、必要なら `/openspec-archive-change` を使う（merge 前のため未実行、手順のみ固定）
