# Tasks: v0.4.0 — conversation history and control

> session 履歴、送信済み message 編集、処理中の steering / interrupt / queue を、v0.1.0 の標準 UI 基礎から分離して実装する。

## Branch Rule

本 change では、以下のブランチ運用を適用する。

- 統合ブランチ: `v0-4-0-conversation-history-and-control`
- task ブランチ: `v0-4-0-conversation-history-and-control-task<N>`
- feedback ブランチ: `v0-4-0-conversation-history-and-control-feedback`

base branch を固定で決めず、PR 作成時に `/create_pull_request` で確認する。

---

## 0. Session History and List Contract

### Definition of Ready

- [ ] v0.1.0 の標準 UI foundation が完了している
- [ ] `docs/chat-ui-spec.ja.md` の session 履歴要件を確認している
- [ ] 書き込み対象が `session` / `message` / `surface` / `output` のどこかを明確にしている
- [ ] `git status --short` で既存差分を確認している

### Definition of Done

- [ ] `SessionHistoryStore` interface が save、list、load、delete を持つ
- [ ] `SessionSnapshot` が message、thinking reference、output reference、attachments metadata、provider / runtime selection、draft、scroll restore state を持つ
- [ ] `SessionHistoryListItem` が session id、title、provider、updated at、preview、status を持つ
- [ ] 標準 UI から履歴一覧を表示し、対象 session を選択できる
- [ ] UI layer が保存形式の file path / DB schema を知らない
- [ ] save / load の unit test が、message、draft、scroll restore、output reference の round trip を検証している
- [ ] history list の unit test が title、provider、updated at、preview、status の表示 model を検証している
- [ ] load で provider を再実行しない regression test がある
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 1. Sent Message Edit and Rewind

### Definition of Ready

- [ ] Task 0 の実装、自己レビュー、検証、報告が完了している
- [ ] edit 時に無効化する対象範囲を message / thinking / output 単位で design.md に追記している
- [ ] provider が rewind を持たない場合の UI 表示文言を text catalog に定義している

### Definition of Done

- [ ] 送信済み user message を編集開始、確定、取消できる
- [ ] 編集確定時に対象 message 以降の response、thinking、output が invalidated になる
- [ ] rewind 対応 provider だけ rewind intent を受け取る
- [ ] rewind 非対応 provider では disabled reason を返し、勝手な fallback 送信をしない
- [ ] edit cancel で元の message と後続状態が保持される unit test がある
- [ ] edit commit で後続状態が無効化される unit test がある
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 2. Steering / Interrupt / Queue

### Definition of Ready

- [ ] Task 1 の実装、自己レビュー、検証、報告が完了している
- [ ] provider capability に steering、interrupt、queue の support state がある
- [ ] `SteerMessage`、`InterruptAndSend`、`QueueMessage` の違いを spec に記載している

### Definition of Done

- [ ] 処理中の追加投稿が通常送信ではなく conversation intent として扱われる
- [ ] `SteerMessage` は provider が対応する場合だけ有効になる
- [ ] `InterruptAndSend` は進行中 generation を停止してから新しい generation を開始する
- [ ] `QueueMessage` は current generation 完了後に送信される
- [ ] provider 非対応 intent は disabled reason を返す
- [ ] generation id が一致しない chunk、thinking、output を破棄する unit test がある
- [ ] 処理中の追加投稿を 2 回行っても message order が壊れない state test がある
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 3. UI and Harness Verification

### Definition of Ready

- [ ] Task 2 の実装、自己レビュー、検証、報告が完了している
- [ ] egui / Floem / GPUI の adapter が v0.1.0 の標準 UI contract を満たしている
- [ ] manual harness の確認手順を docs に追記する場所が決まっている

### Definition of Done

- [ ] 履歴一覧、履歴復元、送信済み message 編集、処理中追加投稿を標準 UI から操作できる
- [ ] debug surface は `options: { debug: true }` の右端 output hover だけで表示される
- [ ] manual harness が独自の履歴 UI や debug UI を実装していない
- [ ] `just harness-up floem` で履歴復元、編集、割り込みの手動確認手順を実行できる
- [ ] `just harness-up egui` と `just harness-up gpui` で同じ標準 UI の起動確認ができる
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 4. Chat Search

### Definition of Ready

- [ ] Task 3 の実装、自己レビュー、検証、報告が完了している
- [ ] current chat search と history search の対象範囲が design.md に明記されている
- [ ] 検索結果から session / message へ移動する UI intent が定義されている

### Definition of Done

- [ ] current chat search が current session の message、thinking summary、output summary を検索できる
- [ ] history search が保存済み session の title、message、metadata、output summary を検索できる
- [ ] search result が session id、message id、matched range、preview、score を持つ
- [ ] user は検索結果から対象 session を復元し、該当 message へ移動できる
- [ ] search は provider を再実行せず保存済み index / snapshot を使う
- [ ] unit test が current chat search、history search、result navigation intent を検証している
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
- [ ] 6.3 `npx -y @fission-ai/openspec validate "v0-4-0-conversation-history-and-control"` が通る
- [ ] 6.4 `/openspec-verify-change` で Critical がない
- [ ] 6.5 PR 作成が必要な場合は `/create_pull_request` を使う
- [ ] 6.6 merge 後、必要なら `/openspec-archive-change` を使う
