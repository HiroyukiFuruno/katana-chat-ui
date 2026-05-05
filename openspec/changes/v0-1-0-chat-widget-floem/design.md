## Context

ACP 公式仕様では、prompt content は text、image、embedded resource などの content block で表現される。kcu の添付と path drop は、この構造へ写像できる形で保持する。

`katana-chat-ui` は core crate として UI framework を知らない。Floem は reference implementation であり、egui app は host 側 adapter で同じ render model を描画する。

## Goals

- kcu core から `egui` と親アプリ固有語を取り除く。
- 標準的な AI chat 入力を v0.1.0 の contract として固定する。
- message role、attachment、Markdown subset、theme、SVG icon、usage 表示を render model で表現する。
- rendering implementation は core state を読むだけにし、provider 接続や secret を直接扱わない。

## Non-Goals

- secret store、account usage の取得、ACP connection setup — v0.2.0。
- Claude Code / Codex / GitHub Copilot / Bedrock / Vertex AI adapter — v0.3.0。
- document generation、translation overlay、autofix diff — v0.5.0 以降。

## Architecture

```
katana-chat-ui
  session.rs          ChatSession / ChatTurn lifecycle
  message.rs          ChatMessage / MessageRole / MessageStatus
  input.rs            ChatInputDraft / Attachment / PathDropRequest
  markdown.rs         MarkdownSubset / ParsedBlock
  theme.rs            ThemeTokens / IconRegistry / SvgIcon
  usage.rs            ContextUsageSnapshot / AccountUsageSnapshot
  render_model.rs     ChatRenderModel

katana-chat-ui-floem
  panel.rs            ChatPanelView
  input.rs            ComposerView
  message.rs          MessageListView
  usage.rs            UsageMeterView

tools/e2e-host-app
  Cargo.toml          publish = false。workspace member にしない
  src/main.rs         kcu を downstream dependency として取り込む最小 host app
  tests/e2e.rs        起動、描画、入力、添付、usage 表示を検証する E2E
```

## Attachment Model

`Attachment` は次の種類を持つ。

- `Text`: ユーザーが直接入力した補助テキスト。
- `FileResource`: host が読み取った file URI、MIME type、text content、size を持つ。
- `ImageResource`: MIME type と binary data reference を持つ。
- `Unsupported`: capability 不足や file size 超過を UI に返すための状態。

path drop 時、kcu は直接 filesystem を読むのではなく、host callback に読み取りを依頼する。host callback は許可済みの file content だけを `FileResource` として返す。

## Markdown Subset

対応する syntax は code block、inline code、blockquote、ordered list、unordered list、link、paragraph に限定する。HTML block、script、raw style は描画対象外とする。

## Theme and Icon Contract

theme は以下の token を受け取る。

- background / surface / user bubble / assistant bubble / border / muted text / accent
- spacing scale
- typography role
- danger / warning / success

button icon はすべて SVG asset id で表す。host は `IconRegistry` で同じ id を上書きできる。

## Usage Surface

v0.1.0 では usage を取得しないが、表示する枠を定義する。

- `ContextUsageSnapshot`: used tokens、max tokens、percentage、status。
- `AccountUsageSnapshot`: auth method、account label、organization label、plan label、quota rows。
- unknown な provider は `Unavailable(reason)` を返し、UI は空表示ではなく「取得不可」状態を持つ。

## Verification

- `cargo tree -p katana-chat-ui | grep -E "floem|egui|vello|eframe"` が空。
- `rg -n "egui::|<<KATANA" crates/katana-chat-ui README.md` が空。
- attachment / path drop / markdown subset / role visual model / theme / SVG override / usage snapshot の unit test がある。
- Floem reference implementation の smoke test がある。
- 実動作検証用 app は `crates/` に含めず、`tools/e2e-host-app/` などの非公開 host fixture に置く。
- E2E は kcu を downstream dependency として実際に取り込み、host application 視点で起動・描画・入力・添付・usage 表示を検証する。
