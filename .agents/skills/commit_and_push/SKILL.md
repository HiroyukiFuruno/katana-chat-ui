---
name: commit_and_push
description: katana-chat-ui の commit と push を行うスキル。明示承認、関心事分離、日本語 commit message、通常 push の品質 gate を守る。
---

# Commit and Push

ユーザーが明示した場合だけ実行する。
編集後すぐに commit せず、実装と自己検証の報告後に承認を待つ。

## 手順

1. 状態を確認する。

```bash
git status --short
git diff --stat
```

2. 差分を関心事ごとに分ける。
3. `/self-review` を実行する。
4. 必要な検証を通す。最終確認では原則として次を使う。

```bash
just check
just ast-lint
```

5. stage は対象ファイルだけにする。

```bash
git add <files>
```

6. commit message は日本語で書く。

```text
feat: chat state の初期化処理を追加
fix: Ollama 応答の status 確認を追加
docs: OpenSpec tasks を更新
```

7. push は通常の `git push` を使う。

```bash
git push
```

## 禁止

- ユーザー承認なしの commit。
- 他者の差分をまとめて stage すること。
- `git push --no-verify` で品質 gate を飛ばすこと。
- lint や test の失敗を「関係なさそう」で無視すること。
- 複数の関心事を 1 commit に混ぜること。

## 失敗時

検証が失敗したら commit しない。
失敗理由、根拠、修正方針を短く報告する。
