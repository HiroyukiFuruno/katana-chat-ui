# Tasks: v0.1.0 — chat widget (Floem) + autofix diff surface

> neutral chat state（`katana-chat-ui`）と Floem impl（`katana-chat-ui-floem`）を確立する。
> v0.0.1 で確立した `katana-acp-client` の上に chat session 管理・autofix state・diff surface・settings schema を実装する。

## Branch Rule

`release/v0.1.0` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [ ] `katana-acp-client` v0.0.1 がリリース済みであること
- [ ] `design.md` の Architecture・State Flow・Streaming 実装方針セクションをレビュー済みであること
- [ ] Floem の headless test（スクリーンショット test または同等機能）の利用可否を確認済みであること
- [ ] `Cargo.toml` に追加する Floem / cosmic-text / vello のバージョンが確定していること

---

## 0. ワークスペース準備

- [ ] 0.1 `Cargo.toml` の `[workspace] members` に `crates/katana-chat-ui` と `crates/katana-chat-ui-floem` を追加する
- [ ] 0.2 `crates/katana-chat-ui/Cargo.toml` を作成する（依存: `katana-acp-client`、UI フレームワーク非依存）
- [ ] 0.3 `crates/katana-chat-ui-floem/Cargo.toml` を作成する（依存: `katana-chat-ui`・`katana-acp-client`・`floem`・`cosmic-text`・`vello`）
- [ ] 0.4 `cargo build --workspace` が通ること

---

## 1. katana-chat-ui: neutral chat state を実装する

### 実施内容

UI フレームワーク非依存の chat state 層。`floem` / `egui` を一切 import しない。

### 完了条件

- [ ] 1.1 `crates/katana-chat-ui/src/session.rs` に `ChatSession` を実装する
  - message 履歴（`Vec<ChatMessage>`）
  - pending flag
  - streaming buffer（`response_rx: Option<Receiver<String>>`）
  - turn 管理（`push_user_turn` / `push_assistant_turn` / `clear`）
- [ ] 1.2 `crates/katana-chat-ui/src/autofix/request.rs` に以下を実装する
  - `AutofixRequestBuilder`: path + original_content + diagnostics → `FileAutofixRequest`
  - `AutofixPromptBuilder`: `FileAutofixRequest` → LLM prompt 文字列（`<<KATANA_AUTOFIX_CONTENT>>` マーカー形式）
  - `AutofixResponseNormalizer`: LLM response → `FileAutofixCandidate`
- [ ] 1.3 `crates/katana-chat-ui/src/autofix/state.rs` に `AutofixState` / `FileAutofixRequest` / `FileAutofixCandidate` を実装する
- [ ] 1.4 `crates/katana-chat-ui/src/diff.rs` に `DiffPreviewState`（行単位差分: before/after lines）を実装する
- [ ] 1.5 `crates/katana-chat-ui/src/config.rs` に `ChatConfig` を実装する
  - `ChatConfig::from_path(path: &Path)`: JSON ファイル path 渡し
  - `ChatConfig::from_slice(json: &str, on_change: impl Fn(&str))`: コールバック方式
- [ ] 1.6 `cargo tree -p katana-chat-ui | grep -E "floem|egui|vello"` が空であること
- [ ] 1.7 unit test を追加する
  - `ChatSession` の turn 管理
  - `AutofixRequestBuilder` → prompt 文字列（マーカー形式確認）
  - `DiffPreviewState` の行差分計算

---

## 2. katana-chat-ui-floem: Floem impl を実装する

### 準備完了条件

- [ ] Task 1 完了

### 実施内容

Floem + cosmic-text を使った chat UI の rendering 層。`katana-chat-ui`（neutral state）を消費する。
host application は `ChatPanelView` を embed するだけ。

### 完了条件

- [ ] 2.1 `crates/katana-chat-ui-floem/src/panel.rs` に `ChatPanelView` を実装する
  - Floem View として実装（`impl View for ChatPanelView`）
  - サイドパネル + overlay、固定/非固定切り替え、開閉アニメーション
  - メッセージ一覧（ユーザー / アシスタント）のスクロール表示
  - 入力欄（cosmic-text ベース: IME 完全対応・カラー絵文字対応）
  - 送信ボタン / streaming 中の中断ボタン
  - provider 未設定 / unavailable / pending の disabled state 表示
- [ ] 2.2 `crates/katana-chat-ui-floem/src/autofix.rs` に `AutofixDiffView` を実装する
  - `impl View for AutofixDiffView`
  - 行単位 before/after 差分表示（削除行: 赤、追加行: 緑）
  - confirm / reject / apply ボタン
- [ ] 2.3 `crates/katana-chat-ui-floem/src/lib.rs` で `ChatPanelView` / `AutofixDiffView` を pub re-export する
- [ ] 2.4 `cargo tree -p katana-chat-ui-floem | grep -E "egui|epaint"` が空であること
- [ ] 2.5 smoke test（Floem headless / screenshot test）を追加する

---

## 3. settings schema を整備する

### 準備完了条件

- [ ] Task 2 完了

- [ ] 3.1 `docs/settings-schema.json` を定義する
  - `ollama.endpoint`: string（デフォルト: `"http://localhost:11434"`）
  - `ollama.selected_model`: string | null
  - `ollama.timeout_secs`: number（デフォルト: 30）
  - `chat.enabled`: bool
  - `autofix.enabled`: bool
- [ ] 3.2 `ChatConfig::from_path` / `ChatConfig::from_slice` のスキーマバリデーション integration test を追加する
- [ ] 3.3 スキーマ外キーは無視され、エラーにならないことを確認する

---

## 4. 品質ゲート

### 準備完了条件

- [ ] Task 3 完了

- [ ] 4.1 `cargo fmt --check` が通ること
- [ ] 4.2 `cargo clippy --workspace -- -D warnings` が通ること
- [ ] 4.3 `cargo test --workspace` が通ること

---

## 5. v0.1.0 release

### 準備完了条件

- [ ] Task 4 完了

### 完了条件（Definition of Done）

- [ ] 5.1 `release/v0.1.0` ブランチから PR を作成し master へ merge する
- [ ] 5.2 release tag `v0.1.0` を切り GitHub Release を作成する
- [ ] 5.3 以下がすべて満たされていること
  - `cargo tree -p katana-chat-ui | grep -E "floem|egui|vello"` が空
  - `cargo tree -p katana-chat-ui-floem | grep -E "egui|epaint"` が空
  - `cargo test --workspace` が通る
  - `ChatPanelView` / `AutofixDiffView` が pub re-export されていること
  - `docs/settings-schema.json` が存在すること
