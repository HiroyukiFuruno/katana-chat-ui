# Tasks: v0.5.0 — output extension views

> file / diff / tool result / permission request を標準 UI の拡張表示として扱い、実処理は host / provider に委譲する。

## Branch Rule

本 change では、以下のブランチ運用を適用する。

- 統合ブランチ: `v0-5-0-output-extension-views`
- task ブランチ: `v0-5-0-output-extension-views-task<N>`
- feedback ブランチ: `v0-5-0-output-extension-views-feedback`

base branch を固定で決めず、PR 作成時に `/create_pull_request` で確認する。

---

## 0. Output Extension Render Model

### Definition of Ready

- [ ] v0.1.0 の output contract と debug hover が完了している
- [ ] `docs/chat-ui-spec.ja.md` の output section を確認している
- [ ] `OutputExtensionView` が扱う kind を design.md に列挙している
- [ ] `git status --short` で既存差分を確認している

### Definition of Done

- [ ] `OutputExtensionView` が id、source message id、generation id、kind、status、summary、actions を持つ
- [ ] `FileCandidate`、`DiffCandidate`、`ToolResult`、`PermissionRequest`、`CommandExecution`、`ReadResult`、`FetchResult` を表現できる
- [ ] output view は chat 本文 message と別 contract である
- [ ] unit test が各 kind の render model 変換を検証している
- [ ] debug JSON と output extension view が混ざらない regression test がある
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 1. File and Diff Views

### Definition of Ready

- [ ] Task 0 の実装、自己レビュー、検証、報告が完了している
- [ ] file 書き込みと diff 適用を host / provider に委譲する intent 名が確定している
- [ ] Zed の diff 表示参照箇所を design.md に記載している

### Definition of Done

- [ ] file candidate view が path、operation、summary、open intent を表示できる
- [ ] diff candidate view が target path、unified diff、status、open / apply / reject intent を表示できる
- [ ] kcu core は file 書き込みや diff 適用を直接実行しない
- [ ] intent は output id と source message id を必ず持つ
- [ ] unit test が apply / reject intent の payload を検証している
- [ ] UI adapter test が長い diff を本文吹き出しへ混ぜないことを検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 2. Tool Result and Permission Views

### Definition of Ready

- [ ] Task 1 の実装、自己レビュー、検証、報告が完了している
- [ ] permission decision の action intent と provider callback の境界が design.md に記載されている
- [ ] command execution / read / editing / fetch の状態と output view の対応が spec にある

### Definition of Done

- [ ] tool result view が title、status、summary、stdout / stderr reference、copy intent を表示できる
- [ ] permission request view が request reason、対象 tool、approve / reject intent を表示できる
- [ ] 実際の permission 承認処理は host / provider callback へ委譲する
- [ ] command execution、read、editing、fetch の状態表示から output view を開ける
- [ ] unit test が approve / reject intent と unsupported permission state を検証している
- [ ] AST lint が provider ID 直書き分岐を検知する
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 3. Framework Adapter Verification

### Definition of Ready

- [ ] Task 2 の実装、自己レビュー、検証、報告が完了している
- [ ] egui / Floem / GPUI の output extension slot が存在する
- [ ] manual harness の確認項目が docs に追記されている

### Definition of Done

- [ ] egui / Floem / GPUI で file candidate、diff candidate、tool result、permission request を表示できる
- [ ] output extension view は composer と thread layout を押し潰さない
- [ ] `options: { debug: true }` の hover JSON と output extension view の表示責務が分離している
- [ ] `just harness-up floem` で output extension view の手動確認ができる
- [ ] `just harness-up egui` と `just harness-up gpui` で adapter 起動確認ができる
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 4. User Review

> ユーザーレビューの指摘は `[/]` で閉じる。通常 task の `[x]` と混ぜない。

- [ ] 4.1 実装結果と検証結果をユーザーへ提示する
- [ ] 4.2 ユーザーからのフィードバックをこの tasks.md に追記する
- [ ] 4.3 defer 指定がないフィードバックをすべて解消する

---

## 5. Final Verification

- [ ] 5.1 `just check` が通る
- [ ] 5.2 `just ast-lint` が通る
- [ ] 5.3 `npx -y @fission-ai/openspec validate "v0-5-0-output-extension-views"` が通る
- [ ] 5.4 `/openspec-verify-change` で Critical がない
- [ ] 5.5 PR 作成が必要な場合は `/create_pull_request` を使う
- [ ] 5.6 merge 後、必要なら `/openspec-archive-change` を使う
