# Tasks: v0.0.1 ACP Client Foundation（neutral interface のみ）

> egui は一切含まない。Chat widget / autofix diff surface は v0.1.0 で実装する。
> v0.0.1 の完了条件は `katana-acp-client` neutral crate のリリースのみ。

## Branch Rule

`master` 直接作業（scaffold 段階のため）。

---

## 準備完了条件（Definition of Ready）

- [ ] KatanA `release/v0.23.0` ブランチの `katana-core/src/ai/` を参照済みであること
- [ ] `DocumentContext` の mandatory フィールド（URI / 内容 / カーソル位置 / diagnostics）が合意されていること

---

## 1. katana-acp-client neutral interface を定義する

- [ ] 1.1 `AiProvider` trait を定義する（`id` / `display_name` / `execute` / `is_available` / `capabilities` / `list_models`）
- [ ] 1.2 `DocumentContext` struct を定義する（mandatory）
- [ ] 1.3 `AiIntent` enum を定義する（`Modify` / `Create` / `Autofix`）
- [ ] 1.4 `AiRequest` / `AiResponse` DTO を定義する
- [ ] 1.5 `cargo check` が egui なしで通ることを確認する

---

## 2. OllamaProvider を実装する

### 準備完了条件

- [ ] Task 1 完了

- [ ] 2.1 `ureq` を dependency に追加する
- [ ] 2.2 `/api/generate` 経由の turn 送受信を実装する
- [ ] 2.3 `/api/tags` 経由の model 一覧取得を実装する
- [ ] 2.4 endpoint URL normalize と timeout 設定を実装する
- [ ] 2.5 `OllamaProvider` が `AiProvider` trait を実装していることを確認する

---

## 3. unit test を追加する

- [ ] 3.1 `OllamaProvider::new()` のデフォルト endpoint normalize テスト
- [ ] 3.2 `AiRequest` / `AiResponse` の serde roundtrip テスト
- [ ] 3.3 `cargo test` が CI 上（Ollama なし）で通ることを確認する

---

## 4. v0.0.1 リリース

- [ ] 4.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 4.2 `v0.0.1` release tag を切り push する
- [ ] 4.3 KatanA 側から `{ git = "...", tag = "v0.0.1" }` で参照できることを `cargo tree` で確認する
