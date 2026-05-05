## Context

ACP 対応 agent と direct LLM provider は同じものではない。Claude Code や Codex は CLI / agent としての設定、認証、permission flow を持つ。一方、Anthropic API や Vertex AI は direct API provider であり、secret store と cloud credential が必要になる。

Zed の外部 agent 連携では Claude Agent と Codex が ACP adapter 経由で動作し、認証は agent 側の login flow と分離されている。GitHub Copilot は GitHub Copilot Extensions や client-specific surface が中心で、汎用 direct provider として扱わない。

## Goals

- vendor ごとの接続方式を分類し、MVP と future support を明確にする。
- Ollama MVP を最初に完成させる。
- Claude Code / Codex / GitHub Copilot は ACP adapter or extension availability を前提に扱う。
- direct cloud provider は v0.2.0 の secret store contract に従う。
- provider ごとの model、thinking、permission、usage、attachment support を capability model に載せる。

## Non-Goals

- Claude Code subscription、ChatGPT subscription、GitHub Copilot subscription の認証情報を kcu が直接再利用すること。
- GitHub Copilot の private / editor-specific API を kcu core から直接呼ぶこと。
- vendor ごとの UI を hard-code すること。
- Bedrock / Vertex AI の cloud credential を settings JSON に保存すること。

## Vendor Support Matrix

| Target | Support Mode | v0.3.0 Status | Contract |
|---|---|---|---|
| Ollama | local-direct | MVP | endpoint、model list、availability |
| OpenAI-compatible endpoint | openai-compatible-direct | planned | endpoint、model、secret ref、SSE streaming |
| Anthropic API | cloud-direct | planned | secret ref、model、thinking support、usage if available |
| Claude Code | acp-agent | planned | ACP adapter。direct Anthropic API とは別扱い |
| Codex CLI | acp-agent | planned | ACP adapter。OpenAI-compatible direct とは別扱い |
| GitHub Copilot | acp-agent or host extension | blocked until adapter/surface exists | kcu core から direct provider にしない |
| Vertex AI | cloud-direct | planned | OAuth / ADC / secret store |
| Bedrock | cloud-direct | planned | AWS credential provider / secret store |
| Ollama via cloud router | openai-compatible-direct | conditional | router が compatible endpoint を出す場合のみ |

## Provider Capability Model

```
ProviderDescriptor
  id
  display_name
  support_mode
  connection_kind
  setup_state
  config_options
  prompt_capabilities
  usage_capability
  account_capability
```

`config_options` は model、thinking、permission を含む。ACP agent では session config options から生成し、direct connector では provider schema から生成する。

## Streaming

streaming は provider event として扱う。

- ACP agent: `session/update` を provider event に写像する。
- direct provider: SSE、chunked response、non-streaming response を provider event に写像する。
- UI は provider event の source を知らない。

## Unsupported States

対応不能な provider は実装しないままにせず、明示的な state を返す。

- `Unsupported(reason: NoPublicApi)`
- `Unsupported(reason: RequiresHostExtension)`
- `Unsupported(reason: AdapterMissing)`
- `Unsupported(reason: AuthUnavailable)`

## Verification

- support matrix が `ProviderDescriptor` test で固定されている。
- Ollama MVP が direct connector として動く。
- Claude Code / Codex / GitHub Copilot は direct provider として登録されない。
- unsupported reason が UI model に表示できる。
- direct cloud provider は `SecretRef` だけを持ち、secret value を settings に持たない。
