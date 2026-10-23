# Adapter headless evidence

`adapter_conformance` は共有 KUC の render、layout、Markdown、attachment、typed action を実行する。
egui、Floem、GPUI の host window は作成しない。このテストを3 hostの実行証拠として扱わない。

| 旧要件 | このcrateでの共有KUC実行 | `just headless-adapter-check` が実行するhost固有証拠 | 状態 |
| --- | --- | --- | --- |
| Header provider、New chat、history、debug/settings/outputなし | `kuc_frame_places_provider_and_toolbar_before_the_composer` | 各raster hostは同じframeを消費する | 共有KUCのみ |
| Provider popupとtyped selection | `kuc_dispatches_typed_toolbar_provider_and_submit_actions` | Floem `raster_view::host::tests`、GPUI input routing | 共有KUCのみ |
| Composer control、usage、resize bounds | `kuc_frame_keeps_usage_and_composer_bounded_on_resize` | egui texture frame、Floem raster image、GPUI TestAppContext | 共有KUCのみ |
| Thread width | `kuc_frame_keeps_user_message_bounded_to_agent_width` | 各adapterがKUC RGBAを描画する | 共有KUCのみ |
| Markdown code/table/raw HTML | `kuc_frame_renders_structured_messages_and_attachment_intent` | 各adapterがKUC RGBAを描画する | 共有KUCのみ |
| IME、CmdEnter、clipboard、pointer、wheel、focus | このcrateでは実行不可 | egui `raster::tests::input`、Floem `raster_view::host::tests`、GPUI `raster_view::tests::headless_window` とinput tests | host gate必須 |
| Host mount、managed history/output/provider | このcrateでは実行不可 | manual-host integration gates | このtest crateでは未検証 |
| Screenshot baselineとvisible window opt-in | `screenshots.rs` | screenshot matrix はegui artifact経路 | 構成検査のみ |

`just headless-adapter-check` は3 adapter libraryを実行する別の必須gateである。この文書はその代替ではない。
