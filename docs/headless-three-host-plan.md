# 3 host headless screenshot 実装計画

## 目的と判定

`openspec/v0-1-0-scope.md` の0014/0015の DoD は変更しない。0014 は三つの host が同じ render model と header/thread/composer の責務を持ち、三 host の screenshot で同じ操作対象に見え、host 独自 debug/output UI がないことを求める。0015 は通常検証で native window を開かず、headless screenshot で三 host の主要 layout を検証し、composer 欠落・provider 誤表示・debug 混入を画像差分で検知することを求める。

scope の Status: 完了は達成証拠ではない。`openspec/release-tasks.md` は0014/0015を未完了としており、旧 native green、共通 UiTree、source 文字列、各 adapter の texture/image だけを完了根拠に使わない。

## 現在ある真の証拠

- Egui の `scripts/screenshot/src/executor_harness.rs` は `egui_kittest::Harness::render()` の最終 RGBA を PNG 化する。ただし Egui の in-process headless 経路だけであり、三 host screenshot ではない。
- Floem の raster tests は `peniko::Image`、GPUI の test window は `RenderImage` と pointer/IME を検査する。いずれも adapter が KUC pixels と native input を橋渡しする証拠であり、OS compositor が提示した最終 window framebuffer の証拠ではない。
- `scripts/screenshot/src/native_capture.rs` は三つの actual manual host を build/spawn/capture/kill する。これは標準 mount を通る実 window 採取の起点だが、54--55 行で macOS 以外を拒否する。
- `justfile` 142--147 行の visible native check は `KCU_ALLOW_VISIBLE_WINDOWS=1` と Darwin を要求し、Linux では skip する。CI の `ui-harness` は macOS で `manual-ui-check`、`host-e2e`、Egui screenshot matrix のみを実行する。

従って現証拠は0014の adapter 経路を部分的に支持するが、0015の三 host final screenshot には不足する。

## Xvfb を使う完成経路

Linux CI で Xvfb の仮想 X11 display を起動し、各 actual manual host の top-level window を順に表示して採取する。CI の仮想 display だけを使うため、通常の開発者 desktop と通常 gate に native window を開かない。これは公開 offscreen API に依存せず、各 framework の実 platform renderer と OS compositor を通る。

1. Xvfb `:99` を固定 viewport で起動し、`DISPLAY=:99` を各 manual host と capture tool に渡す。Wayland を選ばないよう X11 を明示し、Mesa software rendering の環境を固定する。
2. `native_capture.rs` を Mac Quartz と Linux X11 の capture backend に分ける。Linux backend は title で window ID を解決し、`xwd -id` と PNG 変換、または同等の window-id capture を使う。root display 全体ではなく対象 window の ID、寸法、PNG を検証する。
3. host ごと・scenario ごとに `egui`、`floem`、`gpui` の別 PNG を保存し、client size、非空 RGBA、scenario 固有の画像差分を検査する。framework 間の raw hash 一致は要求しない。artifact と stderr を CI から保存する。
4. Linux 専用の required recipe/job を追加する。既存の macOS opt-in recipe や `harness-native-screenshot-check` を非 Darwin で成功 skip にしたまま0015の証拠と呼ばない。

## 実行前提と platform リスク

CI は Xvfb、X11 query/input、window capture/PNG conversion、フォント、Mesa EGL/GL/Vulkan と各 framework の X11/keyboard 依存を install する。具体パッケージ名は採用 runner で lock し、job 内で version と利用 binary を記録する。GPUI 0.2.2 は default feature に `x11` と `wayland` があり、registry `gpui-0.2.2/src/platform.rs:137-150` は `DISPLAY` があれば X11 を選ぶため、Xvfb 候補として成立する。

Floem は最優先の実証対象である。現 manifest は `floem 0.2.0` の `vger` feature を選び、registry `floem_vger_renderer-0.2.0/src/lib.rs:35-63` の `VgerRenderer::new` は 50--52 行で `adapter.get_info().device_type == DeviceType::Cpu` を `only cpu adapter found` として拒否する。ただし親による呼出元の再検証では、`floem-0.2.0/src/renderer.rs:126-134` がVger初期化失敗後に標準搭載のTinySkiaRendererへ移行する。89行の `FLOEM_FORCE_TINY_SKIA=1` も上流標準機能であり、CPU adapter拒否だけをFloem起動不能の根拠にはできない。新しいrenderer依存や製品差替えは不要である。Xvfb上で標準TinySkia経路とwindow captureを実測し、GPU経路と同じ最終画像契約を確認する。

## 操作、fixture、検証順

現 `native_capture.rs` は起動後の一枚だけを採取し、OS pointer/key driver を持たない。0014の操作対象まで検査する次段では、X11 の window-id 指定 input で composer click、text/IME、modifier+Enter、provider dropdown/pointer を actual window に送る。modifier と scale/client origin は framework ごとに実測し、Command を Ctrl と仮定しない。

再現可能な表示には CI 専用の固定 screenshot scenario が必要である。これは各 manual host の標準 KUC mount を通し、保存済み history や provider network の時刻差を除く UI 描画 fixture に限定する。fixture は四 provider の実通信、ACP initialize/prompt/cancel、履歴永続化、output 物理 I/O の E2E を代替しない。それらは別の real provider/host gate で直接確認する。

検証順は、(1) Linux dependency と Xvfb 起動、(2) Egui/GPUI/Floem の actual window 作成と client-size capture、(3) Floem adapter-kind 判定、(4) 固定 scenario の三 host PNG/assert/artifact、(5) 実 OS input と状態遷移 screenshot、(6) 既存の real provider E2E と分離した最終 gate、である。

## 実装・検証状況（2026-09-05）

### 最新の実測（以下の初期記録より優先）

- egui/Floemの既存5scenarioを実ウィンドウから取得した。10枚すべてrequest寸法と一致し、同scenarioの両host PNG hashが一致した（`tmp/native-matrix-evidence.json`）。ただし出力欄の重複と狭幅header欠落を検出したためbaselineには未採用。
- GPUIはMesa22.3.6では標準quadだけでも黒一色。prepaint/paint到達・正しい寸法を確認し、resizeとLavapipe ICD明示でも変わらなかった。同binaryがDebian backports Mesa25.0.7-2~bpo12+1では正しい色を描画した（`tmp/gpui-mesa25-probe.log`）。正式Dockerfileへこの版を固定した。通常KUC画面の再buildと3host matrixは残る。
- 3manualはrequestのlogical viewportを受け取り、幅320以上・高さ240以上・各辺4096以下を検証する。Xvfbを4096角として縦長scenarioもdisplay内に収める。UI fixtureはprovider通信の証拠に使わない。

Linux X11 capture backend、3manual hostの明示surface fixture入口、`--native-request`接続を実装した。全体compileと三host画像は未検証である。baselineなしのnative captureはartifact採取だけと表示し、0015 gateの完了には使わない。

`linux/run-xvfb.sh`はdisplayfdとxdpyinfoを使って実readyを待ち、通常desktopにwindowを出さない。Docker内の実GL/Vulkan probeはexit0、子失敗exit17の伝播も実測した。Mesa llvmpipe/OpenGL4.5/Vulkan CPU deviceが利用可能である（`tmp/headless-xvfb-wrapper.log`）。Linux workspace compileは`tmp/linux-workspace-check.log`で進行中。
