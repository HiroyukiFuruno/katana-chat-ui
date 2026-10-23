## ADDED Requirements

### Requirement: 履歴一覧を会話から独立表示する

KCU SHALL 共通KUC runtimeの履歴一覧をヘッダー直下のスレッド viewport に固定して表示し、長い会話のスクロール位置から独立させる。セッション一覧が viewport より長い場合も、履歴専用スクロールで全行を表示・選択可能にする。履歴専用 wheel 操作は会話のスクロール位置を変更しない。provider-free Storybookもこの共通runtimeの表示・操作経路を使用する。

#### Scenario: 長い会話でも履歴を開ける

- **WHEN** 開発者が長い会話の途中で Storybook の履歴操作を行う
- **THEN** 履歴一覧はヘッダーと composer の間に表示され、会話のスクロール位置を変えず、見える履歴行に session 選択 action を提供する

#### Scenario: 長い履歴一覧を専用スクロールで選択する

- **WHEN** 履歴一覧の行がスレッド viewport より多く、開発者が履歴一覧上で wheel 操作する
- **THEN** 履歴一覧だけが viewport 内でスクロールし、会話のスクロール位置と composer / header の操作領域を維持したまま最後の行を表示・選択できる

#### Scenario: 履歴 overlay が背後の会話を隠す

- **WHEN** 履歴一覧を開いた状態で会話本文が異なる二つの surface を同じ viewport に描画する
- **THEN** 履歴 viewport の pixel は会話本文に依存せず、背後の conversation message action は pointer hit tree に残らない

#### Scenario: 履歴行の可視情報を保持する

- **WHEN** session が一件以上存在する履歴一覧を表示する
- **THEN** 各行は title、provider/updated、preview を折り返し可能な公開 KUC text として表示し、`No history` は表示しない
