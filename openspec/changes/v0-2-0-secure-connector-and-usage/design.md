## Context

ACP 公式仕様では、接続開始時に `initialize` で protocol version、capabilities、authentication methods を交渉する。transport は stdio JSON-RPC が安定済みで、Streamable HTTP は議論中である。

ACP の session config options は model、mode、thought level などの session-level 設定を表示する推奨手段になっている。usage / context status は RFD として提案されており、kcu は同じ形の optional provider capability として扱う。

## Goals

- ACP 対応 agent の接続設定を最小化する。
- ACP 非対応 provider でも secret を平文保存しない。
- secret は通信時に毎回取得・復号し、利用後に破棄する。
- model、thinking、permission、usage を provider capability として扱う。
- account / usage 画面を provider ごとに表示可能にする。

## Non-Goals

- Claude Code / Codex / GitHub Copilot など個別 adapter の実装 — v0.3.0。
- Streamable HTTP / WebSocket transport — ACP 側の安定後に別 change。
- billing provider の金額計算を kcu 側で推定すること。

## ACP Connection Flow

```
AgentCommandConfig
  └─ spawn stdio process
      └─ initialize
          ├─ protocolVersion
          ├─ clientCapabilities
          ├─ agentCapabilities
          └─ authMethods
              └─ session/new
                  ├─ cwd
                  ├─ mcpServers
                  └─ configOptions
```

kcu は ACP agent を「provider 実装」としてではなく「agent process」として扱う。provider-specific な API key は、agent 側の login flow または agent-specific settings へ委譲する。

## Direct Connector Secret Model

```
SecretStore
  ├─ put_secret(provider_id, secret_ref, plaintext)
  ├─ acquire_secret(provider_id, secret_ref) -> SecretLease
  └─ delete_secret(provider_id, secret_ref)

SecretLease
  ├─ expose_for_request()
  └─ drop 時に memory zeroize
```

保存戦略は次の順に採用する。

1. OS credential store。macOS Keychain、Windows Credential Manager、Linux Secret Service など。
2. host-provided `SecretStore`。host が保存場所と暗号方式を管理する。
3. encrypted file store。host が渡した保存領域に ciphertext だけを保存し、復号 key は OS credential store または host-provided key store に置く。

平文 file、平文 JSON、環境変数への永続保存は採用しない。環境変数は開発時の one-shot injection として許可できるが、kcu が保存してはならない。

## Provider Settings

provider settings は secret を含まない。

- provider id
- display name
- connection kind: `acp` / `direct`
- selected model
- thinking option
- permission option
- endpoint URL
- secret reference id

model、thinking、permission は ACP agent では session config options を優先する。direct connector では同じ意味の `ProviderConfigOption` に写像する。

## Account & Usage

`AccountUsageSnapshot` は次の表示要素を持つ。

- auth method
- account label
- organization label
- plan label
- quota rows: label、used percentage、reset label、external management URL
- unavailable reason

provider は usage を返せる場合だけ値を返す。kcu は provider が返した値を表示し、推定で plan や quota を作らない。

## Security Rules

- secret value は log、error message、metadata、debug dump に出さない。
- secret を clone 可能な `String` として長期保持しない。
- LLM request ごとに `SecretLease` を取得する。
- request 完了後に lease を drop する。
- secret 未設定時は provider call を行わず、UI に setup required を返す。

## Verification

- secret を含む settings JSON を拒否する test がある。
- direct connector が request ごとに `SecretStore::acquire_secret` を呼ぶ test がある。
- `SecretLease` drop の zeroize test がある。
- ACP initialize / session/new / configOptions の mock JSON-RPC test がある。
- account usage available / unavailable の render model test がある。
