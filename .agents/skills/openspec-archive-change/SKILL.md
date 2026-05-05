---
name: openspec-archive-change
description: katana-chat-ui の完了済み OpenSpec change を検証し、archive に移すスキル。
---

# OpenSpec Archive Change

完了済み change を archive に移す。
未完了 task や未検証のまま archive しない。

## CLI 入口

リポジトリルートから `npx -y @fission-ai/openspec` を使う。

## 前提確認

1. change 名を確定する。
2. `git status --short` で無関係な差分を確認する。
3. tasks.md に未完了の `- [ ]` がないことを確認する。
4. `/openspec-verify-change` を実行し、Critical がないことを確認する。
5. `openspec validate "<change-name>"` を実行する。

## archive 手順

```bash
mkdir -p openspec/changes/archive
```

archive 先は `YYYY-MM-DD-<change-name>` とする。
同名ディレクトリがある場合は止める。

```bash
mv openspec/changes/<change-name> openspec/changes/archive/YYYY-MM-DD-<change-name>
```

## 停止条件

- tasks.md に未完了がある。
- validate が失敗している。
- 仕様と実装の不一致が残っている。
- archive を現在の PR に含めるのか、merge 後に行うのかが不明。

## 報告

- archive 先
- 実行した検証
- 残した注意点
- コミットが必要なら、ユーザー承認待ちで止まる
