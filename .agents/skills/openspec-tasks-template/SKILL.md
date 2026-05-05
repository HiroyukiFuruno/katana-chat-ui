---
name: openspec-tasks-template
description: katana-chat-ui の OpenSpec tasks.md を、branch rule、DoR、DoD、user review、final verification 付きで整えるスキル。
---

# OpenSpec Tasks Template

tasks.md を、別の agent がそのまま実装できる粒度に整える。
低レベルな実装手順ではなく、守る contract、書き込み範囲、検証条件を明確にする。

## Change 名

`openspec/changes/` 配下は `vX-Y-Z-short-slug` 形式にする。

例:

```text
v0-3-0-vendor-adapter-expansion
```

## Branch Rule

tasks.md の先頭付近に入れる。

```markdown
## Branch Rule

本 change では、以下のブランチ運用を適用する。

- 統合ブランチ: `<change-name>`
- task ブランチ: `<change-name>-task<N>`
- feedback ブランチ: `<change-name>-feedback` または `<change-name>-fixes`

base branch を固定で決めず、PR 作成時に `/create_pull_request` で確認する。
```

## Definition of Ready

Task 2 以降の冒頭に入れる。

```markdown
### Definition of Ready

- [ ] 前 task の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している
```

## Definition of Done

各 task に入れる。

```markdown
### Definition of Done

- [ ] 要件を満たす実装または artifact 更新が完了している
- [ ] 回帰テストまたは対象検証が追加、更新されている
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している
```

## User Review

Final Verification の直前に置く。

```markdown
---

## x. User Review

> ユーザーレビューの指摘は `[/]` で閉じる。通常 task の `[x]` と混ぜない。

- [ ] x.1 実装結果と検証結果をユーザーへ提示する
- [ ] x.2 ユーザーからのフィードバックをこの tasks.md に追記する
- [ ] x.3 defer 指定がないフィードバックをすべて解消する
```

## Final Verification

tasks.md の末尾に置く。

```markdown
---

## x. Final Verification

- [ ] x.1 `just check` が通る
- [ ] x.2 `just ast-lint` が通る
- [ ] x.3 `openspec validate "<change-name>"` が通る
- [ ] x.4 `/openspec-verify-change` で Critical がない
- [ ] x.5 PR 作成が必要な場合は `/create_pull_request` を使う
- [ ] x.6 merge 後、必要なら `/openspec-archive-change` を使う
```

## 分割ルール

大きすぎる task は `2A`、`2B` のように分ける。
分割ごとに目的、依存関係、書き込み範囲、検証範囲を明記する。
