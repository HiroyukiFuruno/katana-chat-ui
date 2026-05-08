# katana-chat-ui 標準チャット UI 仕様雛形

この文書は、これまでの指摘から分かっている `katana-chat-ui` のあるべき姿を固定するための雛形です。
現状実装の追認ではなく、v0.1.0 で満たすべき標準チャット UI の契約として扱います。

## 0. 仕様ステータス

- この文書を、標準チャット UI の仕様雛形として扱う。
- 現在の実装や既存 tasks の完了マークは、この仕様への適合証明として扱わない。
- 実装修正は、この仕様と OpenSpec の整合を取ってから着手する。
- 合格判定は、回帰テスト、静的検査、手動確認でこの文書の契約を満たした時だけ行う。

## 1. 目的

`katana-chat-ui` は、利用側が独自のチャット画面を作らなくても使える標準チャット UI を提供する。

API だけを使う道は残すが、それは標準 UI に満足できない利用者向けの拡張手段であり、v0.1.0 の完了条件ではない。

## 2. 画面上でどう見えるか

- 会話タイトルは画面上部に表示する。
- 会話タイトルの左には現在の提供元（provider）または実行主体のアイコンを表示する。
- 右上には「新規チャット開始」「履歴」「設定」の操作アイコンを表示する。これらは装飾ではなく、押すと対応する操作を行う。
- 会話タイトル左の提供元アイコンは、候補が複数ある場合に下向き印を併記し、提供元を切り替えられることを視覚的に示す。
- ユーザー発言は右側、エージェント応答は左側に表示する。
- `User`、`Assistant` のような役割名は通常表示しない。
- 入力欄（composer）は画面下に固定する。
- 添付、モデル、思考設定、権限設定、context 使用率、送信、停止は入力欄の内側に置く。
- 提供元切り替えは入力欄内ではなく、タイトル左の提供元アイコンから行う。
- 送信中は送信ボタンの位置を停止ボタンに切り替える。
- 上下に狭い領域で表示されても入力欄は消さない。縮むのは会話領域とする。
- 回答本文の吹き出しは本文の長さで幅を変えない。利用可能幅に対する一定割合で表示し、最大幅だけ制限する。
- 思考中、実行中、処理中などの状態表示は、回答本文の吹き出し幅ルールの対象外とする。

## 3. 何を操作するものか

- `Enter` は改行する。
- `Command + Enter` は現在の入力内容を 1 回だけ送信する。
- 添付ボタンは添付 intent を発行し、利用側に添付対象の解決を依頼する。
- OS のファイル選択画面を開く実装と実機確認は v0.6.0 で扱う。
- ファイルのドラッグアンドドロップ実装と実機確認は v0.6.0 で扱う。
- ドラッグアンドドロップの入力元は、OS の実ファイル、利用側が持つ論理リソース、仮想添付を区別して扱う。
- `katana-chat-ui` core は OS のファイルを直接読まず、利用側 callback に解決を依頼する。
- 添付済みファイルは入力欄内に表示し、取り消せる。
- 提供元、モデル、思考設定、権限設定は選択 UI で切り替えられる。
- 新規チャット開始は右上の `+` 操作から行う。
- 履歴一覧は右上の履歴操作から開く。
- 設定画面は右上の設定操作から開く。
- 選択肢は実際に利用できるものだけを表示する。
- 利用できない提供元を、選べる状態で表示しない。
- 入力欄で `/` を入力すると、prompt、skill、workflow、command などを起動する候補表示を開く。
- `/` 候補は利用可能なものだけを表示し、未設定や未対応のものを実行可能に見せない。
- 送信済みのユーザー発言は編集できる。
- 送信済み発言を編集した場合、その発言以降の応答と output は無効化し、編集した時点から会話を再開する。
- 処理中でも追加投稿できる。追加投稿は会話誘導（steering）または割り込み（interrupt）として扱い、通常送信と区別する。
- 割り込み時は進行中の応答を曖昧に継続せず、停止、打ち切り、キュー投入のいずれかを provider 能力に応じて明示する。

## 4. 入力欄

- 日本語 IME の入力、変換候補、確定位置を壊さない。
- プレースホルダーは入力値ではなく、入力が空の時だけ薄く表示する。
- プレースホルダー文言は文言カタログ（text catalog）で上書きできる。
- 添付操作、提供元切り替え、モデル切り替えで入力中の文字を消さない。
- 行番号付きエディタのような見た目にしない。

## 5. 会話表示

- 応答はストリーミング（streaming）断片ごとに追記する。
- 完了後に全文を一括表示する挙動へ戻さない。
- Markdown 表示は v0.1.0 の必須要件とする。
- Markdown は `comrak` を暫定採用し、将来 `katana-document-viewer` に差し替えられる境界を持つ。
- 対応対象は、見出し、段落、強調、取り消し線、リンク、自動リンク、インラインコード、コードブロック、引用、通常リスト、番号付きリスト、タスクリスト、表とする。
- Markdown の見た目の磨き込みは v0.1.0 で深追いしないが、コードブロックの開始と終了は parser で判定し、途中で途切れさせない。
- fenced code block は内側の ``` を本文として保持できる。単純な文字列分割で Markdown を壊さない。
- raw HTML は描画せずテキストとして扱う。
- 画像 Markdown は v0.1.0 では画像描画せず、安全な alt / link 相当の表示にする。
- 会話内容は session ごとに履歴ファイルへ保存する。
- 履歴ファイルには、会話本文、状態表示履歴、添付 metadata、provider / runtime 選択、output 参照、draft、scroll 復元に必要な情報を持たせる。
- UI から session 履歴一覧を表示し、過去の会話を選んで復元できる。
- session 履歴一覧には title、provider、最終更新時刻、短い preview、状態を表示できる。
- 現在開いている chat 内を検索できる。
- 保存済み履歴全体を横断検索できる。
- 検索結果から対象 session と message へ移動できる。

## 6. 思考表示

- 思考設定が `false` の場合、思考中表示も思考ログも出さない。
- 思考設定が有効な場合だけ、応答中に思考表示を出す。
- 思考中は動作中であることが分かるアニメーションを表示する。
- 思考ログは回答本文に混ぜない。
- 思考ログは折りたたみ可能な別表示にする。
- 応答完了後は思考ログを自動で折りたたむ。

## 7. 状態表示

状態表示は、会話本文ではなくエージェントの作業状態を示すための表示である。
本文吹き出しと同じ幅ルールを適用しない。

- 思考中（thinking）: モデルまたはエージェントが回答方針や手順を考えている状態。
- コマンド実行中（execution）: shell、task runner、外部コマンドなどを実行している状態。
- 処理中（processing）: 応答生成、変換、集計、解析など、具体的な読み書きやコマンド実行ではない処理を行っている状態。
- 読み取り中（read）: ファイル、差分、設定、履歴などを読んでいる状態。
- 編集中（editing）: ファイル生成、ファイル更新、差分作成など、変更を作っている状態。
- 取得中（fetch）: ネットワーク、提供元 API、外部 adapter から情報を取得している状態。

状態表示は次を満たす。

- 実行中であることが分かる短いラベルと動きのある表示を持つ。
- 詳細ログを持つ場合は折りたたみ可能にする。
- 完了後は必要な履歴だけ残し、本文の一部として混ぜない。
- 失敗した場合は、どの状態で失敗したかを表示できる。
- provider / model / endpoint / prompt size の dump を状態表示の代替にしない。

## 8. 提供元と実行能力

- 表示上の概念名は提供元（provider）を基本とする。
- 以前の提供元名（vendor）は内部互換や過去仕様の表現として扱い、UI 文言には原則出さない。
- Ollama はローカルモデル基盤（local model backend）であり、ファイル編集やコマンド実行を直接行うエージェント提供元ではない。
- Ollama を使う場合は会話バックエンドまたは agent provider の model runtime として扱い、編集権限や権限設定を持つ提供元のように見せない。
- v0.1.0 の既定は、Ollama を低コストの文書作成バックエンドとして使う。
- v0.1.0 の provider selector には Ollama を編集・コマンド実行できる agent provider として出さない。
- Ollama を文書作成バックエンドとして表示する場合も、permission、file edit、terminal execution の UI は出さない。
- Ollama の endpoint や model は、agent provider が使う local runtime 設定として扱う。
- Claude Code、Codex CLI、GitHub Copilot、OpenCode などはエージェント提供元候補として扱う。
- 提供元ごとの UI 表示は、公式情報に基づく提供元情報（provider facts）と実行時の利用可能状態から決める。
- provider ごとの token usage、account usage、quota などの利用状況は、provider capability として扱う。
- usage を返せない provider は取得不可の理由を表示できるが、推定値を作らない。
- UI 実装内で `ollama` などの ID を直書き分岐しない。
- 根拠のない UI は表示しない。
- 対応はあるが設定不足の場合は、非表示ではなく無効状態として表示する。

## 9. 出力と検証補助

- 出力（output）は、生成ファイル、差分、ツール結果、権限要求などを利用側へ渡す付随情報である。
- `katana-chat-ui` 自身も output を読む。標準 UI は本文とは別の契約として、生成物の要約、実行状態、利用側へ渡す操作意図を扱う。
- 標準チャット本文に debug JSON や検証用差分カードを常時表示しない。
- 標準チャット本文を、手動確認用 UI で汚染しない。
- ファイル内容、差分、コマンド実行結果の詳細表示は、本文ではなく output 拡張領域または利用側の差分ビュー、ファイルビューへ渡す。
- 現在の context 利用状況は標準 UI で表示する。
- context 利用状況は used tokens、max tokens、percentage、status を持つ。
- context 利用状況は入力欄内の送信ボタン左側に、percentage を円グラフとして表示する。
- 手動確認用の output JSON は右端のホバー領域からだけ表示する。
- `options: { debug: true }` の場合だけ、標準 UI 側の debug surface として output JSON を確認できる。
- `debug: false` の通常利用では output JSON を表示しない。
- 通常時、右端の補助表示はチャット UI の幅を奪わない。
- ホバーした時だけ、何もない右端から右から左へ重なる形で表示する。
- output の右端ホバー領域は、後から項目を追加できる拡張 slot として設計する。
- output の実処理、ファイル書き込み、差分適用、コマンド実行、権限承認は利用側または提供元が所有する。
- `katana-chat-ui` は output を構造化して返すが、利用側と密結合しない。
- file / diff / tool result の詳細 UI は v0.1.0 で劣後してよい。ただし後から標準 UI に追加できる output extension interface は v0.1.0 で用意する。

## 10. 設定画面

- 設定画面は右上のトグルアイコンから開く。
- 設定画面は `debug: true` だけの検証補助ではなく、標準 UI の機能として提供する。
- v0.1.0 では、テーマ、言語、placeholder 文言、SVG icon 差し替え、provider 表示順、composer 挙動を設定できる。
- 設定画面は後から項目を追加できる section / slot 構造にする。
- provider 接続設定、secret、account usage、provider usage は v0.2.0 で扱う。
- prompt 設定、skill、workflow、command、hook、MCP は agent / runtime 設定として扱う。
- prompt / skill / workflow / command / hook / MCP の実エントリは、provider / adapter capability と host 設定から生成する。
- 設定の保存先は利用側が指定する。
- 利用側は保存先 JSON を省略できない。省略は invalid configuration として扱う。
- kcu は指定された JSON へ typed settings を merge する。
- secret value は設定 JSON に保存しない。
- unknown key や不正な型は silent fallback せず、invalid settings として返す。

### 10.1 `/` 起動

- 入力欄で `/` を入力すると、prompt、skill、workflow、command を検索・起動できる候補表示を開く。
- 候補は provider / adapter / host が利用可能と返したものだけを表示する。
- skill、workflow、command、hook、MCP は同じ文字列ではなく、種類を持つ structured entry として扱う。
- 候補を選んだ結果は、ただの入力文字列ではなく `CommandLaunchIntent` として表現する。
- 未対応の候補は表示しないか、無効状態と理由を表示する。
- `/` 起動は IME、添付、draft、cursor 位置を壊さない。

## 11. 手動確認枠

- 手動確認枠（harness）は標準チャット UI を載せるだけの枠にする。
- 手動確認枠が独自のチャット UI を作らない。
- 起動入口は `just harness-up` とする。
- 引数なしは Floem を起動する。
- `just harness-up egui`、`just harness-up floem`、`just harness-up gpui` で表示方式を切り替える。
- egui / Floem / GPUI は v0.1.0 で同等の手動確認対象とする。
- 手動確認枠は標準 UI に `options: { debug: true }` を渡せるが、独自の debug UI を作らない。
- output JSON などの検証補助は、crate 側の標準 debug surface として提供し、手動確認枠はそれを起動するだけにする。

## 12. 禁止事項

- API だけを実装して v0.1.0 完了扱いにしない。
- 標準 UI の会話本文に手動確認用 debug 表示を混ぜない。
- `Thinking: false` で思考表示を出さない。
- `Enter` 単体で送信しない。
- 回答幅を本文の長さで変えない。
- provider が利用不可な状態を、利用可能に見せない。
- fallback で別 provider や固定モデルを勝手に使わない。
- Ollama を agent provider selector に表示しない。
- 手動確認枠専用の debug / output UI を標準 UI と別実装で作らない。
- 設定保存先を省略可能にしない。
- `/` 候補に未対応の skill / workflow / command を実行可能として出さない。
- provider usage や account usage を推定で作らない。
- 検証コードだからという理由で雑な UI や雑な設計にしない。
- 状態表示を回答本文の代わりに使わない。

## 13. 回帰テスト契約

- `Thinking: false` で思考 message が生成されないこと。
- `Command + Enter` が送信になり、`Enter` 単体が送信にならないこと。
- `Command + Enter` は現在の入力内容を 1 回だけ送信すること。
- 応答本文の吹き出しが固定割合の幅を使うこと。
- thinking / execution / processing / read / editing / fetch の状態表示が本文吹き出しとして扱われないこと。
- output JSON が標準 message list に混入しないこと。
- output JSON は debug 有効時の右端ホバーだけで表示されること。
- 右上設定トグルで設定画面を開けること。
- 右上の新規チャット開始、履歴、設定の操作 surface が存在し、それぞれ文言カタログと SVG icon を持つこと。
- provider selector は入力欄内に表示されず、タイトル左の provider icon から開けること。
- 設定保存先 JSON が未指定の場合に invalid configuration になること。
- 設定 JSON merge が既存値を破壊しないこと。
- `/` で候補表示が開き、選択結果が `CommandLaunchIntent` になること。
- `/` 候補で draft、IME、cursor、添付が壊れないこと。
- debug 無効時に右端の検証補助 UI が出ないこと。
- provider selector に Ollama が agent provider として出ないこと。
- Ollama は local runtime 設定として扱われ、permission UI を持たないこと。
- 現在の context usage が used / max / percentage / status として表示されること。
- provider usage は capability がある場合だけ表示し、取得不可時は理由を持つこと。
- 添付操作で draft と thread が消えないこと。
- 添付操作が OS file、host logical resource、virtual attachment を区別できる interface を通ること。
- host 論理リソースの drop が OS ファイルと区別され、host callback で解決されること。
- 送信済み発言を編集すると、その発言以降を無効化して再開できること。
- 処理中の追加投稿が、通常送信ではなく steering / interrupt intent として扱われること。
- session 履歴を保存し、UI から復元できること。
- session 履歴一覧を表示できること。
- 現在 chat 内検索と履歴横断検索ができること。
- fenced code block の開始と終了を parser で判定し、途中の ``` で途切れないこと。
- 送信ボタンの SVG が作業のたびに別アイコンへ変わらないこと。
- 入力欄は上下 resize で消えないこと。

## 14. 次期以降も保持する UI 仕様

この節は v0.1.0 の必須実装から外す場合でも、`katana-chat-ui` の UI としてあるべき姿からは外さない。
実装時期は `openspec/roadmap.md` で管理する。

### 13.1 添付入力

- OS のファイル選択画面からファイルを追加できる。
- ファイルをドラッグアンドドロップで添付できる。
- OS file、host logical resource、virtual attachment の実入力を区別し、host callback で解決できる。
- クリップボードから画像を貼り付けて添付できる。
- 貼り付けた画像は、通常の添付ファイルと同じように入力欄内へ表示し、取り消せる。
- 画像貼り付けで入力中の文字、添付済みファイル、会話履歴を消さない。

### 13.2 文字入力

- macOS の絵文字パレットから絵文字を挿入できる。
- 絵文字挿入でカーソル位置、IME 変換状態、プレースホルダー表示を壊さない。

### 13.3 時間表示

- 応答中は試行時間をリアルタイムに表示する。
- 試行時間は送信時に開始し、完了、失敗、停止で止まる。
- ユーザー発言には送信時刻を持たせる。
- エージェント応答には開始時刻、完了時刻、失敗時刻、停止時刻を持たせる。
- 時刻情報は UI 表示と render model の両方で扱える。

### 14.4 提供元選択

- 提供元（provider）選択 UI の各候補行には、提供元ごとの SVG アイコンを表示する。
- 候補行の SVG は、会話タイトルや入力欄の provider 表示と同じ icon registry を使う。
- `+`、汎用コード記号、欠落アイコンを provider icon の代替として使わない。

### 14.5 Markdown renderer 差し替え

- `comrak` は暫定 renderer として扱う。
- `katana-document-viewer` の準備ができたら、Markdown 表示を差し替える。
- 差し替え時に、chat message、output、provider、composer の public contract を壊さない。
- renderer 差し替え後も、v0.1.0 で定めた Markdown subset の安全性は維持する。

### 14.6 思考履歴

- thinking は Zed に近い折りたたみ履歴 UI へ磨き込む。
- v0.1.0 では、考慮ログの表示、本文との分離、完了後の自動折りたたみまでを必須とする。
- 次期以降は、何を考えているか、どの段階にいるか、どれだけ時間がかかったかを履歴として追えるようにする。
- thinking 履歴は回答本文に混ぜず、専用の表示契約として扱う。

## 15. Zed 参照対象

Zed は完全コピーではなく、責務分離、composer、thinking、streaming、tool / output、履歴復元の参照元として扱う。
参照対象は少なくとも次とする。

- `crates/agent_ui/src/conversation_view.rs`: thread 表示、composer、送信 / 停止、送信済み message 編集、処理中の queue / interrupt、thinking 表示。
- `crates/acp_thread/src/acp_thread.rs`: message chunk、thought chunk、tool call status、cancel、rewind、diff / terminal / usage event。
- `crates/acp_thread/src/connection.rs`: ACP session update、permission request、cancel command。
- `crates/agent/src/thread.rs`: session 内容、draft、snapshot、tool input / output。
- `crates/agent/src/thread_store.rs` と `crates/agent/src/db.rs`: session 保存、thread 復元、draft / scroll 復元。
- `crates/agent_ui/src/thread_metadata_store.rs`: thread list、metadata 保存、session id と UI 復元情報の対応。
- `crates/agent_ui/src/message_editor.rs`: composer editor、placeholder、入力操作、paste / external file path。
- `crates/agent_ui/src/mention_set.rs`: file、image、path、context、diagnostics、git diff の mention 解決。
- `crates/agent_ui/src/config_options.rs`、`crates/agent_ui/src/model_selector.rs`、`crates/agent_settings/src/agent_settings.rs`: model、thinking、permission、profile などの設定。
- prompt、skill、workflow、command、hook、MCP の設定と `/` 起動も Zed の設定、command palette、agent config の分離を参照する。
- `crates/acp_thread/src/diff.rs` と `crates/acp_thread/src/terminal.rs`: diff、terminal、tool 実行結果の扱い。

## 16. 現時点の確定事項

- Ollama は v0.1.0 の既定文書作成バックエンドとして使う。ただし編集・コマンド実行できる agent provider としては表示しない。
- Ollama の endpoint / model / thinking は local runtime 設定として扱い、permission UI は出さない。
- file / diff / tool result の詳細 UI は劣後可能。ただし output extension interface は v0.1.0 の範囲とする。
- Markdown 表示の見た目の磨き込みは最小限でよい。ただし code fence の途切れは v0.1.0 で解消する。
- egui / Floem / GPUI は v0.1.0 で同等の標準 UI 手動確認対象とする。
- output JSON の補助表示は `options: { debug: true }` の標準 UI 機能として提供し、手動確認枠だけの実装にしない。
- 設定画面は右上トグルから開く。
- output JSON は `debug: true` の右端 hover として表示する。
- prompt、skill、workflow、command、hook、MCP は設定画面と `/` 起動の両方から扱えるように設計する。

## 17. 優先度と対応 version

この節は、本書に書かれたあるべき姿を release version へ割り当てるための優先度表である。
v0.1.0 は標準チャット UI が成立し、人間が egui / Floem / GPUI で確認できる範囲に限定する。

| Priority | Version | Scope | OpenSpec change |
| --- | --- | --- | --- |
| P0 | v0.1.0 | 標準 chat UI、composer、provider / runtime control、Command+Enter、stop / send 切替、IME、resize、streaming、thinking false 制御、thinking ログ、Markdown code fence、現在 context usage、設定画面 shell、`/` 起動 contract、output contract、`debug: true` 右端 output hover、3 host 手動確認 | `v0-1-0-chat-widget-floem` |
| P0 | v0.2.0 | ACP 接続、direct connector、secret store、provider usage、account usage、settings schema、settings JSON merge、provider 接続設定、MCP server 設定 | `v0-2-0-secure-connector-and-usage` |
| P1 | v0.3.0 | agent provider と local runtime の分類、Claude Code / Codex / GitHub Copilot / OpenCode などの adapter hook、cloud direct の分類、prompt / skill / workflow / command / hook catalog、`/` 候補供給 | `v0-3-0-vendor-adapter-expansion` |
| P1 | v0.4.0 | session 履歴保存 / 一覧 / 復元、現在 chat 検索、履歴横断検索、送信済み message 編集、処理中の steering / interrupt / queue、旧 response chunk 混入防止 | `v0-4-0-conversation-history-and-control` |
| P2 | v0.5.0 | file / diff / tool result / permission request の詳細表示、Zed 型の output extension view、command execution 表示 | `v0-5-0-output-extension-views` |
| P3 | v0.6.0 | OS file picker、drag and drop 添付実装、クリップボード画像添付、絵文字パレット、試行時間、送受信時刻、provider selector icon、`katana-document-viewer` 連携 | `v0-6-0-rich-input-and-renderer-polish` |

### v0.1.0 の明確なスコープ

v0.1.0 に含める。

- 利用側が独自 chat UI を作らず載せられる標準 UI。
- 入力欄、添付、provider / runtime control、model、thinking、permission の基本 control。
- 添付 intent と attachment interface。OS file picker と drag and drop の実機確認は v0.6.0 へ劣後する。
- `Enter` 改行、`Command + Enter` 送信、送信中の stop button 切替。
- 応答の streaming 表示。
- `Thinking: false` では thinking surface を作らないこと。
- thinking が有効な場合の考慮ログ表示と完了後の自動折りたたみ。
- Markdown の v0.1.0 subset と code fence の途切れ防止。
- file / diff / tool result / permission request を output として分類し、host action intent として返す contract。
- `options: { debug: true }` の右端 output hover。
- 右上トグルから開く標準設定画面。
- 現在の context usage 表示。
- 入力欄の `/` 起動 contract。
- egui / Floem / GPUI の手動確認 host。
- Ollama は agent provider ではなく local runtime として扱う。

v0.1.0 から外す。

- session 履歴の UI 復元。
- 送信済み message の編集と、その時点からの再開。
- 処理中の追加投稿による steering / interrupt。
- file / diff / tool result の詳細 view。
- OS file picker、drag and drop 添付実装、クリップボード画像添付、絵文字パレット、送受信時刻、試行時間。
- `katana-document-viewer` への renderer 差し替え。
