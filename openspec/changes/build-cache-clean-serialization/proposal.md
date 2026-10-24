# Proposal: Cargoビルドキャッシュcleanupの直列化

## 背景

KCU の全 workspace check 中に、別の cache cleanup が `scripts/screenshot/target` を削除し、Cargo の dep-info 書き込みが `No such file or directory` で失敗した。

## 目的

同一 KCU リポジトリの通常 Cargo 実行と cleanup を単一の共有ロックで直列化し、target の安全な所有範囲を明示する。

## 非目的

依存関係、public API、Cargo.lock、容量上限、Linux container 用の別 volume、source code の変更は対象外とする。

## 完了条件

通常実行、cleanup 競合、open file、unsafe/symlink/outside target、終了コードと引数を回帰検査し、通常の KCU check-macos に guard test を組み込む。
