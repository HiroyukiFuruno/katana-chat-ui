## Why

chat widget（egui）と ACP/Ollama interface（egui 非依存）を最初から分離する。v0.0.1 は **egui を一切持たない** neutral interface crate のみを対象とし、kcf v0.0.1 と同じパターンを踏む。

`DocumentContext` は v0.0.1 の段階から mandatory とする。これにより将来 ACP agent が「どのドキュメントを対象にしているか」を常に受け取れる設計を確立する。

## What Changes

- `katana-acp-client`（neutral crate、egui 非依存）に以下を定義する：
  - `AiProvider` trait：`id()` / `display_name()` / `execute()` / `is_available()` / `capabilities()` / `list_models()`
  - `DocumentContext`（mandatory）：現在開いているドキュメントの URI・内容・カーソル位置・diagnostics
  - `AiIntent` enum：`Modify` / `Create` / `Autofix`
  - `AiRequest` / `AiResponse` DTO
- `OllamaProvider` を実装する：
  - `ureq` ベース（非同期不要の MVP）
  - `/api/generate`（turn 送受信）/ `/api/tags`（model 一覧）
  - endpoint normalize、timeout 設定
- unit test のみ（egui なし、CI 上での実行可能）
- `v0.0.1` として release tag を切る

## Capabilities

### New Capabilities

- `acp-interface`: `AiProvider` trait + `DocumentContext` + `AiIntent` の neutral contract
- `ollama-provider`: ureq ベースの Ollama adapter

## Implementation Reference

KatanA `release/v0.23.0` ブランチに実装済みの以下を参照・移管する：

- `katana-core/src/ai/types.rs` → `AiProvider` trait と DTO の原型
- `katana-core/src/ai/registry.rs` → provider registry パターン
- `katana-core/src/ai/ollama.rs` → `OllamaProvider` 実装

差分：`DocumentContext` を v0.0.1 段階から mandatory にする（v0.23.0 では optional）。

## Impact

- `crates/katana-acp-client/` — neutral interface + OllamaProvider（egui 非依存）
