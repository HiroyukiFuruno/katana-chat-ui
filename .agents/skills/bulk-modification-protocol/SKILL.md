---
name: bulk-modification-protocol
description: katana-chat-ui で複数ファイルの一括置換や破壊的変更を行う前に使う安全手順スキル。
---

# Bulk Modification Protocol

複数ファイルの一括置換、削除、機械的変換を行う前に使う。
雑な一括変更は速くない。失う文脈を先に守る。

## 適用条件

- `sed`、`awk`、`find`、`xargs` などで複数ファイルを書き換える。
- 手製 script で repository を走査して書き換える。
- 抽象構文木（AST）ベースの大規模変換をかける。
- 複数ファイルに同時 patch を当てる。

## 手順

1. `git status --short` と `git diff --stat` を確認する。
2. 他者の差分、未整理差分、今回の差分を分ける。
3. 破壊的変更の前に checkpoint を作る必要がある場合は、ユーザー承認を得る。
4. 対象を責務単位の小さい batch に分ける。
5. 1 batch ずつ変更する。
6. batch ごとに `git diff` を読み、消した文脈に WHY が含まれていないか確認する。
7. 結果を報告し、次 batch や commit の前に止まる。

## 禁止

- 事前確認なしの一括置換。
- 差分確認なしで次 batch に進むこと。
- `git reset --hard`、`git checkout .`、`git clean -fd`。
- lint を通すためだけの exclude 追加。
- 消えた文脈を「不要そう」で済ませること。

## 報告

- 対象 batch
- 変更理由
- 巻き込み確認の結果
- 残した懸念
