## ADDED Requirements

### Requirement: AiProvider trait と DocumentContext を neutral interface として提供しなければならない

システムは、`AiProvider` trait（`id` / `display_name` / `execute` / `is_available` / `capabilities` / `list_models`）、`DocumentContext`（mandatory）、`AiIntent` enum（`Modify` / `Create` / `Autofix`）、`AiRequest` / `AiResponse` DTO を `katana-acp-client` neutral crate として提供しなければならない（MUST）。

#### Scenario: KatanA から AiProvider trait を呼ぶ

- **WHEN** KatanA が `AiProvider::execute(&AiRequest)` を呼ぶ
- **THEN** システムは active provider が `AiResponse` を返す
- **THEN** `AiRequest` には `DocumentContext` が mandatory に含まれる

#### Scenario: katana-acp-client は egui に依存しない

- **WHEN** `cargo tree -p katana-acp-client` を実行する
- **THEN** `egui` / KatanA UI state は含まれない

### Requirement: Ollama provider を ureq ベースで提供しなければならない

システムは、Ollama を最初の AiProvider 実装として `ureq` ベースで提供し、`/api/generate` 経由の turn 送受信と `/api/tags` 経由の model 一覧取得を実装しなければならない（MUST）。

#### Scenario: Ollama に turn を送信する

- **WHEN** KatanA が `OllamaProvider` 経由で `AiRequest` を送信する
- **THEN** Ollama endpoint の `/api/generate` を呼び、`AiResponse` を返す
- **THEN** endpoint URL の normalize と timeout 設定が適用される

#### Scenario: Ollama model 一覧を取得する

- **WHEN** KatanA が `OllamaProvider::list_models()` を呼ぶ
- **THEN** `/api/tags` を呼び、利用可能な `AiModel` のリストを返す
