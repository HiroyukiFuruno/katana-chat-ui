# Tasks: v0.2.0 — document generation + translation overlay

> v0.1.0 で確立した chat state / Floem impl の上に、LLM による文書生成と動的翻訳 overlay を追加する。

## Branch Rule

`release/v0.2.0` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [ ] v0.1.0 がリリース済みであること
- [ ] `DocumentGenerationRequest` / `TranslationOverlayState` の public API が設計済みであること
- [ ] 翻訳対象テキストの抽出戦略（全文 / 選択範囲 / diff 単位）が合意済みであること

---

## 1. katana-chat-ui: document generation state を追加する

### 完了条件

- [ ] 1.1 `crates/katana-chat-ui/src/generation/request.rs` に以下を実装する
  - `DocumentGenerationRequest`: context（DocumentContext）+ instruction（String）+ target_path（PathBuf）
  - `DocumentGenerationPromptBuilder`: `DocumentGenerationRequest` → LLM prompt 文字列
  - `DocumentGenerationResponseNormalizer`: LLM response → `GeneratedDocumentCandidate`
- [ ] 1.2 `crates/katana-chat-ui/src/generation/state.rs` に `DocumentGenerationState` / `GeneratedDocumentCandidate` を実装する
- [ ] 1.3 `crates/katana-chat-ui/src/translation/state.rs` に `TranslationOverlayState` を実装する
  - 対象テキスト（source_lang / target_lang / segments）
  - translated segments のキャッシュ
  - overlay 表示フラグ
- [ ] 1.4 unit test を追加する
  - `DocumentGenerationPromptBuilder` → prompt 内容確認
  - `TranslationOverlayState` の segment キャッシュ動作確認

---

## 2. katana-chat-ui-floem: document generation / translation overlay View を追加する

### 準備完了条件

- [ ] Task 1 完了

### 完了条件

- [ ] 2.1 `crates/katana-chat-ui-floem/src/generation.rs` に `DocumentGenerationView` を実装する
  - instruction 入力欄
  - 生成中インジケーター
  - 生成結果プレビュー（before/after diff 表示）
  - confirm / reject ボタン
- [ ] 2.2 `crates/katana-chat-ui-floem/src/translation.rs` に `TranslationOverlayView` を実装する
  - テキスト上に overlay 表示（元テキスト / 翻訳テキスト）
  - 言語選択（source / target）
  - キャッシュ済みセグメントはネットワーク不要で即表示
- [ ] 2.3 `crates/katana-chat-ui-floem/src/lib.rs` に `DocumentGenerationView` / `TranslationOverlayView` を pub re-export する
- [ ] 2.4 smoke test（headless）を追加する

---

## 3. settings schema を更新する

### 準備完了条件

- [ ] Task 2 完了

- [ ] 3.1 `docs/settings-schema.json` に以下を追加する
  - `generation.enabled`: bool
  - `translation.enabled`: bool
  - `translation.default_target_lang`: string（デフォルト: `"en"`）
- [ ] 3.2 スキーマバリデーション integration test を更新する

---

## 4. 品質ゲート

### 準備完了条件

- [ ] Task 3 完了

- [ ] 4.1 `cargo fmt --check` が通ること
- [ ] 4.2 `cargo clippy --workspace -- -D warnings` が通ること
- [ ] 4.3 `cargo test --workspace` が通ること

---

## 5. v0.2.0 release

### 準備完了条件

- [ ] Task 4 完了

- [ ] 5.1 `release/v0.2.0` ブランチから PR を作成し master へ merge する
- [ ] 5.2 release tag `v0.2.0` を切り GitHub Release を作成する
