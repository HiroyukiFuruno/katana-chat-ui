## ADDED Requirements

### Requirement: OS file picker と drag and drop で添付できなければならない

システムは、OS file picker と drag and drop から添付を追加でき、OS file、host logical resource、virtual attachment を区別して host callback へ解決を委譲しなければならない（MUST）。

#### Scenario: OS file picker から添付する

- **WHEN** user が composer の添付操作から OS file picker を開き、file を選ぶ
- **THEN** UI は file attachment chip を composer 内に表示する
- **THEN** draft、cursor、IME state、既存 attachment は保持される

#### Scenario: drag and drop で添付する

- **WHEN** user が OS file、host logical resource、virtual attachment のいずれかを composer へ drop する
- **THEN** kcu core は attachment source を区別する
- **THEN** host callback に解決を依頼する
- **THEN** 解決失敗時は fallback せず、添付失敗として表示する

#### Scenario: 添付を取り消す

- **WHEN** user が attachment chip の取消操作を行う
- **THEN** 対象 attachment だけが composer から外れる
- **THEN** draft、cursor、IME state、他の attachment は保持される

### Requirement: クリップボード画像を添付できなければならない

システムは、クリップボードから画像を貼り付け、通常の添付と同じ interface で扱えなければならない（MUST）。

#### Scenario: 画像を貼り付ける

- **WHEN** user が clipboard image を composer に貼り付ける
- **THEN** UI は image attachment chip を composer 内に表示する
- **THEN** draft、cursor、IME state、既存 attachment は保持される

#### Scenario: 画像 decode を委譲する

- **WHEN** clipboard image attachment を解決する
- **THEN** kcu core は host callback に解決を依頼する
- **THEN** OS file path と virtual attachment を区別する

### Requirement: 絵文字入力を壊してはならない

システムは、macOS の絵文字パレットから絵文字を挿入でき、cursor 位置と placeholder 表示を壊してはならない（MUST）。

#### Scenario: 絵文字を挿入する

- **WHEN** user が macOS 絵文字パレットから絵文字を選ぶ
- **THEN** 絵文字は現在の cursor 位置へ挿入される
- **THEN** placeholder は入力値として残らない

### Requirement: message timing を表示できなければならない

システムは、user message の送信時刻、agent response の開始 / 完了 / 失敗 / 停止時刻、応答中の elapsed duration を扱えなければならない（MUST）。

#### Scenario: 応答中の時間を表示する

- **WHEN** agent response が running である
- **THEN** UI は elapsed duration を更新する
- **THEN** 完了、失敗、停止のいずれかで elapsed duration を固定する

#### Scenario: session 復元で時刻を保持する

- **WHEN** session を復元する
- **THEN** 保存済み message timing が復元される
- **THEN** 完了済み response の elapsed duration は再計算されない

### Requirement: provider selector に icon を表示しなければならない

システムは、provider selector の候補行に provider icon を表示しなければならない（MUST）。

#### Scenario: provider icon を表示する

- **WHEN** provider selector を開く
- **THEN** 各候補行は `ProviderIconRegistry` から取得した icon を表示する
- **THEN** 会話 title icon と同じ provider icon source を使う

#### Scenario: icon がない provider を扱う

- **WHEN** provider に公式または明示登録された icon がない
- **THEN** registry は `IconMissing(provider_id)` を返す
- **THEN** `+` や汎用コード記号を代替 icon として表示しない

### Requirement: Markdown renderer を差し替え可能にしなければならない

システムは、`comrak` adapter と `katana-document-viewer` adapter を `MarkdownRenderer` interface 経由で差し替え可能にしなければならない（MUST）。

#### Scenario: renderer を差し替える

- **WHEN** `katana-document-viewer` adapter を有効にする
- **THEN** chat message、output、provider、composer の public contract は変わらない
- **THEN** v0.1.0 の Markdown subset と code fence contract は維持される
