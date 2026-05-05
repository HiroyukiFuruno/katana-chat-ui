---
name: kcu-workflow-guide
description: katana-chat-ui の開発で、仕様管理（OpenSpec）、自己レビュー、静的検査（lint）、抽象構文木検査（AST lint）、コミット、PR 作成を組み合わせる入口スキル。
---

# kcu Workflow Guide

このスキルは、katana-chat-ui（kcu）で作業するときの入口です。
kcu は vendor 非依存の LLM chat UI と ACP client を提供する Rust library として扱い、親リポジトリの実装詳細を前提にしません。

## 基本方針

- 仕様が曖昧な作業は、先に `/openspec-propose` で仕様を作る。
- 既存の仕様変更を進める作業は、`/openspec-apply-change` で対象 change を確認してから進める。
- 実装後は `/self-review` と `/lint-and-ast-lint` を通し、必要なら `/openspec-verify-change` で仕様との一致を確認する。
- コミットと push は、ユーザーの明示指示があるときだけ `/commit_and_push` で行う。
- PR 作成は `/create_pull_request` で行い、base branch を推測で固定しない。

## kcu の境界

- public API に UI framework 型や vendor SDK 型を漏らさない。
- ACP client、chat state、UI 実装を混ぜない。
- vendor 固有差分は UI 分岐ではなく capability や adapter 境界で表す。
- React、TypeScript、WebView 前提の設計を持ち込まない。
- 親リポジトリ固有の配布、翻訳、アイコン、変更履歴、画面 shell 前提を持ち込まない。

## 作業の選び方

- 新規機能、仕様整理、複数 task 化が必要: `/openspec-propose`
- tasks.md に沿った実装: `/openspec-apply-change`
- 実装後の仕様照合: `/openspec-verify-change`
- 完了済み change の整理: `/openspec-archive-change`
- tasks.md の構造整理: `/openspec-tasks-template`
- 静的検査や AST 検査の失敗対応: `/lint-and-ast-lint`
- 大量置換や複数ファイルの破壊的変更: `/bulk-modification-protocol`

## 実装前チェック

1. `git status --short` で他者の変更を把握する。
2. `just --list --unsorted` または `Justfile` を読み、既存の入口を優先する。
3. バグ修正では先に回帰テストを置く。
4. 複数ファイルへ一括変更する場合は、変更前に `/bulk-modification-protocol` を使う。

## 完了前チェック

- `just check` と `just ast-lint` が必要な範囲で通っている。
- `allow`、exclude、検査条件の緩和で逃げていない。
- OpenSpec の tasks.md が実装状況と一致している。
- コミット前にユーザーへ結果を報告し、承認を待っている。
