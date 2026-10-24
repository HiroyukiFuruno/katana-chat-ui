## ADDED Requirements

### Requirement: output extension view を提供しなければならない

システムは、file、diff、tool result、permission request を chat 本文とは別の output extension view として表示できなければならない（MUST）。

#### Scenario: output extension view を描画する

- **WHEN** provider event が output を返す
- **THEN** UI は output を message body ではなく output extension view へ写像する
- **THEN** output view は source message id と generation id を持つ

#### Scenario: debug JSON と混ぜない

- **WHEN** `options: { debug: true }` が有効である
- **THEN** debug JSON は右端 hover surface にだけ表示される
- **THEN** output extension view は debug JSON と同じ表示領域に混入しない

### Requirement: file / diff candidate を表示できなければならない

システムは、生成または変更候補の file と diff を、host / provider へ渡す action intent 付きで表示できなければならない（MUST）。

#### Scenario: file candidate を表示する

- **WHEN** output が file candidate を含む
- **THEN** UI は path、operation、summary、open intent を表示する
- **THEN** kcu core は file を直接書き込まない

#### Scenario: diff candidate を表示する

- **WHEN** output が diff candidate を含む
- **THEN** UI は target path、unified diff、status、open / apply / reject intent を表示する
- **THEN** diff 適用は host / provider callback に委譲される

### Requirement: tool result と permission request を表示できなければならない

システムは、tool result と permission request を output extension view として表示し、承認や拒否を action intent として返せなければならない（MUST）。

#### Scenario: tool result を表示する

- **WHEN** command execution が完了する
- **THEN** UI は title、status、summary、stdout / stderr reference、copy intent を表示する

#### Scenario: permission request を表示する

- **WHEN** provider が permission request を返す
- **THEN** UI は request reason、対象 tool、approve / reject intent を表示する
- **THEN** kcu core は permission 承認処理を直接実行しない

### Requirement: 状態表示と output view を関連付けなければならない

システムは、execution、read、editing、fetch の状態表示から、関連する output extension view を開けるようにしなければならない（MUST）。

#### Scenario: execution 状態から tool result を開く

- **GIVEN** command execution の状態表示が存在する
- **WHEN** command execution が完了する
- **THEN** 状態表示は関連する tool result view の output id を参照する
- **THEN** user は状態表示から tool result view を開ける
