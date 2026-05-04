## Context

v0.1.0 の `ChatSession` / `AiProvider` / Floem impl が完成している前提。
LLM を「chat 応答」以外の用途（文書生成・翻訳）に拡張する。

## Goals

- 文書生成・翻訳 overlay を neutral state 層に追加し、Floem View で表示する。
- 既存の `ChatSession` / `AutofixState` とは独立した state として管理する。
- 翻訳 segment キャッシュで UX のラグを最小化する。

## Non-Goals

- 追加 vendor adapter — v0.3.0。
- 履歴永続化・複数会話管理 — v0.4.0。
- 翻訳 API（DeepL / Google Translate 等）外部サービスとの統合 — 翻訳は Ollama 経由の LLM で行う。

## Architecture 追加分

```
katana-chat-ui（追加）
  generation/
    request.rs      DocumentGenerationRequest / PromptBuilder / ResponseNormalizer
    state.rs        DocumentGenerationState / GeneratedDocumentCandidate
  translation/
    state.rs        TranslationOverlayState（segments / cache / lang / enabled）

katana-chat-ui-floem（追加）
  generation.rs     DocumentGenerationView（impl View）
  translation.rs    TranslationOverlayView（impl View）
```

## 翻訳対象テキストの抽出戦略

翻訳対象は **文書全体** を基本とする。v0.2.0 では選択範囲・diff 単位の抽出は対象外。
host application は `TranslationOverlayState::new(full_text, src_lang, tgt_lang)` を呼ぶ。
将来バージョンで選択範囲対応が必要になった場合は別 spec で検討する。

## host write callback の型

`DocumentGenerationView` の confirm ボタン押下時に呼ばれる callback の型：

```rust
on_confirm: Arc<dyn Fn(GeneratedDocumentCandidate) + Send + Sync>
```

host は受け取った `candidate.content` をファイルに書き込む責務を持つ。

## State Flow

```
[document generation]
DocumentGenerationRequest（ctx + instruction + target_path）
  └─► DocumentGenerationPromptBuilder ──► LLM prompt
  └─► OllamaProvider::execute ──► LLM response
  └─► DocumentGenerationResponseNormalizer ──► GeneratedDocumentCandidate
  └─► DiffPreviewState（既存）──► DocumentGenerationView（before/after）
  └─► confirm ──► host write callback

[translation overlay]
TranslationOverlayState.request_segments(text, src_lang, tgt_lang)
  └─► キャッシュヒット ──► 即表示
  └─► キャッシュミス  ──► OllamaProvider::execute ──► segments キャッシュ更新 ──► 表示
```

## Verification

- `cargo tree -p katana-chat-ui | grep -E "floem|egui"` が空
- `DocumentGenerationPromptBuilder` → prompt 内容 unit test が通る
- `TranslationOverlayState` segment キャッシュ動作 unit test が通る
- `DocumentGenerationView` / `TranslationOverlayView` Floem headless smoke test が通る
- settings schema バリデーション integration test が通る
