# 実装・検証タスク

- [x] 1.1 公開KUC frameまたはadapter APIを変更せず、production adapterの公式Floem Vello featureをdefaultにした（`e5af641`、adapter37件・Floem54件・E2E9件、全7 scopeの依存込みlocked metadata成功）。
- [ ] 1.2 明示的TinySkia fallbackを維持・記録し、観測できたrenderer identityだけを受入artifactへ残す。
- [ ] 1.3 callbackのrecordIndex/uptimeを保持したまま、Native22 macOSで3 vendorすべてをfresh captureし、canonical RGBA sampleを2点取得する。
- [ ] 1.4 Linux Floemの実raster captureを追加し、反復composition/image結果を比較する。control labelのみの実行で代用しない。
- [x] 1.5 公開Vgerの6 frame後`None`観測を再現し、packing contractとは主張せず診断証拠として記録した（`tmp/verification/floem-vger-atlas-diagnostic/src/main.rs`、`tmp/verification/floem-vger-atlas-diagnostic/run.log`）。
- [ ] 1.6 renderer設定でpublic consumer、app、Storybookの確認を再実行する。
- [x] 1.7 GPUI依存の公開待ちは[Issue #7](https://github.com/HiroyukiFuruno/katana-chat-ui/issues/7)と上位作業台帳の`upstream_publication`で別に追跡した。公開修正版採用とLinux GPUI受入そのものは未完了であり、Floem修正でwaiveしない。
- [x] 1.8 `scripts/openspec validate floem-native-raster-backend --strict`を実行しexit 0を確認した。残りの項目は物理証拠が揃うまでpendingとする。
