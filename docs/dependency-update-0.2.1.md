# 0.2.1 候補の依存更新

0.2.0 の未公開候補から patch version を 0.2.1 に上げる。公開済みの最新版は 0.1.0 であり、0.2.0 が公開済みという意味ではない。

## Rust 依存

- Comrak を 0.54.0 から 0.59.0 に更新する。0.56 で削除された `extension.tagfilter` の設定を取り除く。KCU は HTML の出力ではなく Markdown AST の解析を使い、raw HTML は既存どおりテキストとして扱う。既存の raw HTML、GFM、画像などの回帰テストを維持する。
- katana-ast-lint を 0.5.3、Floem manual host の Tokio 指定を 1.54.1 に更新する。7つの lockfile は通常の `just update-safe` で解決する。
- 2026-10-28に公開KUC 0.5.1へ更新した。公開archiveのchecksumを照合し、0.5.0とのRustソース1455件の一致、MSRV 1.95.0維持、eguiの最低指定0.36.2への更新を確認した。全7lockの差分はKUCのversion/checksumだけで、解決済みeguiなどは不変。更新後の完全release-checkは2026-10-28にexit0、行カバレッジ86.69%と既存画像基準を維持した。新HEADのreview/CIはPRの実結果で別途回収する。ACP 3.5.0、GPUI 0.2.2 は調査時点の最新公開版。未公開の修正案や別 package の更新を、修正済みの公開依存として扱わない。
- GPUI の既存 Linux 依存との互換性のため `libc =0.2.189` を維持する。公開された互換修正版の確認なしに pin を外さない。
- 新たな直接・推移依存の宣言 MSRV は workspace の Rust 1.95.0 以下。最終的な採用可否は既存の全品質ゲートで判断し、閾値や画像基準を緩和しない。

## GitHub Actions

2026-10-27 の公式公開情報を照合した。現在の moving ref は最新の互換リリースを参照しているため、参照方針を変更しない。

| 使用中の参照 | 確認した最新リリース |
| --- | --- |
| actions/checkout@v7 | v7.0.1 |
| actions/upload-artifact@v7 | v7.0.2 |
| Swatinem/rust-cache@v2 | v2.9.3 |
| taiki-e/install-action@just / @cargo-llvm-cov | v2.87.32 の tool-name tag |
| dtolnay/rust-toolchain@stable | Rust 1.99.1 |

moving ref の照合は、将来も同一の内容を指す保証ではない。公開時の Actions 実行結果は別途確認する。

macOSのホスト検査と公開前検査は、入れ子workspaceのビルド先も共通の`target`へ揃える。通常CIの実ログでKUCのコンパイル7回を確認したため、検証済みのローカル実行と同じ共有先・incremental無効・dev/testのdebug情報なしの設定を使う。Cargoの依存・featureごとの識別を維持し、検査コマンド、画像基準、coverage閾値、timeoutは変更しない。runnerを確保できずキャンセルされた事象はGitHub側の容量不足として別に記録し、この設定で解決したとは扱わない。

## 未完了の受入

0.2.1 の全品質検査、同じ source の Draft PR・2回のレビューと指摘対応、required CI、GitHub Actions での公開、公開 registry の独立 consumer、公開版アプリと Storybook の背景確認はそれぞれ別工程である。旧候補の合格をこれらの完了へ流用しない。

Linux 実 IME に必要な上流の修正済み公開依存と、公開版の受入は引き続き未完了。詳細な完了条件は `openspec/changes/kuc-0-5-compatibility/tasks.md` を維持する。

過去の単色画像1件の原因も未確定だが、2026-10-27の明示承認により、その元事象だけは公開停止条件から分離して[未解決の追跡対象](gpui-black-frame-followup.md)へ残す。診断追加や正常再取得を原因特定とは扱わず、今後の検査失敗や他の必須受入を免除しない。
