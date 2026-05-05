## ADDED Requirements

### Requirement: provider の接続種別を明示しなければならない

システムは、provider を `local-direct`、`openai-compatible-direct`、`cloud-direct`、`acp-agent`、`unsupported-direct` のいずれかに分類しなければならない（MUST）。

#### Scenario: provider descriptor を表示する

- **WHEN** UI が provider list を表示する
- **THEN** 各 provider は support mode、connection kind、setup state を持つ
- **THEN** unsupported provider は理由を表示できる

#### Scenario: direct API がない provider を扱う

- **WHEN** provider に kcu から使える汎用 direct API がない
- **THEN** システムは `unsupported-direct` として登録する
- **THEN** 非公開 API や host-specific API を kcu core から呼ばない

### Requirement: Ollama を MVP provider として扱わなければならない

システムは、Ollama を local-direct MVP provider として扱い、endpoint、availability、model list、selected model を提供しなければならない（MUST）。

#### Scenario: Ollama が利用可能である

- **WHEN** Ollama endpoint が `/api/tags` に成功応答する
- **THEN** provider は available state と model list を返す

#### Scenario: Ollama usage が取得できない

- **WHEN** Ollama provider が account usage を返せない
- **THEN** UI は `Unavailable(reason)` として表示する

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
