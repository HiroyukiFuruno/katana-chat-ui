---
name: self-review
description: katana-chat-ui のコミット前自己レビュー。差分の設計、検証、破壊的変更、言語、lint すり抜けを確認するスキル。
---

# Self Review

コミット前に必ず実行する。
対象は原則として現在の diff だけ。ただし public API 変更は call site まで追う。

## 観点

### 設計

- public API に UI framework 型や vendor SDK 型が漏れていない。
- ACP client、chat state、UI 実装の責務が混ざっていない。
- `struct` と `impl` を中心に責務が分かれている。
- fallback は仕様化されたものだけ。
- 関数は小さく、深い nest を避けている。

### 検証

- バグ修正では、先に失敗する回帰テストがある。
- テストは実際の挙動を検証している。
- mock は単体テストに閉じている。
- 統合テストや E2E は実環境に近い経路を使っている。
- 固定待機や timeout 指定で不安定さを隠していない。

### lint すり抜け

- `allow`、exclude、検査条件の緩和を追加していない。
- `unwrap`、`expect`、`unwrap_or_default` を安易に使っていない。
- `todo!`、`unimplemented!`、`dbg!` が残っていない。
- 型の曖昧化で設計問題を隠していない。

### OpenSpec

- tasks.md の checkbox が実装状況と一致している。
- ユーザーフィードバックは `[/]` で閉じている。
- 仕様変更が必要になった場合、artifact に反映されている。

## 実行

差分を確認する。

```bash
git diff --stat
git diff
```

必要な検証を選ぶ。
最終確認では、次を両方通す。

```bash
just check
just ast-lint
```

静的検査（lint）または抽象構文木検査（AST lint）が失敗した場合は、`/lint-and-ast-lint` を使う。

## 出力

```markdown
# Self Review: <対象>

## 結論
PASS / FAIL

## 指摘
- <必要な修正>

## 検証
- <実行したコマンドと結果>
```
