# katana-chat-ui Rust コーディングルール

> [!IMPORTANT]
> このファイルを更新する場合は、英語版 `docs/coding-rules.md` も同時に更新する。

この文書は、`katana` の規約を `katana-chat-ui` 向けに移譲したものです。
`katana-chat-ui` は UI を含むライブラリなので、検証コードや手動確認用の小さなコードでも同じ品質基準を適用します。

## 1. 構造と責務

- 公開 API は `struct` と `impl` に寄せる。
- 公開された単独関数（public free function）は禁止する。
- private helper は許容するが、責務が増えたら型へ分離する。
- 1つの型、1つのファイル、1つの関数に複数の責務を混ぜない。

## 2. サイズ制約

- 1ファイルは 150 行を目安、200 行を上限にする。
- 1関数は 30 行を上限にする。
- 行数超過の解消は、行数だけで機械的に分けず、状態、表示、変換、検証などの責務で分ける。

## 3. ネストとエラーファースト

- ネストは最大 3 段、目標は 2 段までにする。
- 成功パスを `if let Ok(...)` で深く包まない。
- 失敗時は `?`、`let ... else`、早期 return で先に処理する。

## 4. 型安全

- `unwrap`、`expect`、`unwrap_or_default` は禁止する。
- `todo!`、`unimplemented!`、`dbg!` は禁止する。
- `Box<dyn std::any::Any>`、`HashMap<String, serde_json::Value>`、用途不明の `serde_json::Value` は使わない。
- 必ず存在する値を `Option` にしない。

## 5. コメント

- コメントは「なぜ（WHY）」を残すためだけに使う。
- 通常の `//` コメントは禁止する。
- 必要な注釈は `/* WHY: ... */`、`/// SAFETY: ...`、`/// TODO: ...` のように理由が先頭で分かる形にする。

## 6. マジックナンバー

- `0`、`1`、`2`、`100`、`-1` 以外の数値リテラルは名前付き定数へ出す。
- UI の幅、高さ、余白、しきい値も例外にしない。

## 7. プロセス起動

- `std::process::Command::new` を直接使わない。
- プロセス起動が必要になった場合は、専用の境界型を作ってそこへ集約する。

## 8. 依存関係更新入口

取り込み後に依存関係を最新化してから検証へ進む場合は、次の入口を使う。

```bash
just update
```

- `just update-safe` は `Cargo.toml` の SemVer 範囲内で `Cargo.lock` を更新する。
- `just update` は `cargo upgrade -i` で破壊的変更を含む依存関係更新を取り込み、その後 `cargo update` を実行する。
- workspace 外の検証 host も `--locked` で動くように、`tools/e2e-host-app` と `tools/manual-host-*` の lock も同じ入口で更新する。
- `cargo-upgrade` が使えない環境では、導入方法を表示して失敗させる。更新処理を黙って省略しない。

## 9. 検査入口

```bash
just fmt-check
just lint
just ast-lint
```

これらは「通すための作業」ではなく、設計の品質 gate として扱う。
検査を緩める変更は、ユーザーに相談してから行う。

## 10. lefthook

- `lefthook install` で local hook を有効化する。
- `pre-commit` は `just fmt-check`、`just lint`、`just ast-lint` を順番に実行する。
- `pre-push` と `pre-pr` は `just release-verify` を実行する。
- hook を迂回する `--no-verify` は使わない。
