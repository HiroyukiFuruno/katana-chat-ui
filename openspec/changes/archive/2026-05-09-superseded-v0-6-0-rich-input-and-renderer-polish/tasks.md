# Tasks: v0.6.0 — rich input and renderer polish

> OS file picker、drag and drop、画像貼り付け、絵文字、時間表示、provider icon、`katana-document-viewer` 連携を追加する。

## Branch Rule

本 change では、以下のブランチ運用を適用する。

- 統合ブランチ: `v0-6-0-rich-input-and-renderer-polish`
- task ブランチ: `v0-6-0-rich-input-and-renderer-polish-task<N>`
- feedback ブランチ: `v0-6-0-rich-input-and-renderer-polish-feedback`

base branch を固定で決めず、PR 作成時に `/create_pull_request` で確認する。

---

## 0. OS File and Drag-and-Drop Attachment

### Definition of Ready

- [ ] v0.1.0 の attachment intent と attachment interface が完了している
- [ ] OS file、host resource、virtual attachment の区別が spec にある
- [ ] host callback の成功、失敗、取消を表す typed result が design.md にある
- [ ] `git status --short` で既存差分を確認している

### Definition of Done

- [ ] OS file picker から file を追加できる
- [ ] drag and drop で OS file を追加できる
- [ ] drag and drop で host logical resource と virtual attachment を OS file と区別できる
- [ ] 添付済み file を composer 内で取り消せる
- [ ] attachment 追加と取消で draft、cursor、IME state、既存 attachment が壊れない
- [ ] host callback の失敗時に fallback せず、添付失敗として表示できる
- [ ] unit test が attachment source の区別と取消を検証している
- [ ] integration test が host callback 成功、失敗、取消を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 1. Clipboard Image and Emoji Input

### Definition of Ready

- [ ] Task 0 の実装、自己レビュー、検証、報告が完了している
- [ ] clipboard image を host callback へ渡す payload が design.md にある
- [ ] `git status --short` で既存差分を確認している

### Definition of Done

- [ ] クリップボード画像を attachment として追加できる
- [ ] 画像貼り付けで draft、cursor、IME state、既存 attachment が壊れない
- [ ] macOS 絵文字パレットから絵文字を挿入できる
- [ ] 絵文字挿入で cursor 位置と placeholder 表示が壊れない
- [ ] clipboard image の decode / file resolve は host callback に委譲される
- [ ] unit test が画像貼り付けと絵文字挿入の draft / cursor 保持を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 2. Message Timing

### Definition of Ready

- [ ] Task 1 の実装、自己レビュー、検証、報告が完了している
- [ ] render model に timing を追加する場所が決まっている
- [ ] text catalog に時刻表示と elapsed 表示の文言がある

### Definition of Done

- [ ] user message が sent at を持つ
- [ ] agent message が started / completed / failed / stopped at を持つ
- [ ] 応答中は elapsed duration が更新され、完了 / 失敗 / 停止で固定される
- [ ] timing は UI 表示と session snapshot の両方に含まれる
- [ ] unit test が running、completed、failed、stopped の timing state を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 3. Provider Selector Icons

### Definition of Ready

- [ ] Task 2 の実装、自己レビュー、検証、報告が完了している
- [ ] provider fact registry に icon source を追加する方針が design.md にある
- [ ] 公式 icon がない provider の扱いが `IconMissing(provider_id)` として決まっている

### Definition of Done

- [ ] provider selector の候補行に provider icon を表示する
- [ ] 会話 title icon と selector icon が同じ registry を使う
- [ ] 公式 icon がない provider は `IconMissing(provider_id)` を返す
- [ ] `+`、汎用コード記号、欠落アイコンを provider icon の代替として使わない
- [ ] snapshot test が provider icon の asset mapping を固定している
- [ ] AST lint が provider icon の silent fallback を検知する
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 4. Markdown Renderer Adapter

### Definition of Ready

- [ ] Task 3 の実装、自己レビュー、検証、報告が完了している
- [ ] `katana-document-viewer` の利用可能 API が確認されている
- [ ] v0.1.0 の Markdown subset と code fence contract が regression test として残っている

### Definition of Done

- [ ] `MarkdownRenderer` interface が `comrak` adapter と `katana-document-viewer` adapter を差し替えられる
- [ ] `katana-document-viewer` adapter 有効時も v0.1.0 の Markdown subset が壊れない
- [ ] raw HTML は renderer 差し替え後もテキスト扱いになる
- [ ] code fence の途中途切れ regression test が通る
- [ ] adapter contract test が heading、list、task list、table、code block を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 5. User Review

> ユーザーレビューの指摘は `[/]` で閉じる。通常 task の `[x]` と混ぜない。

- [ ] 5.1 実装結果と検証結果をユーザーへ提示する
- [ ] 5.2 ユーザーからのフィードバックをこの tasks.md に追記する
- [ ] 5.3 defer 指定がないフィードバックをすべて解消する

---

## 6. Final Verification

- [ ] 6.1 `just check` が通る
- [ ] 6.2 `just ast-lint` が通る
- [ ] 6.3 `npx -y @fission-ai/openspec validate "v0-6-0-rich-input-and-renderer-polish"` が通る
- [ ] 6.4 `/openspec-verify-change` で Critical がない
- [ ] 6.5 PR 作成が必要な場合は `/create_pull_request` を使う
- [ ] 6.6 merge 後、必要なら `/openspec-archive-change` を使う
