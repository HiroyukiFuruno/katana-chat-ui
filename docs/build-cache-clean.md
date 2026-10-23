# Cargoビルドキャッシュの直列化

KCU の Cargo 実行は `scripts/maintenance/cargo-guard` を通して共有ロックを取得する。通常の `run` は共有ロックを保持したまま実コマンドを実行し、通常の Cargo 同士は並行実行できる。`clean` と `sweep` の `cleanup` は同じリポジトリ内の全 Cargo target を対象に非ブロッキング排他ロックを取得する。実行中の Cargo がある場合は cleanup を拒否する。

ロックは無視対象の `tmp/maintenance/build-cache.lock` に置く。cleanup は排他ロック取得後、実際の Cargo prefix で7 manifestそれぞれの `metadata --no-deps --offline --locked --format-version 1` を実行して全7 target_directoryを解決し、全target専用で処理する。既存 target は完全な Cargo `CACHEDIR.TAG` 所有署名を必須とし、`CARGO_TARGET_DIR` と Cargo config の指定もこの解決結果で検査する。cleanup の `--manifest-path`、`--package`、`--profile`、`--target`、`--target-dir` など選択的cleanフィルターは非対応として拒否し、metadata失敗も安全側で拒否する。リポジトリ外、ソース、`.git`、シンボリックリンク、所有署名のない target を拒否する。POSIX では各 target の open file を `lsof` で検査し、検査不能な場合も安全側で拒否する。

通常の Cargo コマンドの引数、終了コード、並列数は変更しない。回帰検査は `just build-cache-guard-check` で実行し、ロック競合、open file、unsafe target、引数と終了コードを実 OS の子プロセスで確認する。

cleanup 対象 target の Git tracked / non-ignored untracked 内容は拒否し、Git checkout root と routing environment を検証する。ignore 済みの default/shared/nested target 内に生成された Cargo.toml と src は受け入れる。

複数manifestを検査する重いgateでは、同じtargetを明示すると依存artifactの重複を抑えられる。今回の完全検査には次の設定を使用する。通常のCargo設定や利用者の環境変数の既定値は変更しない。

```sh
CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 rtk proxy just JOBS=2 check
```

リリース検査も同じ環境で`just JOBS=2 VERSION=v0.1.0 release-check`を使用する。Linux native検査は別のDocker volumeを使用し、このhost側targetとは共有しない。
