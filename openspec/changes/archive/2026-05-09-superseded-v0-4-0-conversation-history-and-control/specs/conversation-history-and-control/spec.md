## ADDED Requirements

### Requirement: session 履歴を保存して復元できなければならない

システムは、session ごとの会話本文、状態表示履歴、添付 metadata、provider / runtime 選択、output 参照、draft、scroll 復元情報を保存し、履歴一覧を表示し、UI から復元できなければならない（MUST）。

#### Scenario: session 履歴一覧を表示する

- **GIVEN** 保存済み session が存在する
- **WHEN** user が履歴一覧を開く
- **THEN** UI は session title、provider、最終更新時刻、preview、status を表示する
- **THEN** UI は保存形式の file path / DB schema を知らない

#### Scenario: session を復元する

- **GIVEN** 保存済み session が存在する
- **WHEN** user が session 一覧から対象 session を選ぶ
- **THEN** message、thinking log reference、output reference、attachments metadata が復元される
- **THEN** draft と scroll position が復元される
- **THEN** provider へ再送信しない

#### Scenario: session 保存形式を UI から隠す

- **WHEN** UI が session を表示する
- **THEN** UI は `SessionHistoryStore` interface だけを使う
- **THEN** file path や DB schema に依存しない

### Requirement: chat 検索を提供しなければならない

システムは、現在 chat 内検索と保存済み履歴の横断検索を提供しなければならない（MUST）。

#### Scenario: 現在 chat 内を検索する

- **GIVEN** current session に複数 message がある
- **WHEN** user が current chat search を実行する
- **THEN** UI は message id、matched range、preview を持つ検索結果を表示する
- **THEN** user は検索結果から該当 message へ移動できる

#### Scenario: 履歴を横断検索する

- **GIVEN** 保存済み session が複数存在する
- **WHEN** user が history search を実行する
- **THEN** UI は session id、message id、matched range、preview、score を持つ検索結果を表示する
- **THEN** user は検索結果から対象 session を復元し、該当 message へ移動できる

### Requirement: 送信済み message を編集して再開できなければならない

システムは、送信済み user message を編集でき、編集確定時にその message 以降の response、thinking、output を無効化して会話を再開できなければならない（MUST）。

#### Scenario: 送信済み message を編集確定する

- **GIVEN** user message の後に agent response と output が存在する
- **WHEN** user がその user message を編集して確定する
- **THEN** 編集対象以降の response、thinking、output は invalidated になる
- **THEN** provider が rewind を support する場合だけ rewind intent を送る

#### Scenario: rewind 非対応 provider で編集する

- **GIVEN** provider が rewind を support しない
- **WHEN** user が送信済み message の編集確定を試みる
- **THEN** UI は disabled reason を表示する
- **THEN** fallback 送信を行わない

### Requirement: 処理中の追加投稿を conversation intent として扱わなければならない

システムは、処理中の追加投稿を通常送信ではなく、`SteerMessage`、`InterruptAndSend`、`QueueMessage` のいずれかの intent として扱わなければならない（MUST）。

#### Scenario: interrupt して送信する

- **GIVEN** agent response が running である
- **WHEN** user が interrupt 送信を選ぶ
- **THEN** current generation は停止される
- **THEN** 新しい generation id で追加 message が送信される

#### Scenario: queue に積む

- **GIVEN** agent response が running である
- **WHEN** user が queue 送信を選ぶ
- **THEN** 追加 message は current generation 完了まで送信されない
- **THEN** message order は送信順で保持される

### Requirement: 古い generation の event を混入させてはならない

システムは、current generation id と一致しない chunk、thinking、output、complete、failed event を current thread に反映してはならない（MUST）。

#### Scenario: 古い chunk が遅れて届く

- **GIVEN** generation A を停止し generation B が開始している
- **WHEN** generation A の chunk が遅れて届く
- **THEN** UI は generation A の chunk を current response に追記しない
- **THEN** output list に generation A の output を混ぜない
