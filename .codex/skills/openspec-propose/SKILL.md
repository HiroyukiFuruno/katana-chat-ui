---
name: openspec-propose
description: katana-chat-ui で新しい仕様変更を作るスキル。proposal、design、spec、tasks を揃え、実装前に曖昧さを減らすときに使う。
---

# OpenSpec Propose

新しい change を作り、実装に必要な artifact を一式そろえる。
artifact は原則として日本語で書く。

## CLI 入口

リポジトリルートから次を使う。

```bash
npx -y @fission-ai/openspec
```

以下の例で `openspec` と書く箇所は、実際には上記の command に読み替える。

## 手順

1. 目的が曖昧なら、何を作るかを短く質問する。
2. change 名は `vX-Y-Z-short-slug` 形式にする。例: `v0-3-0-vendor-adapter-expansion`
3. scaffold を作る。

```bash
openspec new change "<change-name>"
```

4. artifact の状態を確認する。

```bash
openspec status --change "<change-name>" --json
```

5. `instructions` の template と rules に従い、必要 artifact を順に作る。

```bash
openspec instructions <artifact-id> --change "<change-name>" --json
```

6. `proposal.md`、`design.md`、`specs/*/spec.md`、`tasks.md` を、実装者が迷わない粒度にする。
7. 生成後に validate する。

```bash
openspec validate "<change-name>"
```

## 書く内容の基準

- kcu の public API 境界を明記する。
- UI framework、vendor SDK、host application への依存を分ける。
- tasks.md は `/openspec-tasks-template` の構造に合わせる。
- 実装方法を細かく固定しすぎず、守るべき contract は明確にする。

## 禁止

- 親リポジトリ固有の配布、翻訳、アイコン、画面 shell を前提にすること。
- template の見出しを省略すること。
- OpenSpec の validate をせずに完了扱いにすること。
