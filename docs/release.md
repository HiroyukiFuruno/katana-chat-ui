# リリース手順

## 方針

`release/vX.Y.Z` ブランチから `main` または `master` へ取り込み依頼（Pull Request）を作る。
その取り込み依頼（Pull Request）では通常の品質ゲート（quality gate）とリリース前検査を必須にする。
取り込み（merge）後は自動実行基盤（GitHub Actions）がタグ（tag）、GitHub リリース（GitHub Release）、crates.io 公開を実行する。

## 必須検査

GitHub のブランチ保護（branch protection）では、少なくとも次を必須検査（required check）にする。

- `lint`
- `ast-lint`
- `test`
- `coverage`
- `release-preflight`

## リリース前検査

`release-preflight` は `release/v...` ブランチの取り込み依頼（Pull Request）で `just release-check` を実行する。
内容は次の通り。

- 整形確認（format）、静的検査（lint）、単体テスト（unit test）、抽象構文木検査（AST lint）
- カバレッジ（coverage）。現状の下限は行カバレッジ（line coverage）77%
- `Cargo.toml` の版番号（version）とブランチ版番号（branch version）の一致
- 作業領域（workspace）内部依存の版番号（version）一致
- 対象版番号（version）が crates.io に未公開であること
- `katana-acp-client` の梱包（package）と公開の事前実行（publish dry-run）
- `katana-chat-ui` と `katana-chat-ui-floem` の梱包（package）収録対象確認

`katana-chat-ui` は `katana-acp-client` を先に公開しないと crates.io 上で依存解決できない。
`katana-chat-ui-floem` は `katana-chat-ui` を先に公開しないと crates.io 上で依存解決できない。
そのため取り込み依頼（Pull Request）時点では `katana-acp-client` を事前実行（dry-run）し、後続 crate は収録対象確認までに留める。

## 公開順序

取り込み（merge）後の `Release` ワークフロー（workflow）は次の順で動く。

1. `just release-verify`
2. リリースタグ（release tag）作成
3. GitHub リリース（GitHub Release）作成
4. `katana-acp-client` を crates.io に公開
5. crates.io で `katana-acp-client` が見えるまで待機
6. `katana-chat-ui` を crates.io に公開
7. crates.io で `katana-chat-ui` が見えるまで待機
8. `katana-chat-ui-floem` を crates.io に公開

## 必要な秘匿値

自動実行基盤（GitHub Actions）には `CARGO_REGISTRY_TOKEN` が必要。
値は crates.io の API トークン（API token）を使う。

ユーザーが実行する登録コマンド:

```bash
cd /Users/hiroyuki_furuno/works/private/katana-chat-ui
gh secret set CARGO_REGISTRY_TOKEN
```

`Cargo` は crates.io のトークン（token）を `CARGO_REGISTRY_TOKEN` 環境変数で受け取れる。
トークン（token）は秘匿値として扱い、リポジトリ（repository）に保存しない。

参考:

- https://doc.rust-lang.org/cargo/reference/config.html#credentials
- https://doc.rust-lang.org/cargo/commands/cargo-publish.html
