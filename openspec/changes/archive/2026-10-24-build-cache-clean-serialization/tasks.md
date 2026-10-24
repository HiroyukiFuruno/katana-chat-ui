# Tasks

- [x] 共有 lock と target 境界を持つ Cargo guard を追加する（通常Cargoはshared、cleanupはexclusive）。
- [x] Justfile と screenshot runner の Cargo 導線を guard 経由にする。
- [x] cleanup 競合、open file、unsafe target、symlink、引数と終了コードの実 OS 回帰を追加する。
- [x] 通常 Cargo 並行実行、metadata target解決、所有タグ、metadata失敗、prefix保持、cleanup子プロセスの終了コード伝播を回帰へ追加する。
- [x] `check-macos` に guard regression を組み込む。
- [x] root review 後に full `just check` を再実行する（`tmp/verification/kuc044-full-just-check-libc189.log`、構造化exit0。guard実配置18回帰、既存5画像hash、Linux native15画像、Storybook実window smokeを含む）。
