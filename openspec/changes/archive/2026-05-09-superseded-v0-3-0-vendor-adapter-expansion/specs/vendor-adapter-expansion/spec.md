## ADDED Requirements

### Requirement: provider の接続種別を明示しなければならない

システムは、provider を `openai-compatible-direct`、`cloud-direct`、`acp-agent`、`unsupported-direct` のいずれかに分類し、Ollama などの local model backend は `local-runtime` として provider から分離しなければならない（MUST）。

#### Scenario: provider descriptor を表示する

- **WHEN** UI が provider list を表示する
- **THEN** 各 provider は support mode、connection kind、setup state を持つ
- **THEN** unsupported provider は理由を表示できる

#### Scenario: direct API がない provider を扱う

- **WHEN** provider に kcu から使える汎用 direct API がない
- **THEN** システムは `unsupported-direct` として登録する
- **THEN** 非公開 API や host-specific API を kcu core から呼ばない

### Requirement: Ollama を local runtime として扱わなければならない

システムは、Ollama を agent provider ではなく local runtime として扱い、endpoint、availability、model list、selected model を提供しなければならない（MUST）。

#### Scenario: Ollama が利用可能である

- **WHEN** Ollama endpoint が `/api/tags` に成功応答する
- **THEN** runtime は available state と model list を返す
- **THEN** provider selector には Ollama を agent provider として表示しない

#### Scenario: Ollama usage が取得できない

- **WHEN** Ollama runtime が account usage を返せない
- **THEN** UI は `Unavailable(reason)` として表示する

#### Scenario: Ollama が agent capability を持たない

- **WHEN** UI が permission、file editing、command execution control を組み立てる
- **THEN** Ollama runtime はそれらの control を有効化しない
- **THEN** agent provider の capability と混ぜない

### Requirement: Claude Code と Codex は ACP agent adapter として扱わなければならない

システムは、Claude Code と Codex を direct LLM provider ではなく ACP agent adapter として扱わなければならない（MUST）。

#### Scenario: Claude Code を登録する

- **WHEN** Claude Code support を有効にする
- **THEN** provider registry は ACP agent adapter entry を作る
- **THEN** Anthropic API direct connector とは別 provider として扱う

#### Scenario: Codex を登録する

- **WHEN** Codex support を有効にする
- **THEN** provider registry は ACP agent adapter entry を作る
- **THEN** OpenAI-compatible direct connector とは別 provider として扱う

### Requirement: GitHub Copilot は adapter availability なしに direct provider 化してはならない

システムは、GitHub Copilot を汎用 direct LLM provider として扱ってはならない（MUST）。ACP-compatible adapter または host extension surface が確認できる場合だけ support state を available にできる。

#### Scenario: Copilot adapter が存在しない

- **WHEN** GitHub Copilot の ACP adapter または host extension surface が設定されていない
- **THEN** provider registry は `Unsupported(AdapterMissing)` を返す
- **THEN** direct API key 入力欄を表示しない

### Requirement: cloud direct provider は secret store contract に従わなければならない

システムは、Anthropic API、Vertex AI、Bedrock、OpenAI-compatible cloud endpoint を cloud-direct provider として扱う場合、secret value を settings に保存せず `SecretRef` を使わなければならない（MUST）。

#### Scenario: cloud direct provider を設定する

- **WHEN** user が cloud direct provider を設定する
- **THEN** settings には endpoint、model、secret reference だけを保存する
- **THEN** secret value は `SecretStore` に保存される

#### Scenario: Ollama 経由 cloud router を使う

- **WHEN** Ollama または中間 router が OpenAI-compatible endpoint を露出する
- **THEN** システムは `openai-compatible-direct` として扱う
- **THEN** provider が compatible endpoint を露出しない場合は cloud direct adapter を使う

### Requirement: agent capability catalog を提供しなければならない

システムは、provider / adapter / host が利用可能と返した prompt、skill、workflow、command、hook、MCP を kind 付きの agent capability catalog として提供しなければならない（MUST）。

#### Scenario: catalog を生成する

- **WHEN** ACP agent adapter が session config options と available commands を返す
- **THEN** kcu は prompt、skill、workflow、command、hook、MCP を kind 付き entry として保持する
- **THEN** entry は id、display label、source、enabled state、disabled reason を持つ

#### Scenario: 未対応 entry を扱う

- **WHEN** adapter が skill または workflow を利用不可と返す
- **THEN** kcu は実行可能な候補として表示しない
- **THEN** 表示する場合は disabled reason を必ず持つ

#### Scenario: slash launcher へ候補を供給する

- **WHEN** composer の slash launcher が候補を要求する
- **THEN** kcu は agent capability catalog から launchable な prompt、skill、workflow、command だけを返す
- **THEN** hook と MCP は adapter が launchable と返した場合だけ候補へ含める
- **THEN** 選択結果は `CommandLaunchIntent` として provider / adapter / host へ渡される
