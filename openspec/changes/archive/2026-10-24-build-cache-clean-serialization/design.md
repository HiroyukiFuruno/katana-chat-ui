# Design

## 共有ロック

`tmp/maintenance/build-cache.lock` を KCU 全 target 共通のロックとして使用する。通常の `run` は blocking shared lock、`cleanup` は non-blocking exclusive lock を取得し、cleanup は lock を保持したまま open-file 検査と実コマンドを実行する。通常の Cargo 同士は並行実行し、cleanup 競合時は exit 3 とする。

## target境界

排他 lock 取得後、実際の Cargo prefix で7 manifestそれぞれに metadata を実行し、`CARGO_TARGET_DIR` と Cargo config を反映した全7 target_directoryを得て、全target専用cleanupを行う。既存 target には完全な Cargo `CACHEDIR.TAG` 所有署名を要求し、リポジトリ外、`.git`、source、symlink、非ディレクトリ、metadata失敗は fail-closed で拒否する。cleanupの `--manifest-path`、`--package`、`--profile`、`--target`、`--target-dir` など選択的cleanフィルターは非対応として拒否し、sweepのtail引数はmetadataへ渡さない。normal runのargvとCARGO opaque prefixは変更しない。

## 導線

Justfile の workspace build/test/lint/host/manual/storybook/screenshot 導線と screenshot runner を guard 経由にする。clean/sweep は cleanup mode を使い、通常 Cargo の引数、parallelism、終了コードを保持する。

### Git 境界検査
既存 target の ownership tag 検査後、checkout root を `git -C <root> rev-parse --show-toplevel` で照合する。`GIT_*` routing environment は subprocess へ渡さず、tracked files と non-ignored untracked files を target directory の literal pathspec で個別に検査する。Git の不在・非ゼロ終了・root 不一致は fail-closed とする。
