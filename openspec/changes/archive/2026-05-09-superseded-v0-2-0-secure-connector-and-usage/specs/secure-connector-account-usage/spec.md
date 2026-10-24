## ADDED Requirements

### Requirement: ACP agent 接続を簡略化しなければならない

システムは、ACP 対応 agent に対して stdio JSON-RPC transport、initialize、session/new、session config options を扱う接続 contract を提供しなければならない（MUST）。

#### Scenario: ACP agent を起動する

- **WHEN** host が ACP agent command config を渡す
- **THEN** kcu は agent process を起動し、stdio JSON-RPC で接続する
- **THEN** `initialize` で protocol version と capabilities を交渉する

#### Scenario: provider 設定を表示する

- **WHEN** ACP agent が session config options を返す
- **THEN** kcu は model、thinking、permission に対応する options を UI model に写像する
- **THEN** unknown option は無視せず、advanced options として保持する

### Requirement: direct connector は secret を安全に扱わなければならない

システムは、ACP が使えない provider の secret を settings JSON に平文保存せず、`SecretStore` 経由で通信ごとに取得・復号・破棄しなければならない（MUST）。

#### Scenario: API key を登録する

- **WHEN** ユーザーが direct connector の API key を登録する
- **THEN** kcu は plaintext を settings JSON に保存しない
- **THEN** settings JSON には `SecretRef` だけを保存する

#### Scenario: LLM request を送信する

- **WHEN** direct connector が LLM request を送信する
- **THEN** connector は request ごとに `SecretStore::acquire_secret` を呼ぶ
- **THEN** request 完了後に `SecretLease` を破棄する
- **THEN** secret value を log、error、metadata に含めない

#### Scenario: secure store が利用できない

- **WHEN** OS credential store も host-provided store も利用できない
- **THEN** kcu は平文 fallback を使わない
- **THEN** UI は setup blocked state と理由を表示する

### Requirement: account と usage を provider capability として表示しなければならない

システムは、provider が provider usage / account usage 情報を返せる場合に token usage、context usage、account label、organization label、plan label、quota rows、reset label、external management URL を表示しなければならない（MUST）。

#### Scenario: usage が取得できる

- **WHEN** provider が account usage を返す
- **THEN** UI は quota row の label、percentage、reset label を表示できる
- **THEN** external management URL がある場合は management action を表示できる

#### Scenario: token usage が取得できる

- **WHEN** provider が request / response token usage を返す
- **THEN** UI は provider usage として token usage を表示できる
- **THEN** context usage と account usage を混同しない

#### Scenario: usage が取得できない

- **WHEN** provider が account usage を提供しない
- **THEN** UI は `Unavailable(reason)` を表示状態として持つ
- **THEN** plan や quota を推定で作らない

### Requirement: settings JSON へ typed settings を merge しなければならない

システムは、host が指定した non-null settings JSON reference へ typed settings patch を merge しなければならない（MUST）。

#### Scenario: settings 保存先が指定される

- **WHEN** host が settings JSON reference を渡す
- **THEN** kcu は provider connection、runtime、MCP server、usage 表示設定を typed settings として merge できる
- **THEN** 既存の unrelated settings を破壊しない

#### Scenario: settings 保存先が指定されない

- **WHEN** host が settings JSON reference を渡さない
- **THEN** kcu は invalid configuration を返す
- **THEN** 省略時の in-memory fallback を使わない

#### Scenario: secret が混入する

- **WHEN** typed settings patch に secret value が含まれる
- **THEN** kcu は invalid settings として拒否する
- **THEN** settings JSON へ secret value を保存しない

### Requirement: MCP server 設定を扱えなければならない

システムは、ACP agent の session setup で使う MCP server 設定を settings model として扱えなければならない（MUST）。

#### Scenario: MCP server を設定する

- **WHEN** user が MCP server 設定を追加する
- **THEN** kcu は server id、command または transport、env reference、enabled state を settings model に保持する
- **THEN** secret value は settings JSON に含めない

#### Scenario: MCP server が無効である

- **WHEN** MCP server 設定が validation に失敗する
- **THEN** UI は disabled reason を表示できる
- **THEN** provider call を fallback 実行しない
