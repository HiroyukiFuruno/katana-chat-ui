---
name: openspec-apply-change
description: katana-chat-ui の OpenSpec change にある tasks.md を読み、task 単位で実装と検証を進めるスキル。
---

# OpenSpec Apply Change

既存 change の tasks.md に沿って、実装を進める。
一気に全部進めず、task 単位で実装、自己レビュー、検証、報告を区切る。

## CLI 入口

リポジトリルートから `npx -y @fission-ai/openspec` を使う。

## 手順

1. change 名が指定されていない場合は、active change を確認して選ぶ。

```bash
openspec list --json
openspec status --change "<change-name>" --json
```

2. apply instructions を取得する。

```bash
openspec instructions apply --change "<change-name>" --json
```

3. `contextFiles` に出る proposal、design、spec、tasks を読む。
4. 残 task、依存関係、書き込み範囲を確認する。
5. バグ修正を含む task では、先に失敗する回帰テストを置く。
6. 実装後に tasks.md の該当項目を `[x]` にする。ユーザーフィードバックは `[/]` で閉じる。
7. `/self-review` と `/lint-and-ast-lint` を通す。
8. 結果を報告して、コミット前に止まる。

## 実装ルール

- public API の変更は call site まで追う。
- UI framework と vendor adapter の責務を混ぜない。
- fallback は仕様化されている場合だけ使う。
- `allow`、exclude、検査条件の緩和で lint を通さない。
- 他者の差分を戻さない。

## 停止条件

- task の意図が不明。
- 実装中に design や spec の前提が崩れた。
- 書き込み範囲が他者の作業と衝突している。
- 検証が通らず、設計判断が必要。
