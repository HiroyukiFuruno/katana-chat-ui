# 実装・検証タスク

- [x] 1.1 公開KUC frameまたはadapter APIを変更せず、production adapterの公式Floem Vello featureをdefaultにした（`e5af641`、adapter37件・Floem54件・E2E9件、全7 scopeの依存込みlocked metadata成功）。
- [x] 1.2 明示的TinySkia fallbackを維持し、Native22の別条件で実測した（`macos-run22/artifacts/floem-software/`）。既定Velloと混同せず、feature・binary SHAと公式環境変数の記録に限定する。
- [x] 1.3 source `95cdd81`のNative22で3 vendorの全5phaseをfresh captureし、callback時刻と2点canonical RGBA一致を照合、agentが原PNGを目視した（`macos-run22/native22-root-accepted.json`、`native22-diagnosis-receipt.json`）。標準egui比較器の失敗とworkflow全体failureは保持し、全要件達成とは扱わない。
- [x] 1.4 Native25 source `a7ed8ec` のLinux Floem実rasterで、nihongo変換・確定とtesuto未確定・取消の原画、private display候補画像、生state/traceをagentが照合した。`native25-remote/linux25-root-accepted.json`。control labelで代用せず、確定後/取消後の原PNG完全一致も確認した。
- [x] 1.5 公開Vgerの6 frame後`None`観測を再現し、packing contractとは主張せず診断証拠として記録した（`tmp/verification/floem-vger-atlas-diagnostic/src/main.rs`、`tmp/verification/floem-vger-atlas-diagnostic/run.log`）。
- [ ] 1.6 renderer設定でpublic consumer、app、Storybookの確認を再実行する。
- [x] 1.7 GPUI依存の公開待ちは[Issue #7](https://github.com/HiroyukiFuruno/katana-chat-ui/issues/7)と上位作業台帳の`upstream_publication`で別に追跡した。公開修正版採用とLinux GPUI受入そのものは未完了であり、Floem修正でwaiveしない。
- [x] 1.8 `scripts/openspec validate floem-native-raster-backend --strict`を実行しexit 0を確認した。残りの項目は物理証拠が揃うまでpendingとする。

- [x] 1.9 PR #8 discussion4369221140 のLinux取消検証不足を解消した。人間の今回限りの4ファイル診断例外に基づくproducerを含むNative25 source `a7ed8ec` で、全3ホストのEscape前actual marked=true、後のfresh同一Context Hideとactual marked=falseを観測した。GPUIはKUC frame/native marked rangeの両方を要求し、source別sequence/時刻境界と取消後画像まで独立照合した。parser22件の古い・別Context・false後true・欠落の拒否も維持する。`native25-remote/linux25-root-accepted.json`。GPUI表示重複のIssue #7はこの取消契約から分離して未完了のまま保持する。
