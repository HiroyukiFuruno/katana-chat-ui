# Tasks: katana-chat-ui v0.1.0

## Branch Rule

- **標準ブランチ**: `release/v0.1.0`
- **作業ブランチ**: `feature/v0.1.0-task-x`

---

## 1. KatanA からの実装移管

### 実施内容

KatanA v0.23.0 branch で直接実装した AI / chat / autofix コードを katana-chat-ui へ移管し、KatanA 側から参照できる状態にする。

### 完了条件

- [ ] 1.1 `crates/katana-acp-client/src/` に KatanA `katana-core/src/ai/` の Ollama 実装を移管する
- [ ] 1.2 `crates/katana-chat-ui/src/panel.rs` に KatanA `katana-ui/src/app/chat.rs` の widget を移管する
- [ ] 1.3 `crates/katana-chat-ui/src/diff.rs` に autofix diff preview / confirm / apply surface を移管する
- [ ] 1.4 `docs/settings-schema.json` を定義する（Ollama endpoint / model / timeout / appearance）
- [ ] 1.5 `katana-acp-client` の `cargo tree` に `egui` が含まれないことを確認する
- [ ] 1.6 `katana-chat-ui` の `cargo tree` に `katana-core` / `katana-ui` が含まれないことを確認する

---

## 2. ACP transport と Ollama adapter を本実装にする

### 完了条件

- [ ] 2.1 stdio JSON-RPC transport を実装する
- [ ] 2.2 ACP capability negotiation を実装する
- [ ] 2.3 Ollama プロセス起動・停止・health check を adapter として実装する
- [ ] 2.4 Ollama model 一覧取得と turn 送受信を実装する
- [ ] 2.5 Ollama adapter の unit test（mock server 可）を追加する
- [ ] 2.6 Ollama 未起動 / timeout / invalid response を `AcpError` として返す

---

## 3. settings 統合方式を実装する

### 完了条件

- [ ] 3.1 path 渡し方式が動作する（`ChatPanel::builder().settings_path(path).build()`）
- [ ] 3.2 コールバック方式が動作する（`.settings_slice(json).on_settings_changed(cb).build()`）
- [ ] 3.3 スキーマ外キーを無視することを test する
- [ ] 3.4 path 渡し / コールバック両方式の integration test が通る

---

## 4. v0.1.0 release

### 完了条件

- [ ] 4.1 `cargo fmt` / `cargo clippy --workspace -- -D warnings` / `cargo test --workspace` が通る
- [ ] 4.2 release tag `v0.1.0` を切り GitHub Release を作成する
- [ ] 4.3 KatanA v0.23.0 が `katana-chat-ui = { git = "...", tag = "v0.1.0" }` でビルドできることを確認する
