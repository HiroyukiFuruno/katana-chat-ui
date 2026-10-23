# KUC 移行境界

## 現在の接続状態

`katana_chat_ui::kuc::ChatUiRuntime` がKUCの描画・TextSurface入力を統合し、
egui/Floem/GPUIの標準入口はraster adapterへ切り替わっている。
各manual hostは共通`ManagedChatHost`へ接続し、provider処理をUI threadから分離する。
ただし全hostの実操作・画像品質・リリースgateは未完了であり、移行完了とはしない。

`just harness-screenshot-matrix-check`は`headless-adapter-check`を前提にし、3 adapterの
ライブラリテストを必須にした。eguiの実texture uploadとGPUI TestWindowによる
描画・入力の検査、Floemのadapter入力・image提示の検査を整備中である。
後続の5画面PNG hash比較は現在egui経路であり、それ単独を3hostの画像証拠とは扱わない。
最新の実行結果と未検証範囲は`openspec/release-tasks.md`を参照する。

以下は共通runtimeを実装する前の設計調査記録である。「未接続」等の記述を現在の
実装状態の判定には使わず、公開KUC APIの選定理由と責務境界の記録として残す。

## 実装済みの範囲

`katana-chat-ui::kuc::ChatUiTreeBuilder` は `ChatUiSurface` から KUC
`render_model::UiTree` を構築する。会話、Markdown subset、composer、provider 選択、
provider control、context usage、履歴、output action を stable node ID と KUC host action
へ写像する。action の payload は `ChatUiAction` として復元できる。

この tree は標準 UI の構造・表示契約を一元化する。provider 選択はタイトル左、provider
control と context usage は composer 内、composer は thread の後に置く。TextArea は Enter
改行と Command+Enter 送信の KUC props を持つ。

## 未完了の接続

この変更だけでは egui、Floem、GPUI の native adapter は `UiTree` を描画しない。既存の
3 adapter は `ChatUiSurface` から直接 native widget を構築しており、共通treeへの接続は未対応である。
したがって API-only の tree 追加を multi-vendor 提供完了として扱わない。

KUC 0.3.6 の [`adapter_contract::WidgetAdapter`](https://docs.rs/katana-ui-core/0.3.6/katana_ui_core/adapter_contract/trait.WidgetAdapter.html)
は `UiTree` を受ける trait であり、[公開 feature](https://docs.rs/crate/katana-ui-core/0.3.6/source/Cargo.toml)
に Floem / GPUI はなく、その adapter 実装も公開されていない。純KUCの提供経路では、
KUCの共通描画と入力処理を使い、各native hostはOS入力の転送と画像提示を担当する。

## 共通raster rendererの実装先

KUC の [`raster_host::UiTreeSurfaceHost`](https://docs.rs/katana-ui-core/0.3.6/katana_ui_core/raster_host/struct.UiTreeSurfaceHost.html)
は tree の描画、hit test、cursor、host-action hit を公開する。[raster host source](https://docs.rs/crate/katana-ui-core/0.3.6/source/src/raster_host/mod.rs)
は windowing と GUI runtime を意図的に対象外とし、OS event を TextArea へ接続する API はない。
[`TextSurface`](https://docs.rs/katana-ui-core/0.3.6/katana_ui_core/text_surface/struct.TextSurface.html)
は selection、preedit、clipboard request の state/event を持つが、raster tree と統合されず、OS IME、clipboard
read/write、focus、key event を bridge しない。

これは移行不能という意味ではない。KCU共通moduleで`TextSurfaceLayout`を生成し、
`TextSurfaceFrameRecord`のtext・selection・caret・preeditを公開済み`Canvas` / `TextRenderer`で
描けば、3 hostをOS入力と画像提示だけにできる。不足しているのはこれらを統合する既製APIであり、
KCU内で実装するか、汎用入力描画としてKUCへ集約するかは設計上の選択である。

いずれの実装先でも、次を実装・検証する必要がある。

- `UiTree` の TextArea と `TextSurface` を結び、pointer、key、IME composition、selection、
  focus、frame を処理する raster interaction API。
- clipboard read/write と `TextSurfaceClipboardOperation` の host bridge。
- value change、submit、cancel、provider control 選択を型付きに返す generic event contract。
- egui、Floem、GPUIで実際の入力経路を通すadapter conformance検証。

現状の`UiTreeSurfaceHost`単体では、KCUのIME・clipboard・selectionを含む標準composerを
提供できない。共通rendererとhost bridgeを追加し、既存契約を検証してから移行完了と判定する。
KUC側へ集約する場合は別repo変更となるため、対象拡張の確認とIssue作成を先に行う。

## 接続する公開APIと検証範囲

共通runtimeはKUCの`TextSurfaceAction` / `TextSurfaceEvent`、layout、frame、Canvasを所有する。
hostからの入力を処理し、更新したcomposer text、型付きchat action、clipboard要求、
IME候補位置、描画結果を返す。チャット操作とprovider状態は引き続きKCUが管理する。
共通raster実装時はKUCの`raster-host` featureを有効にする。現在のtree変換のみの依存では
まだ有効化していない。

KCU内に実装する場合は、`PlatformTextRasterizer`からcompositionを含む`TextSurfaceLayout`を
作り、tree上のcomposer領域へtext・selection・caret・preeditを合成する。KUCへ集約する場合は、
node ID、bounds、`TextSurfaceFrameRecord`を結びつける汎用入力runtimeを公開候補とする。
どちらの場合も、nested scroll、DPI、clip、IME候補座標が一致することを実入力で確認する。

| host | 入力・OS連携 | 描画提示 |
| --- | --- | --- |
| egui 0.36 | `Event::Ime`、key/pointer、Copy/Cut/Paste、`PlatformOutput::ime`と`IMEOutput` | textureへRGBAを転送し画像を提示 |
| Floem 現行pin | `EventListener`のIME/key/pointer/focus、`set_ime_cursor_area`、`Clipboard` | custom `View::paint`などでRGBAを提示 |
| GPUI 0.2.2 | `EntityInputHandler`、`Window::handle_input`、UTF-16選択範囲とmarked text、clipboard | custom Elementと画像提示APIを接続 |

これらは公開ソースを確認した実装候補であり、移行後の動作検証済みAPI一覧ではない。
GPUIの現行composerは静的Divであり、実入力接続の追加が必要となる。native frameworkの
チャットwidgetを残して純KUC移行済みとはしない。

共通runtimeでは、日本語preedit/commit、grapheme単位選択、複数行・折返し、copy/cut/paste、
IME中の送信抑止、Command+Enter、scroll、provider menu、出力actionを検証する。
各hostでは実入力経路とIME候補位置・focusを検証し、既存のcontract検査を維持する。
既存ソースの文字列一致だけを、新経路が正しい根拠として使わない。

## attachment extension の扱い

attachment の file picker は既存仕様の v0.3 後続 extension であり、今回の純 KUC tree の
完了条件には含めない。今回の tree は既に `ChatUiSurface` にある attachment の text、resource
metadata、unsupported reason と remove action を写像する。将来 file picker を追加するときは、
上記 host bridge と独立した `Attach` action の platform extension として設計する。
