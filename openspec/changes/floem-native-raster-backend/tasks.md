# 実装・検証タスク

- [x] 1.1 公開KUC frameまたはadapter APIを変更せず、production adapterの公式Floem Vello featureをdefaultにした（`e5af641`、adapter37件・Floem54件・E2E9件、全7 scopeの依存込みlocked metadata成功）。
- [x] 1.2 明示的TinySkia fallbackを維持し、Native22の別条件で実測した（`macos-run22/artifacts/floem-software/`）。既定Velloと混同せず、feature・binary SHAと公式環境変数の記録に限定する。
- [x] 1.3 source `95cdd81`のNative22で3 vendorの全5phaseをfresh captureし、callback時刻と2点canonical RGBA一致を照合、agentが原PNGを目視した（`macos-run22/native22-root-accepted.json`、`native22-diagnosis-receipt.json`）。標準egui比較器の失敗とworkflow全体failureは保持し、全要件達成とは扱わない。
- [ ] 1.4 Linux Floemの実raster captureを追加し、反復composition/image結果を比較する。control labelのみの実行で代用しない。
- [x] 1.5 公開Vgerの6 frame後`None`観測を再現し、packing contractとは主張せず診断証拠として記録した（`tmp/verification/floem-vger-atlas-diagnostic/src/main.rs`、`tmp/verification/floem-vger-atlas-diagnostic/run.log`）。
- [ ] 1.6 renderer設定でpublic consumer、app、Storybookの確認を再実行する。
- [x] 1.7 GPUI依存の公開待ちは[Issue #7](https://github.com/HiroyukiFuruno/katana-chat-ui/issues/7)と上位作業台帳の`upstream_publication`で別に追跡した。公開修正版採用とLinux GPUI受入そのものは未完了であり、Floem修正でwaiveしない。
- [x] 1.8 `scripts/openspec validate floem-native-raster-backend --strict`を実行しexit 0を確認した。残りの項目は物理証拠が揃うまでpendingとする。

- [ ] 1.9 PR #8 discussion4369221140 のLinux取消検証不足を解消する。Escape前の実host marked=true、Escape後のfreshな同一InputContext解除と実host marked=falseを観測し、古い・別Context・false後のtrue・状態欠落を拒否する。キー種別から状態を推定しない。商用コードへの読み取り専用診断4ファイル追加は人間が今回限りの例外を明示承認済み。実状態producerと修正後実IME・原画の証拠が揃うまで未完了とする。
