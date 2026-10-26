## ADDED Requirements

### Requirement: KUC 0.5移行でも既存の描画と入力契約を維持する

KCU SHALL 公開registryから取得したKUC0.5以降の採用時、既存のUnicode入力、改行、測定・hit、message row/scroll、3adapter共通描画の契約と必須品質ゲートを維持する。公開APIが返すKUC型の依存identityが変わる場合、KCU SHALL その互換性境界に合わせて公開versionを選択する。

#### Scenario: 公開KUCの文字測定が更新される
- **WHEN** hard-break、RTL/LTR終端、soft-wrap、text layout高さの処理が変わる公開KUCへ更新する
- **THEN** KCUは既存のcore/geometry/Unicode・adapter conformance・画像/native・coverage検査を維持して結果を回収する
- **AND** 失敗をbaselineやtimeoutの緩和で隠さず、互換修正か不採用根拠を記録する

#### Scenario: source検証と公開版受入を分ける
- **WHEN** 現sourceのapp/Storybookとprovider検証が成功する
- **THEN** KCUはそのsourceと実行物に結び付けた範囲だけを合格として記録する
- **AND** 修正版依存とKCU新公開版のconsumer/app/Storybook受入は実物を確認するまで未完了のまま維持する
