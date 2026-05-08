## Why

接続と秘匿情報（secret）の設計を曖昧にしたまま provider を増やすと、settings に API key を平文で置く実装や、provider ごとの UI 分岐が先に固定される。

v0.2.0 では、ACP による接続簡略化、ACP が使えない provider への安全な直接接続、provider usage / account usage 表示、provider settings、settings JSON merge、MCP server 設定の contract を先に確立する。

## What Changes

### ACP connector

- ACP agent は stdio JSON-RPC を必須 transport として接続する。
- `initialize` で protocol version、client capabilities、agent capabilities、auth methods を交渉する。
- `session/new` で working directory、MCP server、session config options を受け取る。
- MCP server 設定は provider / agent 設定の一部として扱い、host が指定した settings JSON へ merge できる。
- model、thinking level、permission mode は ACP の session config options を優先して UI に出す。

### Direct connector

- ACP が使えない provider は direct connector として扱う。
- secret は settings JSON に保存しない。
- secret は `SecretStore` から通信ごとに取得し、復号し、`SecretLease` として短時間だけ扱う。
- OS credential store を優先実装とし、host が保存領域を渡す場合は ciphertext だけを保存する。
- OS credential store が使えない環境では、host-provided `SecretStore` を必須にし、平文 fallback は提供しない。

### Settings, Account & Usage

- 添付画像のような account / usage surface を provider capability として定義する。
- provider ごとの token usage、context usage、request usage を provider usage として定義する。
- account label、organization label、plan label、quota rows、reset time、external management URL を表示できる。
- provider が usage を返せない場合は `Unavailable(reason)` を返す。
- settings 保存先は host が non-null な JSON reference として渡す。
- kcu は provider connection、runtime、MCP server、usage 表示設定を typed settings として merge する。

## Capabilities

### New Capabilities

- `acp-connection`: ACP agent 接続、初期化、session setup
- `direct-secure-connector`: ACP 非対応 provider の安全な接続
- `secret-store`: secret 保存、復号、破棄の境界
- `provider-settings`: model / thinking / permission を provider capability として表示
- `settings-json-merge`: host 指定 JSON への typed settings merge
- `mcp-server-settings`: MCP server 設定の保持
- `provider-usage`: provider token / request / context usage 表示
- `account-usage`: account / quota / plan 表示

## Impact

- `crates/katana-acp-client/` — ACP JSON-RPC connection contract
- `crates/katana-chat-connectors/` — direct connector と secret boundary
- `crates/katana-chat-ui/` — provider settings と usage render model
- `docs/settings-schema.json` — secret を含まない provider settings schema
