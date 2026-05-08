## ADDED Requirements

### Requirement: kcu core は UI framework と host application に依存してはならない

システムは、`katana-chat-ui` core crate の public API から UI framework 型、host application 固有型、vendor SDK 型を排除しなければならない（MUST）。標準 UI は framework 別 crate で提供する。

#### Scenario: core dependency tree を確認する

- **WHEN** `cargo tree -p katana-chat-ui` を実行する
- **THEN** `egui` / `eframe` / `floem` / `vello` は含まれない
- **THEN** host application 固有 crate は含まれない

#### Scenario: framework UI crate から利用する

- **WHEN** framework 別 UI crate が kcu core を利用する
- **THEN** UI crate は `ChatRenderModel` / `ChatUiSurface` と callback contract を読み、標準 chat UI を描画する
- **THEN** kcu core は Floem / egui / GPUI 型を受け取らない

### Requirement: 依存関係更新後に検証へ進む入口を提供しなければならない

システムは、取り込み後に依存関係を最新化してから検証へ進む入口として `just update-safe` と `just update` を提供しなければならない（MUST）。

#### Scenario: SemVer 範囲内で依存関係を更新する

- **WHEN** user が `just update-safe` を実行する
- **THEN** `cargo update` が実行される
- **THEN** `Cargo.toml` の dependency requirement は変更されない
- **THEN** workspace 外の検証 host lock file も `--locked` 検証に追従する

#### Scenario: 破壊的変更を含む依存関係更新を取り込む

- **WHEN** user が `just update` を実行する
- **THEN** `cargo-upgrade` が利用可能であることを先に確認する
- **THEN** `cargo upgrade -i` の後に `cargo update` が実行される
- **THEN** `tools/e2e-host-app` と `tools/manual-host-*` の dependency requirement と lock file も更新される

#### Scenario: cargo-upgrade が利用できない

- **WHEN** `cargo-upgrade` がない環境で user が `just update` を実行する
- **THEN** 導入方法を含む明確なエラーを表示して失敗する
- **THEN** upgrade step を黙って省略しない

### Requirement: kcu は標準 chat UI を提供しなければならない

システムは、API と render model だけでなく、利用側がそのまま載せられる標準 chat UI 部品を提供しなければならない（MUST）。API-only の利用は、標準 UI に満足できない利用者向けの拡張手段であり、MVP の完了条件ではない。

#### Scenario: 標準 Floem UI を利用する

- **WHEN** host が `katana-chat-ui-floem` を利用する
- **THEN** host は独自に会話 UI を実装せず、kcu が提供する Floem chat widget を載せられる
- **THEN** widget は message list、composer、attachment tray、send / stop action を表示する

#### Scenario: API-only で独自 UI を作る

- **WHEN** 利用側が標準 UI ではなく API-only で独自 UI を作る
- **THEN** それは明示的な customization path として扱う
- **THEN** v0.1.0 の完了判定は API-only 実装だけでは満たされない

### Requirement: 標準的な AI chat 入力を扱えなければならない

システムは、multiline text、file attachment、image attachment、path drop、host resource drop を `ChatInputDraft` と `Attachment` で扱えなければならない（MUST）。

#### Scenario: file path が drop される

- **WHEN** ユーザーが UI に file path を drag and drop する
- **THEN** kcu は host callback に file 読み取りを依頼する
- **THEN** host が返した内容だけを `FileResource` attachment として保持する
- **THEN** kcu core は filesystem を直接読まない

#### Scenario: OS file picker で file を添付する

- **WHEN** ユーザーが attach button から OS の file picker を開く
- **THEN** 選択した file は host callback で解決される
- **THEN** 添付済み file は composer 内の attachment tray に表示される
- **THEN** ユーザーは添付済み file を取り消せる

#### Scenario: host resource が drop される

- **WHEN** host application 内の論理リソースが chat UI に drop される
- **THEN** kcu は OS file path と host resource ID を区別する
- **THEN** host resource は host callback で `EmbeddedResource` または `FileResource` に解決される
- **THEN** kcu core は host application 固有型に依存しない

#### Scenario: agent が image input を support しない

- **WHEN** image capability のない provider に画像を添付する
- **THEN** システムは送信前に unsupported attachment state を返す
- **THEN** 画像を黙って text に変換しない

### Requirement: Markdown subset を安全に描画しなければならない

システムは、assistant message と user draft preview に見出し、段落、強調、取り消し線、link、自動 link、inline code、code block、blockquote、通常 list、番号付き list、task list、table を描画できなければならない（MUST）。

#### Scenario: code block を表示する

- **WHEN** assistant message に fenced code block が含まれる
- **THEN** render model は code block として区別できる block を返す
- **THEN** language label は存在する場合のみ metadata として保持する

#### Scenario: longer fence の code block を保持する

- **WHEN** assistant message に ```` などの longer fence で囲まれた code block が含まれる
- **THEN** 内側の ``` は code block の本文として保持する
- **THEN** 内側の fence を理由に code block を途中で分断しない

#### Scenario: code fence の開始と終了を parser で判定する

- **WHEN** assistant message に fenced code block と通常 text が混在する
- **THEN** render model は code block の開始 fence と終了 fence を parser の block として判定する
- **THEN** 単純な文字列分割で code block を途中で閉じない
- **THEN** 未閉じ fence は未完成 block として保持し、後続 chunk で閉じられる

#### Scenario: raw HTML が含まれる

- **WHEN** message に raw HTML、script、style が含まれる
- **THEN** システムは実行可能 HTML として描画しない
- **THEN** text として安全に表示する

#### Scenario: table と task list を表示する

- **WHEN** assistant message に table または task list が含まれる
- **THEN** render model は table row、cell、checked state を区別できる block を返す
- **THEN** Floem widget は Markdown AST ではなく render model の block 表現だけを描画する

#### Scenario: image Markdown が含まれる

- **WHEN** message に image Markdown が含まれる
- **THEN** v0.1.0 では画像を直接描画しない
- **THEN** alt と link 相当の安全な inline 表示として扱う

### Requirement: message role ごとの視覚差分を表現しなければならない

システムは、user、assistant、tool、system の message role ごとに visual intent を返さなければならない（MUST）。

#### Scenario: user message を表示する

- **WHEN** message role が user である
- **THEN** render model は assistant message と異なる bubble / alignment / highlight intent を返す

#### Scenario: agent message を表示する

- **WHEN** message role が assistant である
- **THEN** render model は streaming / complete / error status を表示できる

### Requirement: theme token と SVG icon override を提供しなければならない

システムは、host から theme token と SVG icon override を受け取り、button icon をすべて SVG asset id として扱い、標準 UI 上で SVG として描画しなければならない（MUST）。

#### Scenario: host が theme を渡す

- **WHEN** host が `ThemeTokens` を渡す
- **THEN** chat surface、user bubble、assistant bubble、border、accent、danger、warning、success の色が render model に反映される

#### Scenario: host が icon を上書きする

- **WHEN** host が `send` icon id に別 SVG を登録する
- **THEN** render model は default SVG ではなく override された SVG を参照する
- **THEN** 標準 UI の send button は override 後の SVG を描画する

#### Scenario: icon override が更新される

- **WHEN** host が runtime 中に icon registry を差し替える
- **THEN** 次回 render model / UI surface 生成時に新しい SVG が button に反映される

### Requirement: UI 文言を kcu 側で i18n 対応できなければならない

システムは、UI 文言を kcu 側の text catalog から取得し、host から locale と override JSON を受け取って文言を切り替えられなければならない（MUST）。標準 catalog は katana が対応する `en`、`ja`、`zh-CN`、`zh-TW`、`ko`、`pt`、`fr`、`de`、`es`、`it` を提供する。

#### Scenario: MVP の英語文言を表示する

- **WHEN** host が locale を指定しない
- **THEN** 標準 UI は default text catalog の英語文言を使う

#### Scenario: locale を指定して文言を切り替える

- **WHEN** host が `ja` や `zh-CN` などの supported locale を指定する
- **THEN** 標準 UI は kcu 側の builtin catalog から該当 locale の文言を使う
- **THEN** 標準 UI 実装を分岐だらけにせず文言を差し替えられる

#### Scenario: host が文言 JSON を上書きする

- **WHEN** host が `send_button` などの一部文言だけを override JSON で渡す
- **THEN** kcu は指定された文言だけを差し替える
- **THEN** 指定されていない文言は selected locale の builtin catalog を使う
- **THEN** 未知の文言 key は silent fallback せず invalid override として返す

### Requirement: vendor ごとに UI affordance を変えられなければならない

システムは、active vendor profile の vendor ID、表示名、capability に応じて model selector、mode selector、thinking level、permission mode、tool approval などの UI affordance を出し分けられなければならない（MUST）。

UI affordance の表示可否は、公式ドキュメント根拠を持つ vendor fact から決定しなければならない（MUST）。標準 UI は `VendorControlRenderModel` を読み、widget 内で vendor ID の文字列分岐をしてはならない（MUST）。

#### Scenario: active vendor profile を保持する

- **WHEN** host が vendor ID と表示名を持つ profile を渡す
- **THEN** render model は active vendor ID と表示名を保持する
- **THEN** 標準 UI は active vendor の capability から UI affordance を生成する

#### Scenario: thinking level を持つ vendor を表示する

- **WHEN** provider capability が thinking level を提供する
- **THEN** 標準 UI は thinking level selector を表示できる

#### Scenario: permission mode を持たない vendor を表示する

- **WHEN** provider capability が permission mode を提供しない
- **THEN** 標準 UI は permission mode selector を表示しない

#### Scenario: Ollama は local chat backend として扱う

- **WHEN** active provider が local runtime を必要とする
- **THEN** Ollama endpoint と model selector は runtime 設定として表示される
- **THEN** model selector の候補は `/api/tags` の結果から生成される
- **THEN** thinking selector は Ollama Chat API の `think` 根拠に基づいて表示される
- **THEN** permission mode selector は表示されない
- **THEN** account usage は `Unavailable` として扱われる
- **THEN** Ollama は file edit / terminal execution を行える agent provider ではなく local chat backend として扱われる
- **THEN** provider selector に Ollama を編集・コマンド実行できる agent provider として表示しない

#### Scenario: Ollama を既定の低コスト文書作成バックエンドにする

- **WHEN** manual host が Ollama `/api/tags` から model を取得できる
- **THEN** 起動直後の既定 provider は Ollama local になる
- **THEN** model selector は `/api/tags` から得た model を表示する
- **THEN** thinking selector は `false`、`low`、`medium`、`high` を表示する
- **THEN** permission mode selector は表示しない
- **THEN** 応答本文は streaming chunk ごとに thread に追記される
- **THEN** 生成本文は file candidate output として host handoff へ渡される
- **THEN** Ollama が取得できない場合は fake provider を作らず、利用不可として扱う

#### Scenario: agent provider と local chat backend を区別する

- **WHEN** provider selector が候補を表示する
- **THEN** file edit、terminal、permission を持つ候補は agent provider として扱われる
- **THEN** Ollama のように会話応答だけを返す候補は local chat backend として扱われる
- **THEN** local chat backend には編集権限や permission mode を表示しない
- **THEN** local chat backend は agent provider が使う model runtime 設定として扱える

#### Scenario: vendor と可変 control を選択できる

- **WHEN** registry に複数 vendor fact がある
- **THEN** 標準 UI は active vendor を選択できる control を表示する
- **THEN** model / mode / thinking / permission は、active vendor の `VendorControlRenderModel` が持つ option から選択できる
- **THEN** widget は vendor ID の直書き分岐を持たない

#### Scenario: vendor control は会話入力欄と分離する

- **WHEN** 標準 UI が composer を描画する
- **THEN** attach / send / stop など入力操作は composer 内に表示される
- **THEN** provider / model / thinking / permission の選択 UI は composer 内の control row に表示される
- **THEN** composer 外の toolbar へ provider control や用途不明な debug button を戻してはならない

#### Scenario: Supported capability には公式URLが必要

- **WHEN** vendor fact の capability が `Supported` である
- **THEN** capability は公式URLを持たなければならない
- **AND** 公式URLがない場合は検証または lint で失敗する

### Requirement: 現在 context usage の表示枠を持たなければならない

システムは、現在の context token 使用量を表示できる render model を持たなければならない（MUST）。
provider usage と account usage の取得は v0.2.0 で扱うが、v0.1.0 は取得不可状態を表現できなければならない（MUST）。

#### Scenario: context usage が取得できる

- **WHEN** provider から used tokens と max tokens が渡される
- **THEN** UI は percentage と status を計算して表示できる

#### Scenario: account usage が取得できない

- **WHEN** provider が account usage を提供しない
- **THEN** UI は空白ではなく `Unavailable(reason)` として扱う

### Requirement: 標準設定画面を持たなければならない

システムは、右上トグルから開く標準設定画面を持ち、theme、locale、placeholder 文言、SVG icon override、provider 表示順、composer behavior を設定できなければならない（MUST）。

#### Scenario: 設定画面を開く

- **WHEN** user が右上の設定トグルを押す
- **THEN** 標準 UI は設定画面を開く
- **THEN** 設定画面は debug true 専用の補助表示ではない
- **THEN** 用途不明な toolbar button ではなく、text catalog の tooltip を持つ

#### Scenario: 設定保存先がない

- **WHEN** host が settings JSON reference を渡さない
- **THEN** kcu は invalid configuration を返す
- **THEN** silent fallback の in-memory 保存を行わない

#### Scenario: 設定を保存する

- **WHEN** user が theme または locale を変更する
- **THEN** kcu は指定された settings JSON へ typed settings を merge する intent を返す
- **THEN** unknown key や不正な型は invalid settings として返す
- **THEN** secret value を settings JSON に含めない

### Requirement: slash launcher を持たなければならない

システムは、入力欄で `/` を入力した時に prompt、skill、workflow、command を起動する候補表示を開けなければならない（MUST）。

#### Scenario: slash launcher を開く

- **WHEN** composer が空または command position にあり user が `/` を入力する
- **THEN** 標準 UI は slash launcher を開く
- **THEN** draft、IME、cursor、attachment tray を破壊しない

#### Scenario: 候補を選択する

- **WHEN** user が slash launcher の候補を選択する
- **THEN** 選択結果は `CommandLaunchIntent` として表現される
- **THEN** prompt、skill、workflow、command、hook、MCP は kind を持つ structured entry として扱われる
- **THEN** 未対応の候補は表示しないか、disabled reason を持つ

### Requirement: 生成物を host へ渡す output contract を持たなければならない

システムは、LLM の生成物を chat message の文字列だけとして扱わず、host が安全に処理できる output と host action intent として表現しなければならない（MUST）。

#### Scenario: file 生成候補を output として返す

- **WHEN** assistant が file 生成候補を返す
- **THEN** render model は `FileCandidate` output として path、MIME type、content を区別できる
- **THEN** 標準 UI は本文とは別の output extension surface または host action intent として扱える
- **THEN** kcu core は file system に書き込まない
- **THEN** host は `CreateFile` action intent を受け取れる

#### Scenario: diff 候補を output として返す

- **WHEN** assistant が差分候補を返す
- **THEN** render model は `DiffCandidate` output として target path と unified diff を区別できる
- **THEN** 標準 UI は本文とは別の output extension surface または host action intent として扱える
- **THEN** kcu core は diff を適用しない
- **THEN** host は `ApplyDiff` action intent を受け取れる

#### Scenario: tool 実行や権限承認を表示する

- **WHEN** provider が tool result または permission request を返す
- **THEN** render model は表示用 output と host action intent を返す
- **THEN** 標準 UI は本文とは別の output extension surface または host action intent として扱える
- **THEN** kcu core は tool 実行や承認処理を所有しない

#### Scenario: debug true で output handoff を確認する

- **WHEN** host が `ChatUiOptions { debug: true }` を渡す
- **THEN** 標準 UI は右端 hover で output JSON を表示できる
- **THEN** output JSON は message list に常時混入しない
- **THEN** hover 表示は thread と composer の横幅を押し潰さない

#### Scenario: debug false では output handoff を出さない

- **WHEN** host が `ChatUiOptions { debug: false }` を渡す
- **THEN** 標準 UI は output JSON hover を表示しない
- **THEN** 設定画面は debug state に依存せず右上トグルから開ける

#### Scenario: output extension interface を公開する

- **WHEN** assistant response が file、diff、tool result、permission request を含む
- **THEN** kcu は詳細 UI を v0.1.0 で実装しなくても、output extension interface に分類済み output を渡せる
- **THEN** 後続の file view、diff view、tool result view は標準 thread を改修せず slot として追加できる

### Requirement: 外部 host fixture で E2E 検証できなければならない

システムは、`crates/` に含めない外部 host fixture から kcu を downstream dependency として実際に取り込み、Final Verification の直前に起動・描画・入力・添付・usage 表示を検証できなければならない（MUST）。

#### Scenario: 外部 host fixture が起動する

- **WHEN** E2E harness が `tools/e2e-host-app` を起動する
- **THEN** fixture は kcu を path dependency または release 検証時の git dependency として取り込む
- **THEN** fixture は chat panel、composer、attachment tray、usage meter を描画する

#### Scenario: 基本操作を検証する

- **WHEN** E2E harness が text input、file attachment、path drop、send、stop を操作する
- **THEN** fixture は kcu の render model と callback contract を通じて state 変化を確認する
- **THEN** host 固有 API や `crates/` 内の test-only shortcut に依存しない

#### Scenario: E2E 結果からユーザーフィードバックを task 化する

- **WHEN** E2E 結果をユーザーへ提示する
- **THEN** ユーザーからのフィードバックを `tasks.md` の User Feedback Tasks に `[ ]` として追加する
- **THEN** 解消済みフィードバックは `[/]` として閉じる

### Requirement: 人間が UI/UX を手動確認できなければならない

システムは、自動 E2E とは別に、人間が起動して標準 UI 部品の入力、添付、送信、停止、usage、output handoff を確認できる手動 host app を持たなければならない（MUST）。manual host は独自 chat UI を実装せず、標準 UI を載せる枠として扱う。

#### Scenario: 手動 host app を起動する

- **WHEN** ユーザーが `just harness-up` を実行する
- **THEN** Floem 版の手動確認用 window が起動する
- **THEN** user は kcu 標準 UI の chat composer、message list、usage、host action intent を目視確認できる

#### Scenario: 3種類の host adapter を確認する

- **WHEN** user が `just harness-up egui`、`just harness-up floem`、`just harness-up gpui` を実行する
- **THEN** egui / Floem / GPUI の host frame をそれぞれ起動できる
- **THEN** egui / Floem / GPUI は v0.1.0 で同等の標準 UI 手動確認対象である
- **THEN** host frame は標準 UI を載せるだけで、独自の会話 UI を実装しない
- **THEN** Zed の Agent Panel / Prompt Editor を参考にした thread、mode、context、output handoff の見え方を比較できる
- **THEN** kcu core は `egui` / `eframe` / GPUI 型へ依存しない

#### Scenario: 手動 LLM 検証は local Ollama を使う

- **WHEN** user が手動 host で LLM 応答を確認する
- **THEN** host は local Ollama endpoint と model を使う
- **THEN** 有料 LLM API token を消費しない
- **THEN** 自動検証は LLM 実通信を行わない

#### Scenario: manual host は検証用出力を標準 UI から分離する

- **WHEN** user が Floem manual host を起動する
- **THEN** host は kcu 標準 UI の toolbar、thread、composer をそのまま確認できる
- **THEN** output JSON など検証用情報は標準 chat UI の表示領域に常時混ぜない
- **THEN** `debug: true` の場合だけ、output handoff hover から検証用情報を確認できる
- **THEN** hover は標準 chat UI の横幅や composer の表示を押し潰してはならない
- **THEN** `debug: false` の標準利用では検証用表示を出さない
- **THEN** manual host は独自の debug panel を作らず、crate 側の `debug: true` surface を起動するだけにする

#### Scenario: Floem manual host で日本語入力と resize を確認する

- **WHEN** user が Floem manual host の composer に日本語を入力する
- **THEN** IME の変換中文字列と確定文字列が composer に表示される
- **THEN** 行番号付き editor UI は表示されない
- **WHEN** user が window を resize する
- **THEN** 標準 chat UI の composer は上下の高さが狭くなっても消えない
- **THEN** thread は composer を押し出さず、必要な場合は thread 側が scroll する

#### Scenario: AI chat の streaming と thinking を確認できる

- **WHEN** provider から応答 chunk が届く
- **THEN** assistant response は完了まで待たず、chunk ごとに thread へ追記される
- **THEN** provider から thinking chunk が届く場合だけ、thinking は考慮ログとして表示される
- **THEN** `Thinking: false` の場合、標準 UI は thinking 表示を作らない
- **THEN** thinking は provider / model / endpoint / prompt size の dump ではなく、provider が返した考慮ログまたは明示された実行段階を表示する
- **THEN** thinking の詳細ログは response 本文に混入させず、専用表示に分離する
- **THEN** 応答完了後、thinking 詳細は折りたたまれる

#### Scenario: 状態表示を本文吹き出しと分離する

- **WHEN** provider または agent adapter が作業状態を返す
- **THEN** render model は thinking / execution / processing / read / editing / fetch を区別できる
- **THEN** 状態表示は回答本文の message body として扱われない
- **THEN** 状態表示は本文吹き出しの固定幅 contract の対象外である
- **THEN** 状態表示は短い label、実行中 indicator、必要に応じた折りたたみ可能な詳細ログを持てる
- **THEN** 失敗時はどの状態で失敗したかを表示できる
