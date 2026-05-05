## Why

v0.1.0 で確立した chat state / Floem impl の上に、LLM を使った文書生成と動的翻訳 overlay を追加する。

- **文書生成**：instruction を与えると LLM が文書全体または選択範囲を生成・置換する。before/after diff preview → confirm / reject flow で適用する。
- **翻訳 overlay**：テキスト上に元言語 / 翻訳言語を重ねて表示する。キャッシュ済みセグメントはネットワーク不要で即表示。

## What Changes

### katana-chat-ui（neutral state 追加）

- `DocumentGenerationRequest` / `DocumentGenerationPromptBuilder` / `DocumentGenerationResponseNormalizer`
- `DocumentGenerationState` / `GeneratedDocumentCandidate`
- `TranslationOverlayState`（source / target segments・キャッシュ・表示フラグ）

### katana-chat-ui-floem（View 追加）

- `DocumentGenerationView`：instruction 入力 / 生成中インジケーター / before-after diff / confirm・reject
- `TranslationOverlayView`：テキスト上 overlay / 言語選択 / キャッシュ即表示

### docs/settings-schema.json（更新）

- `generation.enabled` / `translation.enabled` / `translation.default_target_lang`

## Capabilities

### New Capabilities

- `document-generation`: LLM 文書生成 state + Floem View
- `translation-overlay`: 動的翻訳 overlay state + Floem View

### Inherited from v0.1.0

- `chat-state` / `chat-ui-floem` / `acp-interface` / `ollama-provider`（変更なし）

## Impact

- `crates/katana-chat-ui/` — generation / translation state 追加
- `crates/katana-chat-ui-floem/` — generation / translation View 追加
- `docs/settings-schema.json` — generation / translation 項目追加
