## Context

ACP 公式仕様では、prompt content は text、image、embedded resource などの content block で表現される。kcu の添付と path drop は、この構造へ写像できる形で保持する。

`katana-chat-ui` core は UI framework を知らない。ただし katana-chat-ui product は UI 込みで提供する。標準 UI は framework 別 crate で提供し、v0.1.0 では egui / Floem / GPUI を同等の手動確認対象とする。

API だけを使って独自 UI を作る path は残すが、これは標準 UI に満足できない利用者向けの拡張手段であり、標準の利用方法ではない。

## Goals

- kcu core から `egui` と親アプリ固有語を取り除く。
- 標準的な AI chat 入力を v0.1.0 の contract として固定する。
- message role、attachment、Markdown subset、theme、SVG icon、usage 表示を render model で表現する。
- kcu 提供の標準 Floem UI 部品を実装する。descriptor / API だけでは完了としない。
- button は SVG を描画し、host から渡された SVG override を反映できる。
- UI 文言は text catalog から取得し、MVP が英語のみでも後続 locale を追加できる。
- vendor / provider capability に応じて composer 周辺の UI affordance を出し分けられる。
- 生成物、差分候補、file 候補、tool 結果、承認待ち操作を output として分類し、host action intent として利用側へ渡す。
- kcu 自身も output を読むが、debug JSON や検証用差分カードを標準 thread に常時混ぜない。
- file / diff / tool / permission は標準 UI では host へ返す付随情報として扱い、v0.1.0 の詳細 UI は劣後してよい。ただし後から標準 UI に追加できる output extension interface は v0.1.0 で固定する。
- `options: { debug: true }` で右端の output hover を crate 側標準機能として提供する。
- 右上トグルから開く標準設定画面を提供する。
- 入力欄の `/` 起動 contract を提供する。
- 編集できる AI 作業者としての提供元（agent provider）と、Ollama のようなローカルモデル実行基盤（local model runtime）を区別する。
- rendering implementation は core state を読むだけにし、provider 接続や secret を直接扱わない。

## Non-Goals

- secret store、provider usage / account usage の取得、ACP connection setup、MCP server 設定 — v0.2.0。
- prompt / skill / workflow / command / hook の実エントリ供給 — v0.3.0。
- Claude Code / Codex / GitHub Copilot / Bedrock / Vertex AI adapter — v0.3.0。
- document generation、translation overlay、autofix diff の実処理 — v0.5.0 以降または host 側。
- file 書き込み、diff 適用、tool 実行、権限承認の実行 — host 側。
- file / diff / tool result の高品質な詳細ビュー — v0.2.0 以降。ただし extension interface は v0.1.0。

## Architecture

```
katana-chat-ui
  session.rs          ChatSession / ChatTurn lifecycle
  message.rs          ChatMessage / MessageRole / MessageStatus
  input.rs            ChatInputDraft / Attachment / PathDropRequest
  markdown.rs         MarkdownSubset / ParsedBlock
  theme.rs            ThemeTokens / IconRegistry / SvgIcon
  usage.rs            ContextUsageSnapshot / AccountUsageSnapshot
  settings.rs         ChatSettingsModel / SettingsStoreRef / SettingsMergeIntent
  slash_launcher.rs   SlashCommandEntry / CommandLaunchIntent
  output.rs           ChatOutput / ChatOutputKind / HostActionIntent
  debug_surface.rs    DebugSurface / OutputHoverSurface
  render_model.rs     ChatRenderModel
  surface.rs          ChatUiSurface / standard UI surface
  text.rs             TextCatalog / locale text
  vendor_ui.rs        VendorUiCapabilities

katana-chat-ui-floem
  widget.rs           FloemChatView
  panel.rs            internal panel layout
  input.rs            composer widget
  message.rs          message list widget
  usage.rs            usage widget
  settings.rs         settings toggle and settings panel
  slash_launcher.rs   slash launcher popup
  icons.rs            SVG icon rendering

tools/e2e-host-app
  Cargo.toml          publish = false。workspace member にしない
  src/main.rs         kcu を downstream dependency として取り込む最小 host app
  tests/e2e.rs        起動、描画、入力、添付、usage 表示を検証する E2E

tools/manual-host-egui
  Cargo.toml          publish = false。workspace member にしない
  src/main.rs         人間が起動して UI/UX と local Ollama を確認する egui host app
  src/*               egui 実装で Zed 参考の thread / mode / context / output handoff を確認する

tools/manual-host-floem
  Cargo.toml          publish = false。workspace member にしない
  src/main.rs         kcu 標準 Floem UI を載せるだけの host frame

tools/manual-host-gpui
  Cargo.toml          publish = false。workspace member にしない
  src/main.rs         kcu を取り込む GPUI host adapter MVP
```

## Interface Boundaries

各レイヤーの境界は依存性逆転の原則（Dependency Inversion Principle）で固定する。上位レイヤーは下位の具象型を知らず、下位レイヤーが上位の差し込み口の約束へ合わせる。

- `ChatUiSurfaceProvider`: core state から標準 UI surface を取り出す境界。UI framework は `ChatSession` の内部状態を読まない。
- `VendorControlProvider`: vendor facts と実行時 state から vendor control render model を作る境界。標準 widget は vendor ID 直書き分岐を持たない。
- `FloemPanelSlotView`: Floem panel に載せる部品の境界。panel は thread / composer / output extension の中身を知らず、slot として配置するだけにする。
- `FloemActionIconButton`: SVG icon button の最小部品。composer 内の attach / stop / send と後続拡張ボタンは同じ atom を使う。
- `FloemVendorControlsView`: provider / model / thinking / permission などの可変値を composer 内の control row として描画する。composer は入力文字列の編集と control row の配置だけを持ち、候補値の決定は `VendorControlProvider` に委譲する。
- `HostActionIntent`: output の操作境界。kcu は agent 由来の操作意図を所有し、物理的な file 書き込み、diff 適用、tool プロセス実行は adapter / host に委譲する。
- `OutputExtensionSlot`: file / diff / tool result / permission request の詳細 UI を後から追加する境界。thread 本体は output 詳細の具象 UI を知らない。
- `DebugSurfaceProvider`: `options.debug` から右端 output hover surface を生成する境界。manual host 固有の debug panel を作らない。
- `SettingsSurfaceProvider`: 右上トグルから開く標準設定画面を生成する境界。theme、locale、placeholder、SVG icon override、provider 表示順、composer behavior を v0.1.0 で扱い、provider connection、prompt、skill、workflow、command、hook、MCP は後続 section として追加できる。
- `SlashLauncherProvider`: composer 内の `/` 入力から候補を生成する境界。v0.1.0 は launcher contract と host-provided entry を扱い、provider / adapter 由来の prompt、skill、workflow、command、hook、MCP は v0.3.0 の capability catalog から供給する。
- `AttachmentSourceResolver`: OS file、host logical resource、virtual attachment を解決する境界。kcu core は filesystem や host 固有型を直接読まない。
- `ProviderRuntimeRole`: 表示上の提供元を、編集や terminal 実行を行える agent provider と、local model runtime に分類する境界。Ollama は local model runtime とし、編集権限や permission UI を持つ agent provider とは扱わない。
- `ChatAgentEvent`: agent adapter から返る `Chunk`、`ThinkingChunk`、`Output`、`Complete`、`Failed` の境界。session はこの event だけを見て本文、thinking、output、完了、失敗を更新する。

この境界より内側で増える具体実装は、既存の interface を満たす限り panel 側の変更を不要にする。新 vendor の追加や vendor 側の仕様変更は、`VendorFactRegistry`、adapter、または該当 widget の追加で閉じる。

## Zed Reference Direction

Zed は Agent Panel で thread、draft prompt、model selector、profile selector、tool permission を明示状態として扱う。公式ドキュメントでは、読み取り、検索、編集、terminal 実行は agent tools として分かれており、変更後は review changes で file 数、行数、diff を確認できる。

v0.1.0 では Zed の実装を直接取り込まない。代わりに、次の設計観点を kcu の contract に反映する。

- thread と draft は分離した state として扱う。
- context 追加は添付や path drop だけでなく host action として表現できる。
- output は chat message の文字列に埋め込まず、host が処理できる候補として別に渡す。
- ただし kcu は output を無視しない。file 候補、diff 候補、tool 結果、permission request は render model と host action intent で保持し、標準 thread の会話表示とは分離する。
- mode / profile / permission / tool result は後続拡張しやすい enum と action intent で表す。
- local model runtime は agent provider と分離する。Ollama は agent provider が使う低コスト runtime であり、Zed / Codex のような file edit / terminal tool 実行能力を直接持つとは扱わない。

具体的な参照対象は次の通り。

- `crates/agent_ui/src/conversation_view.rs`: composer、send / stop、message edit、queue / interrupt、thinking 表示、tool card。
- `crates/acp_thread/src/acp_thread.rs`: thread entry、message chunk、thought chunk、tool call status、cancel、rewind。
- `crates/agent/src/agent.rs`: session 保存、thread 復元、draft 復元。
- `crates/agent_ui/src/message_editor.rs`: editor、placeholder、入力イベント。
- `crates/agent_ui/src/mention_set.rs`: file、image、path、context 追加。
- `crates/agent_settings/src/agent_settings.rs`: model、thinking、permission 設定。
- `crates/acp_thread/src/diff.rs` / `crates/acp_thread/src/terminal.rs`: diff、terminal、tool 実行結果。

## Attachment Model

`Attachment` は次の種類を持つ。

- `Text`: ユーザーが直接入力した補助テキスト。
- `FileResource`: host が読み取った file URI、MIME type、text content、size を持つ。
- `ImageResource`: MIME type と binary data reference を持つ。
- `Unsupported`: capability 不足や file size 超過を UI に返すための状態。

path drop と OS file picker 時、kcu は直接 filesystem を読むのではなく、host callback に読み取りを依頼する。host callback は許可済みの file content だけを `FileResource` として返す。

host application 内の論理リソースを drop した場合も、kcu は host resource ID をそのまま扱うだけにし、`AttachmentSourceResolver` で `EmbeddedResource` または `FileResource` に変換する。

## Markdown Subset

Markdown parser は `comrak` を暫定採用する。対応する syntax は見出し、段落、強調、取り消し線、link、自動 link、inline code、code block、blockquote、通常 list、番号付き list、task list、table とする。raw HTML、script、raw style は実行可能 HTML として描画せず text として扱う。画像 Markdown は v0.1.0 では画像として描画せず、alt と link 相当の安全な inline 表示にする。

Floem 実装は `comrak` の AST を直接描画しない。core 側の `MarkdownBlock` / `InlineSegment` に変換し、将来 `katana-document-viewer` に差し替える境界を残す。

v0.1.0 の見た目は最低限でよいが、code fence の開始と終了は parser の block で判断する。単純な文字列分割で ``` を探して code block を閉じる実装は禁止する。

## Theme and Icon Contract

theme は以下の token を受け取る。

- background / surface / user bubble / assistant bubble / border / muted text / accent
- spacing scale
- typography role
- danger / warning / success

button icon はすべて SVG asset id で表す。host は `IconRegistry` で同じ id を上書きできる。標準 UI はボタンを text fallback ではなく SVG として描画する。

## Text Catalog and i18n

UI 文言は `TextCatalog` を通して取得する。MVP は英語だけでもよいが、locale ごとに文言セットを追加できる構造にする。UI 実装内に固定文言を散らさない。

## Vendor Facts and UI Controls

vendor / provider ごとの差分は、公式ドキュメントを根拠にした `VendorFactRegistry` と、利用側の選択状態である `VendorUiState` から生成する。標準 UI は `VendorControlRenderModel` だけを読み、widget 内で `vendor_id == "..."` のような文字列分岐をしない。

`VendorFactRegistry` は vendor ID、表示名、接続方式、公式URL、確認日、根拠メモを保持する。model、thinking、permission、tools、web search、usage、attachment は `Supported`、`Unsupported`、`RequiresAdapter`、`RequiresHost`、`Unknown` のいずれかで管理する。`Supported` の capability は公式URLを必須にする。

v0.1.0 の低コスト runtime 確認は、agent provider が Ollama を runtime として使う形を対象にする。Ollama は endpoint、model、thinking、token usage を runtime 設定として持てる。ただしこれは runtime capability であり、file edit や terminal execution を行える agent provider ではない。Ollama に公式根拠がない permission mode は表示しない。

agent provider selector は利用可能な ACP / CLI adapter を対象にする。local model runtime は、その agent provider が必要とする model runtime 設定として扱う。v0.1.0 では Ollama を agent provider selector に出さない。

## Usage Surface

v0.1.0 では usage を取得しないが、表示する枠を定義する。

- `ContextUsageSnapshot`: used tokens、max tokens、percentage、status。
- `AccountUsageSnapshot`: auth method、account label、organization label、plan label、quota rows。
- unknown な provider は `Unavailable(reason)` を返し、UI は空表示ではなく「取得不可」状態を持つ。

## Output Handoff Contract

`ChatOutput` は次の種類を持つ。

- `Text`: host がコピーや引用に使える生成テキスト。
- `Code`: language metadata 付きの code block。
- `FileCandidate`: host が保存先、内容、MIME type を確認してから書き込む file 候補。
- `DiffCandidate`: host が対象 path と unified diff を確認してから適用する差分候補。
- `ToolResult`: tool 実行結果の表示用 output。実行プロセス自体は host または adapter が行い、kcu は結果の構造と表示 contract を所有する。
- `PermissionRequest`: host が承認 UI を出すための操作要求。

kcu は `HostActionIntent` として copy / open preview / create file / apply diff / approve / reject を返す。
kcu core は file system へ直接書き込まず、diff を直接適用せず、tool プロセスを直接実行しない。agent adapter / host が side effect を行い、その結果を kcu の event / output contract へ戻す。

標準 UI は output を会話本文として混ぜない。assistant message に紐づく output は render model と host action intent として公開し、利用側が差分ビュー、ファイルビュー、実行結果ビューへ渡せるようにする。

`debug: true` は output JSON を検証するための補助表示であり、標準 UI の常設レイアウトに混ぜない。表示は右端 hover で開く output handoff とし、標準 thread / composer の幅を奪わない。通常時は何もない右端領域に近づいた時だけ、右から左へ重なる補助表示を出す。

設定画面は debug surface ではない。右上トグルから開き、theme、locale、placeholder、SVG icon override、provider 表示順、composer behavior を扱う。保存先は host が non-null な settings JSON reference として渡す。kcu は typed settings を指定 JSON へ merge する intent を返し、settings 保存先未指定は invalid configuration として扱う。

`/` launcher は composer の編集状態を壊さない。候補は `SlashCommandEntry` として kind、display label、source、enabled state、disabled reason を持つ。選択結果は `CommandLaunchIntent` として host / provider / adapter へ渡す。未対応の skill / workflow / command を実行可能に見せない。

## Manual Agent Verification

手動検証は agent provider を対象にする。Ollama は agent provider が使う local runtime として扱い、manual host が direct `OllamaProvider` を呼び出して provider selector に出す構造にはしない。

VT Code、Codex CLI、Claude Code、GitHub Copilot、OpenCode のような agent provider を確認する場合は、それぞれの adapter が file edit / terminal / permission / output を event として返す。kcu は本文、thinking、output を分けて読み、output 詳細は標準 thread に常時混ぜず、output extension surface または host action intent として扱う。

自動検証は LLM request を送らない。unit test と host E2E は provider label、mock agent event、render model を使い、Ollama / cloud LLM への実通信を行わない。

## Verification

- `cargo tree -p katana-chat-ui | grep -E "floem|egui|vello|eframe"` が空。
- `rg -n "egui::|<<KATANA" crates/katana-chat-ui README.md` が空。
- attachment / path drop / markdown subset / role visual model / theme / SVG override / usage snapshot の unit test がある。
- output handoff の unit test がある。
- Floem 標準 UI implementation の smoke test がある。
- `katana-chat-ui-floem` が `floem` dependency と実 widget を持つことを lint で検査する。
- OpenSpec が「標準 UI 提供」「API-only は拡張手段」「manual host は枠だけ」を明記していることを lint で検査する。
- 実動作検証用 app は `crates/` に含めず、`tools/e2e-host-app/` などの非公開 host fixture に置く。
- E2E は kcu を downstream dependency として実際に取り込み、host application 視点で起動・描画・入力・添付・usage 表示を検証する。
- 手動確認用 app は `just harness-up` で Floem 版を起動できる。
- `just harness-up egui`、`just harness-up floem`、`just harness-up gpui` で egui / Floem / GPUI 版を同等の標準 UI 確認対象として起動できる。
- `just manual-ui-check` は egui / Floem / GPUI の3 host adapter を check する。
- 通常の screenshot gate は headless runner のみを使う。実ウィンドウを開く native capture は `KCU_ALLOW_VISIBLE_WINDOWS=1` があるときだけ実行する。
