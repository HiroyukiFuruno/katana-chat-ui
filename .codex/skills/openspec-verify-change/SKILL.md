---
name: openspec-verify-change
description: katana-chat-ui の実装が OpenSpec change の proposal、design、spec、tasks と一致しているか検証するスキル。
---

# OpenSpec Verify Change

実装が change artifact と一致しているかを確認する。
archive 前、PR 前、または実装完了報告前に使う。

## CLI 入口

リポジトリルートから `npx -y @fission-ai/openspec` を使う。

## 手順

1. change 名を確定する。曖昧なら選択肢を出して質問する。
2. status を確認する。

```bash
openspec status --change "<change-name>" --json
```

3. apply instructions から `contextFiles` を取得する。

```bash
openspec instructions apply --change "<change-name>" --json
```

4. tasks.md の未完了項目を確認する。
5. spec の requirement と scenario が実装に反映されているか確認する。
6. design の境界決定が守られているか確認する。
7. `just check` と `just ast-lint` の必要性を判断し、最終検証では両方実行する。
8. `openspec validate "<change-name>"` を実行する。

## 判定軸

- 完了性: tasks.md が実装と一致している。
- 正しさ: requirement と scenario がコードやテストで確認できる。
- 一貫性: design の責務分離と public API 境界を壊していない。
- 検証の健全性: lint や test を緩めて通していない。

## 出力

重要度順に短く報告する。

- Critical: archive や PR 前に必ず直す。
- Warning: 直すべきだが、判断が必要。
- Note: 後続改善として記録する。

根拠はファイルパスと、可能なら行番号で示す。
