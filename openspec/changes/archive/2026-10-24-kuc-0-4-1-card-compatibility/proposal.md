## Why

KUC 0.4.2 はCard子インデントを公開したが、描画座標とhit座標を同時に移動するため、KCUの既存560px本文hit契約とスクリーンショット基準を同時に満たせない。また、Markdownの `monospace` model契約を保ったcode spanは、本文と同じ物理font faceで描画する既存基準と両立しない。KUC v0.4.3 はこれらの公開APIに加え、bidi本文と分割されたinline-code runでselection、link hit、decoration、段落区切りの契約を修正する。KCUは公開registryでv0.4.3を取得できるまで未公開の上流修正を採用せず、公開KUCだけへの依存を維持する。

## What Changes

2026-10-23追記: v0.4.3を公開registryから採用し、既存5画像と幅契約は維持できたが、consumerのbidi/soft wrap回帰で#104/#105が再現した。要件と品質基準は維持し、修正版v0.4.4を上流で公開してから採用する。未公開の上流コードをKCUへ直接取り込まない。

- KUCのCard子インデント、本文hit outset、code raster設定を公開render-model/renderer APIで指定できる上流修正を追跡する。
- 公開済みKUC修正版を取得して、KCUのwide/narrowメッセージで既存の幅・描画・hit test・Markdown code model契約を再検証する。
- KUC v0.4.3のbidi選択契約をKCUの分割inline-code表示で受け入れ、CR、LF、CRLF、U+2028、U+2029を含むcopyとhitの回帰を確認する。
- 既存スクリーンショット基準、coverage、lint、AST検査を変更せずにrelease gateを再実行する。
- 共通composerのforward Deleteを公開KUCの文字境界移動・選択・入力操作で組み立て、末尾のdraftと現在の選択範囲を保護する。関連するBackspaceも選択範囲を削除し、未選択時の従来grapheme削除は維持する。
- 標準host surfaceのoutput actionは実装済み操作だけを表示し、未対応OpenPreview等を有効な操作として提示しない。公開libraryのintentと外部host callback、保存payload/status、既存の高度preview対象外は維持する。

## Capabilities

### New Capabilities

- なし。

### Modified Capabilities

- `pure-kuc-chat`: KUC更新時も既存メッセージの視覚・幅・操作契約を維持し、公開APIが不足する場合は上流公開までリリースを完了扱いにしない。

## Impact

- 上流: [katana-ui-core#85](https://github.com/HiroyukiFuruno/katana-ui-core/issues/85)、[katana-ui-core#86](https://github.com/HiroyukiFuruno/katana-ui-core/issues/86)、[katana-ui-core#98](https://github.com/HiroyukiFuruno/katana-ui-core/issues/98)、[katana-ui-core#100](https://github.com/HiroyukiFuruno/katana-ui-core/issues/100) と公開crate v0.4.3。
- 下流: `crates/katana-chat-ui/src/kuc/content.rs`、KCU dependency/lockfiles、5画面スクリーンショットmatrix、release gate。
