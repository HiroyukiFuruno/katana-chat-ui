# Pure KUC chat renderer contract

## ADDED Requirements

### Requirement: 公開KUC raster frameの安定性

Native host SHALL、入力callbackのrecordIndex/uptimeをcapture前後で変えず、反復composition/image captureで安定した公開KUC raster frameを提示する。diagnostic overlayまたはcontrol label SHALL、KUC frameとして数えない。

#### Scenario: fresh capture sampleが安定する

- **WHEN** fresh Native text-input callbackがrecordIndex/uptimeを示す
- **THEN** hostは同じ入力tokenのままcanonical RGBA sampleを2点取得し、rootが元PNGをレビューする。自動的なvisual semantic PASSは行わない

#### Scenario: control labelは見えるがKUC contentがない

- **WHEN** control labelは見えるがKUC raster sampleが欠落、blank、または不安定である
- **THEN** Native受入はpendingのままとし、renderer/backend failureとして記録する

### Requirement: renderer選択は実装詳細である

共通KUC frameと3つのvendor adapter SHALL renderer-independentのままとする。supported native rendererはhost configurationで選択 MAY だが、共通behavior仕様 SHALL Vello固有の描画動作を要求しない。

#### Scenario: 明示的fallbackを実行する

- **WHEN** hostが明示的に要求されたsupported TinySkia fallbackを使用する
- **THEN** 観測できたrenderer identityを記録し、受入基準を下げず同じ公開KUC raster contractを評価する
