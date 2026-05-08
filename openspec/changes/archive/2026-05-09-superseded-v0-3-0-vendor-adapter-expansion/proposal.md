## Why

multi vendor 対応は、単に API endpoint を増やすだけでは成立しない。Claude Code、Codex、GitHub Copilot は「直接 LLM API」と「agent / product integration」の境界が異なるため、kcu が同じ扱いで実装すると認証、利用状況、permission、thinking 設定が破綻する。

v0.3.0 では、agent provider と local runtime を明確に分ける。Ollama は agent provider ではなく local runtime backend として扱い、Claude Code、Codex CLI、GitHub Copilot、OpenCode などの agent provider adapter とは同列にしない。

## What Changes

### Vendor Classification

- `local-runtime`: Ollama など、agent provider が利用できる local model backend。
- `openai-compatible-direct`: LM Studio、llama.cpp server、Ollama `/v1` など OpenAI-compatible endpoint。
- `cloud-direct`: Anthropic API、Vertex AI、Bedrock など、secret store と cloud auth が必要な provider。
- `acp-agent`: Claude Code、Codex CLI、GitHub Copilot など、ACP adapter 経由で agent として接続する対象。
- `unsupported-direct`: provider 側に汎用 direct API がない、または規約上 kcu から直接扱えない対象。

### Local Runtime

- Ollama は agent provider selector に出さない。
- Ollama は local runtime として local model list、selected model、endpoint、context usage unknown を扱う。
- Ollama は permission mode、file editing、command execution を持つ agent provider として表示しない。
- Ollama 経由で Bedrock / Vertex AI を使うモードは、Ollama または中間 router が OpenAI-compatible endpoint として露出する場合だけ `openai-compatible-direct` として扱う。

### Future Adapters

- Claude Code は ACP adapter 経由を primary とする。Anthropic API direct connector は Claude Code ではなく Anthropic provider として扱う。
- Codex は ACP adapter / Codex CLI 経由を primary とする。OpenAI API direct connector は Codex product ではなく OpenAI-compatible provider として扱う。
- GitHub Copilot は、汎用 direct LLM provider として扱わない。ACP-compatible adapter または GitHub Copilot extension surface が提供される場合だけ対応する。
- Vertex AI と Bedrock は cloud-direct provider として扱い、secret store / OAuth / cloud credentials は v0.2.0 の contract に従う。

### Agent Capability Catalog

- prompt、skill、workflow、command、hook、MCP は agent capability として扱う。
- adapter は利用可能な entry だけを kcu へ返す。
- kcu は entry を settings screen と `/` launcher の両方へ供給する。
- 未対応 entry は実行可能に見せず、disabled reason を返す。

## Capabilities

### New Capabilities

- `provider-classification`: provider の接続種別と support state を明示
- `local-runtime-classification`: Ollama などの local runtime と agent provider を分離
- `ollama-local-runtime`: local Ollama の model / endpoint / availability
- `acp-agent-adapters`: Claude Code / Codex / Copilot などの ACP agent 接続
- `cloud-direct-adapters`: Anthropic / Vertex AI / Bedrock などの direct cloud 接続
- `provider-capability-model`: model / thinking / permission / usage / attachment support を統一表現
- `agent-capability-catalog`: prompt / skill / workflow / command / hook / MCP の利用可能 entry
- `slash-launcher-provider`: agent capability catalog から `/` 候補を生成

## Impact

- `crates/katana-chat-connectors/` — adapter registry と provider classification
- `crates/katana-acp-client/` — ACP agent adapter integration
- `docs/settings-schema.json` — secret を含まない provider schema
- `openspec/project.md` — roadmap と non-goals の更新
