## Why

履歴一覧を開くと、KCU は一覧をメッセージのスクロール領域の末尾へ追加している。長い会話ではその領域のスクロール範囲に履歴が含まれず、一覧を表示・選択できないため、既存の Storybook 履歴操作を常に確認できるようにする。

## What Changes

- 履歴一覧をヘッダー直下のスレッド表示領域に固定し、会話本文のスクロール位置から独立させる。
- 履歴一覧自身に独立したスクロール状態を持たせ、表示範囲がスレッド領域を超えず、すべてのセッションを選択できるようにする。
- 履歴を閉じたときとセッション一覧が変わったときに履歴スクロール位置をリセットし、履歴上のホイール操作で会話位置を変更しない。

## Capabilities

### New Capabilities

なし。

### Modified Capabilities

- `pure-kuc-chat`: Storybook の履歴操作を長い会話でも表示・選択可能にする。

## Impact

- KCU の KUC tree frame、runtime scroll state / input routing、renderer-backed KUC runtime tests、OpenSpec delta。
- 公開 API、KUC API、framework adapter、依存関係は変更しない。
