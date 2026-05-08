# kcu OpenSpec 意向照合レビュー

## 結論

ユーザー要件に合わせて、初期 OpenSpec 計画を再編した。

現在の source of truth は次の 3 change。

- `v0-1-0-chat-widget-floem`: chat UX foundation
- `v0-2-0-secure-connector-and-usage`: secure connector and account usage
- `v0-3-0-vendor-adapter-expansion`: vendor adapter expansion

旧 `v0-2-0-document-generation-and-translation` は、chat / connector foundation が固まる前に進めると責務が混ざるため archive に退避した。

## 反映した前提

- kcu は特定の親アプリを知らない。
- kcu core は egui に依存しない。
- katana-chat-ui product は UI 込みで提供する。標準 UI は framework 別 crate が持つ。
- API-only 利用は標準 UI に満足できない場合の customization path として扱う。
- 実動作検証用 UI harness は `crates/` に含めず、外部 host fixture として kcu を実際に取り込む E2E にする。
- ACP 対応 agent は ACP を primary path として扱う。
- ACP 非対応 provider は direct connector として扱い、secret は平文 settings に保存しない。
- secret は LLM request ごとに取得、復号、利用、破棄する。
- vendor ごとの差分は SDK 型を UI に漏らさず、capability / adapter 境界で扱う。

## 要件別対応

| 要件 | 対応先 |
|---|---|
| ACP による接続設定の簡略化 | `v0-2-0-secure-connector-and-usage` |
| ACP 非対応時の secure secret | `v0-2-0-secure-connector-and-usage` |
| multi vendor | `v0-3-0-vendor-adapter-expansion` |
| Ollama MVP | `v0-3-0-vendor-adapter-expansion` |
| Claude Code / Codex | ACP agent adapter として `v0-3-0-vendor-adapter-expansion` |
| GitHub Copilot | adapter / extension surface がある場合のみ対応、direct provider 化禁止 |
| Bedrock / Vertex AI | cloud-direct provider として `v0-3-0-vendor-adapter-expansion` |
| 標準 AI chat 入力 | `v0-1-0-chat-widget-floem` |
| ファイル添付 / path drop | `v0-1-0-chat-widget-floem` |
| Markdown subset | `v0-1-0-chat-widget-floem` |
| user / agent 視覚差分 | `v0-1-0-chat-widget-floem` |
| theme injection | `v0-1-0-chat-widget-floem` |
| SVG icon override | `v0-1-0-chat-widget-floem` |
| context token 表示 | `v0-1-0-chat-widget-floem` |
| account / usage 表示 | `v0-2-0-secure-connector-and-usage` |
| vendor ごとの model / thinking / permission | `v0-2-0-secure-connector-and-usage` と `v0-3-0-vendor-adapter-expansion` |
| 外部 host としての実動作 E2E | `v0-1-0-chat-widget-floem`。Final Verification 直前に実施し、ユーザー FB を tasks.md に task 化する |

## 検証結果

- `npx -y @fission-ai/openspec validate v0-1-0-chat-widget-floem`: pass
- `npx -y @fission-ai/openspec validate v0-2-0-secure-connector-and-usage`: pass
- `npx -y @fission-ai/openspec validate v0-3-0-vendor-adapter-expansion`: pass
- `npx -y @fission-ai/openspec status --change ... --json`: 3 change とも `isComplete: true`
- `just check`: pass
- `git diff --check`: pass

## 技術判断メモ

- ACP は stdio JSON-RPC を安定 transport として扱う。HTTP / WebSocket は別 change に送る。
- ACP の session config options は model、mode、thought level などの設定 UI に使う。
- usage / context status は ACP RFD の形に寄せつつ、kcu では optional provider capability として扱う。
- GitHub Copilot は kcu core から汎用 direct provider として扱わない。ACP-compatible adapter または host extension surface が確認できる場合だけ対応する。

## 追加で直した backing files

- `README.md`: host-agnostic 説明へ修正。
- `Cargo.toml`: workspace dependency から `egui` を削除。
- `crates/katana-chat-ui/Cargo.toml`: `egui` dependency を削除。
- `crates/katana-chat-ui/src/lib.rs`: `egui::Ui` public API を削除し、render model scaffold に変更。
- `crates/katana-acp-client/src/lib.rs`: 特定親アプリ前提の説明を削除。
