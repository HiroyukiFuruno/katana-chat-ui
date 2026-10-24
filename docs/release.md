# リリース手順

## 方針

`release/vX.Y.Z` ブランチから `main` または `master` へ取り込み依頼（Pull Request）を作る。
その取り込み依頼（Pull Request）では通常の品質ゲート（quality gate）とリリース前検査を必須にする。
取り込み（merge）後は自動実行基盤（GitHub Actions）がタグで公開元コミットを固定し、全5 crateをcrates.ioに公開する。取得確認後にGitHub リリース（GitHub Release）を作成する。

## 必須検査

GitHub のブランチ保護（branch protection）では、少なくとも次を必須検査（required check）にする。

- `lint`
- `ast-lint`
- `test`
- `coverage`
- `release-preflight`

## リリース前検査

Release workflow は最初に release source admission を実行する。通常の Pull Request は同一 repository のマージ済み `release/vX.Y.Z` だけを受け付け、merge commit SHA を checkout して版番号を一度だけ解決する。macOS/Linux の検査と公開 job は admission の成功結果と、その出力 SHA に依存する。

`workflow_dispatch` は部分公開からの復旧専用で、`refs/tags/vX.Y.Z` を ref に指定しなければ失敗する。入力版（省略時は `Cargo.toml`）と tag 名・manifest の版を一致させ、tag が checkout 済み HEAD を指すことを公開前に検査する。任意 branch、存在しない tag、別 commit の tag、版番号不一致は検査を開始せず失敗する。

`release-preflight` は `release/v...` ブランチの取り込み依頼（Pull Request）で、同じ merge commit を macOS と Linux/aarch64 の job に固定して検査する。macOS job は `just release-check-macos`、Linux/aarch64 job は `just release-verify-native-linux-aarch64` を実行し、集約 job は両方の成功を必須にする。ローカルの `just release-check` は両 platform gate と未公開検査を引き続き実行する。
内容は次の通り。

- 整形確認（format）、静的検査（lint）、単体テスト（unit test）、抽象構文木検査（AST lint）
- カバレッジ（coverage）。現状の下限は行カバレッジ（line coverage）77%
- `Cargo.toml` の版番号（version）とブランチ版番号（branch version）の一致
- 作業領域（workspace）内部依存の版番号（version）一致
- 対象版番号（version）が crates.io に未公開であること
- `katana-acp-client` の梱包（package）と公開の事前実行（publish dry-run）
- `katana-chat-ui` と egui / Floem / GPUI adapter の梱包（package）収録対象確認

`katana-chat-ui` は `katana-acp-client` を先に公開しないと crates.io 上で依存解決できない。
3つの UI adapter は `katana-chat-ui` を先に公開しないと crates.io 上で依存解決できない。
そのため取り込み依頼（Pull Request）時点では `katana-acp-client` を事前実行（dry-run）し、後続 crate は収録対象確認までに留める。

## 公開順序

取り込み（merge）後の `Release` ワークフロー（workflow）は次の順で動く。

1. macOS の `just release-verify-macos` と Linux/aarch64 の `just release-verify-native-linux-aarch64` を同じ commit で成功させる
2. リリースタグで公開元コミットを固定（既存タグは現在のコミットとの一致を必須にする）
3. `katana-acp-client` を crates.io に公開
4. crates.io で `katana-acp-client` が見えるまで待機
5. `katana-chat-ui` を crates.io に公開
6. crates.io で `katana-chat-ui` が見えるまで待機
7. `katana-chat-ui-egui` を crates.io に公開し、同版の可視性を確認
8. `katana-chat-ui-floem` を crates.io に公開し、同版の可視性を確認
9. `katana-chat-ui-gpui` を crates.io に公開し、同版の可視性を確認
10. GitHub リリース（GitHub Release）作成

公開対象の順序は `scripts/release/packages.sh` を一次情報とする。GitHub Release の作成だけでは完了とせず、全5 crate が公開レジストリから取得できることを確認する。
手動実行もcrate公開を省略しない。途中で失敗した場合は同じ版・同じソースで再実行し、既に公開済みのcrateを飛ばして残りを公開する。
部分公開後の手動再実行では作成済みのタグをrefに指定する。異なるコミットは公開前に拒否する。タグだけではリリース完了と扱わない。

## 必要な秘匿値

自動実行基盤（GitHub Actions）には repository 専用 Secret `KCU_CARGO_REGISTRY_TOKEN` が必要。
値は公開対象5 crateを許可する crates.io の API token を使い、publish-new/update権限を付ける。公開 step はこのSecretだけを参照し、共通Secretへのfallbackは行わない。

ユーザーが実行する登録コマンド:

```bash
cd /Users/hiroyuki_furuno/works/private/katana-chat-ui
gh secret set KCU_CARGO_REGISTRY_TOKEN
```

公開 step は専用SecretをCargo用の `CARGO_REGISTRY_TOKEN` 環境変数へ渡す。Secret名とCargoが受け取る環境変数名は区別する。
認証の移行・公開しない診断については [公開認証の契約](release-authentication.md) を参照する。
トークン（token）は秘匿値として扱い、リポジトリ（repository）に保存しない。

参考:

- https://doc.rust-lang.org/cargo/reference/config.html#credentials
- https://doc.rust-lang.org/cargo/commands/cargo-publish.html
