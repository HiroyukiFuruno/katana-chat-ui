# Tasks: v0.1.0 — chat UX foundation

> kcu を framework-neutral core として固定し、標準的な AI chat 入力と表示 contract を作る。

## Branch Rule

本 change では、以下のブランチ運用を適用する。

- 統合ブランチ: `v0-1-0-chat-widget-floem`
- task ブランチ: `v0-1-0-chat-widget-floem-task<N>`
- feedback ブランチ: `v0-1-0-chat-widget-floem-feedback`

base branch を固定で決めず、PR 作成時に `/create_pull_request` で確認する。

---

## 0. Foundation Cleanup

### Definition of Ready

- [x] `git status --short` で既存差分を確認している
- [x] active OpenSpec がこの change を source of truth とすることを確認している

### Definition of Done

- [x] `Cargo.toml` と `crates/katana-chat-ui` から `egui` dependency と `egui::` public API が消えている
- [x] README が特定 host app / egui host 前提ではなく、host-agnostic な説明になっている
- [x] `rg -n "egui::|<<KATANA" crates/katana-chat-ui README.md` が一致しない
- [x] `/self-review` を実行し、指摘を解消している
- [x] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [x] ユーザーへ結果を報告し、コミット前に停止している

---

## 1. Framework-neutral Chat Core

### Definition of Ready

- [ ] Task 0 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] `ChatSession` / `ChatMessage` / `MessageRole` / `MessageStatus` / `ChatRenderModel` を実装している
- [ ] user / assistant / tool / system を視覚差分として表現できる render model がある
- [ ] streaming 中、送信不可、provider 未設定、エラーの state を持つ
- [ ] unit test が turn 管理と role 表示 contract を検証している
- [ ] `cargo tree -p katana-chat-ui | grep -E "floem|egui|vello|eframe"` が空である
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 2. Input, Attachment, and Markdown

### Definition of Ready

- [ ] Task 1 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] `ChatInputDraft` が multiline text、attachment tray、submit / cancel intent を持つ
- [ ] file attachment、image attachment、path drop を `Attachment` として表現している
- [ ] path drop は host callback で読み取り、kcu core が勝手に filesystem を読まない
- [ ] ACP の text / image / embedded resource content block へ変換できる contract がある
- [ ] Markdown subset は code block、inline code、blockquote、ordered list、unordered list、link に限定されている
- [ ] HTML block、script、raw style を描画しない test がある
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 3. Theme, SVG Icons, and Usage Surface

### Definition of Ready

- [ ] Task 2 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] `ThemeTokens` が color / spacing / typography / status color を受け取れる
- [ ] button icon はすべて SVG asset id で表現され、host override できる
- [ ] context token usage を used / size / percentage / status として表示できる
- [ ] account usage は取得できない場合も `Unavailable(reason)` として表示状態を持つ
- [ ] unit test が theme merge、icon override、usage percentage を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 4. Floem Reference Implementation

### Definition of Ready

- [ ] Task 3 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] `katana-chat-ui-floem` が `ChatRenderModel` だけを読んで描画する
- [ ] IME 対応 multiline composer、attachment tray、message list、usage meter、settings trigger を表示する
- [ ] Floem crate にも `egui` / `eframe` / host app 固有 dependency がない
- [ ] reference UI は E2E harness に依存せず、通常 crate として利用できる
- [ ] smoke test は reference UI crate 内の最小範囲に留めている
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 5. External Host E2E Verification

> この task は Final Verification の直前に実施する。kcu を実際に取り込む外部 host 視点の確認であり、`crates/` 配下に検証用 app を置かない。

### Definition of Ready

- [ ] Task 4 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] `tools/e2e-host-app/` のような非公開 host fixture がある
- [ ] fixture は workspace member ではなく、配布対象 crate に含まれない
- [ ] fixture は kcu を downstream dependency として実際に取り込む
- [ ] E2E は起動、描画、入力、添付、path drop、send、stop、usage 表示を検証する
- [ ] E2E 結果とスクリーンショットまたは実行ログをユーザーに提示できる
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 6. User Review and Feedback Tasking

> ユーザーレビューの指摘は `[/]` で閉じる。通常 task の `[x]` と混ぜない。

- [ ] 6.1 Task 5 の E2E 結果、スクリーンショットまたは実行ログ、残課題をユーザーへ提示する
- [ ] 6.2 ユーザーからのフィードバックを、この section の `User Feedback Tasks` に `[ ]` で task 化する
- [ ] 6.3 defer 指定がないフィードバックをすべて解消する
- [ ] 6.4 解消済みフィードバックは `[/]` に更新し、通常 task の `[x]` と混ぜない
- [/] 6.5 実動作検証用 UI harness は `crates/` に含めず、外部 host fixture として kcu を実際に取り込む E2E にする

### User Feedback Tasks

- [/] 外部 host E2E を v0.1.0 の計画に含め、Final Verification 直前に実施する
- [/] E2E 結果をユーザーに提示し、ユーザー FB を tasks.md に task 化する構成にする

---

## 7. Final Verification

- [ ] 7.1 `just check` が通る
- [ ] 7.2 `just ast-lint` が通る
- [ ] 7.3 `npx -y @fission-ai/openspec validate "v0-1-0-chat-widget-floem"` が通る
- [ ] 7.4 `/openspec-verify-change` で Critical がない
- [ ] 7.5 PR 作成が必要な場合は `/create_pull_request` を使う
- [ ] 7.6 merge 後、必要なら `/openspec-archive-change` を使う
