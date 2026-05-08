# v0.1.0 自己レビュー

## 結論

v0.1.0 は現時点では all done ではない。

OpenSpec の `status` と `validate` は通っているが、`tasks.md` に未完了の final verification 項目が残っている。
OS file picker と drag and drop 添付の実機確認は v0.6.0 へ劣後した。
それ以外の残項目は v0.1.0 の release blocker として扱う。

## 確認済み

- `npx -y @fission-ai/openspec status --change "v0-1-0-chat-widget-floem" --json`
  - `isComplete: true`
- `npx -y @fission-ai/openspec validate "v0-1-0-chat-widget-floem"`
  - pass
- `just check`
  - pass
- `just manual-ui-check`
  - pass
- `just ast-lint`
  - pass
- `just release-verify`
  - pass

## 今回閉じた項目

- `options: { debug: true }` を crate 側の標準 option として扱う。
- 標準設定画面、slash launcher、context usage 表示、output extension interface。
- token-free mock agent harness による file / diff / tool output handoff。
- Zed 参照レビューを `tmp/zed-chat-ui-reference-review.md` に記録。

## 未完了

- `just update` は依存更新を伴うため未実行。
- `/openspec-verify-change` の Critical なし判定は、この self-review と OpenSpec CLI 確認で代替中だが、tasks.md 上の明示完了は最終確認後に行う。
- egui / Floem / GPUI の実画面を人間が触る最終確認。

## v0.6.0 へ劣後

- OS file picker と drag and drop の両方による添付実機確認。
- drag and drop で OS file、host logical resource、virtual attachment を区別し、host callback で解決する実機確認。

## 判断

v0.1.0 の機械 gate は通っている。
ただし all done 判定には、GUI の人間確認と、実 LLM を使わない範囲での編集系 handoff 確認結果の提示が必要である。
