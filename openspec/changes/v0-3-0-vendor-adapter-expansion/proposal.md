## Why

multi vendor 対応は、単に API endpoint を増やすだけでは成立しない。Claude Code、Codex、GitHub Copilot は「直接 LLM API」と「agent / product integration」の境界が異なるため、kcu が同じ扱いで実装すると認証、利用状況、permission、thinking 設定が破綻する。

v0.3.0 では、Ollama を MVP 基準にしつつ、ACP agent adapter と direct provider adapter を明確に分け、各 vendor の対応モードと未対応条件を固定する。

## What Changes

### Vendor Classification

- `local-direct`: Ollama など、local endpoint に直接接続する provider。
- `openai-compatible-direct`: LM Studio、llama.cpp server、Ollama `/v1` など OpenAI-compatible endpoint。
- `cloud-direct`: Anthropic API、Vertex AI、Bedrock など、secret store と cloud auth が必要な provider。
- `acp-agent`: Claude Code、Codex CLI、GitHub Copilot など、ACP adapter 経由で agent として接続する対象。
- `unsupported-direct`: provider 側に汎用 direct API がない、または規約上 kcu から直接扱えない対象。

### MVP

- MVP は Ollama direct connector を基準にする。
- Ollama は local model list、selected model、endpoint、context usage unknown を扱う。
- Ollama 経由で Bedrock / Vertex AI を使うモードは、Ollama または中間 router が OpenAI-compatible endpoint として露出する場合だけ `openai-compatible-direct` として扱う。

### Future Adapters

- Claude Code は ACP adapter 経由を primary とする。Anthropic API direct connector は Claude Code ではなく Anthropic provider として扱う。
- Codex は ACP adapter / Codex CLI 経由を primary とする。OpenAI API direct connector は Codex product ではなく OpenAI-compatible provider として扱う。
- GitHub Copilot は、汎用 direct LLM provider として扱わない。ACP-compatible adapter または GitHub Copilot extension surface が提供される場合だけ対応する。
- Vertex AI と Bedrock は cloud-direct provider として扱い、secret store / OAuth / cloud credentials は v0.2.0 の contract に従う。

## Capabilities

### New Capabilities

- `vendor-classification`: vendor の接続種別と support state を明示
- `ollama-mvp`: local Ollama の model / endpoint / availability
- `acp-agent-adapters`: Claude Code / Codex / Copilot などの ACP agent 接続
- `cloud-direct-adapters`: Anthropic / Vertex AI / Bedrock などの direct cloud 接続
- `provider-capability-model`: model / thinking / permission / usage / attachment support を統一表現

## Impact

- `crates/katana-chat-connectors/` — adapter registry と provider classification
- `crates/katana-acp-client/` — ACP agent adapter integration
- `docs/settings-schema.json` — secret を含まない provider schema
- `openspec/project.md` — roadmap と non-goals の更新
