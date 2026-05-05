---
name: create_pull_request
description: katana-chat-ui の Pull Request を作成するスキル。base branch を確認し、自己レビューと検証結果を PR に反映する。
---

# Create Pull Request

GitHub の Pull Request を作る。
base branch は固定せず、branch 名と作業文脈から確認する。

## 前提

- commit が完了している。
- push が完了している。
- `/self-review` と必要な検証が完了している。

## base branch の決め方

- `<change-name>-task<N>` は `<change-name>` を base にする。
- `<change-name>-feedback` または `<change-name>-fixes` は `<change-name>` を base にする。
- `<change-name>` 自体から作る PR は、repository の default branch を確認して base にする。
- 判断できない場合は、ユーザーに質問する。

確認例:

```bash
git branch --show-current
git branch -a
gh repo view --json defaultBranchRef
```

## 作成手順

1. PR template を確認する。

```bash
test -f .github/PULL_REQUEST_TEMPLATE.md
```

2. template がある場合は、それに沿って本文を作る。
3. template がない場合は、次の構造で本文を作る。

```markdown
<!-- 日本語でレビューしてください。 -->

## 概要

## 対応内容

## 影響範囲

## 動作確認結果
```

4. `--base` と `--head` を必ず指定する。

```bash
gh pr create --base "<base-branch>" --head "<current-branch>" --title "<title>" --body-file "<body-file>"
```

## 禁止

- `--base` 省略。
- default branch の推測固定。
- 検証結果なしの PR 作成。
- 他者の未整理差分を含めた PR 作成。
