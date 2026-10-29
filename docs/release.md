# リリース手順

## KCU 0.2.2の公開後受入状況（2026-10-29）

KCU 0.2.2は[GitHub Release](https://github.com/HiroyukiFuruno/katana-chat-ui/releases/tag/v0.2.2)として公開済み。公開元tag `e96f38b85b1eeaf516ddf08e5e2ca0a4516b4902` はmerge commit `6254232901b8a8aefee2f7f710bc59c41a9843b4`を指す。PR #16は2回のreview、required CI success後にmergeされ、merge後のmaster CIも6 job successだった。Release run [40750678393](https://github.com/HiroyukiFuruno/katana-chat-ui/actions/runs/40750678393)は公開とworkflow cleanupまでsuccessで終端した。証跡は`tmp/verification/all-open-issues-20261029/release022/`にある。

全5 crateのregistry API/archive/checksum/VCSとRust/Cargo source bindingは成功した。公開registryだけを使う独立consumerは4 binaryを8 phaseすべて実exit 0で構築・検証し、KUC 0.6.0、依存graph、lockとbinary sealを照合した（`public-preparation/v022-consumer/registry-evidence/collection-result.json`、`build-result.json`、`root-public-archive-identity.json`）。

公開appとprovider-free Storybookのdriverは実exit 0で、35 capture hash、15 baseline hash/寸法、3 host画像一致を検査した。rootは18原画を確認し、Storybookのfixture入力・送信・history操作を受け入れた。thread-end原画のprovider/title header帯にも欠落はない。Storybookの固定replyは実SDK通信の証拠ではない（`acceptance-prep/public-ui-v0.2.2/root-public-ui-accepted.json`）。公開Linux IMEの3 hostでは入力・確定1回・誤送信なし・取消状態解除・draft保持の機械契約が成功した一方、GPUI/Linux/X11 preedit重複（#7）とegui/Floem候補窓重なり（#11）は原画で再現し未解決である（`acceptance-prep/public-ime-v0.2.2/root-public-ime-accepted.json`）。

3 providerのresponse/cancel-resume計6シナリオは5件pass、Claude通常responseがtimeout・実exit 1で、受入全体は未完了である。Claudeのcancel/resume成功を全6シナリオ成功へ読み替えない。timeout原因も未確定であり、実証以上の外部要因・製品不具合を断定しない（`public-preparation/v022-consumer/provider-acceptance-final-20261029.json`、`root-provider-evidence-rehash.json`）。Issues #3/#13/#14は証拠コメント後にClose済みだが、#7/#11はOpenのまま。historical単色画像1件の原因も未確定で独立追跡する。これを全要件完了とは扱わない。

Codexの別v0.2.2 consumer-copy response/cancel-resume 2 scenarioは、公開sourceに束縛した検証consumerの誤期待をACP仕様に沿って訂正し、受入済み。元wrapperの失敗記録は保持し、Claude通常response timeoutとIME #7/#11は未解決のまま追跡する。詳細と証跡は[ACP受入記録](acp-compatibility.md)を参照。

公開後のscoped cleanupではlocal `release/v0.2.2`だけを削除し、remote refは保持した。この記録時点では文書作業branchが有効なためcleanup対象外であり、そのbranchの事後cleanup完了とは扱わない。別worktree `hooks/issue-3` の5つの未コミット差分はSHAを記録して現状保持し、統合・削除済みとは扱わない（`release022/root-scoped-release-hygiene.json`）。

## KCU 0.2.1の公開後状態（2026-10-28）

KCU 0.2.1の全5 crateは[GitHub Actions run 40643092212](https://github.com/HiroyukiFuruno/katana-chat-ui/actions/runs/40643092212)から公開済みで、[GitHub Release v0.2.1](https://github.com/HiroyukiFuruno/katana-chat-ui/releases/tag/v0.2.1)も作成済み。実公開元tag/merge SHAは`0c77dde206c32fbb4e34fd21a4459ed3c517c1ba`。各crateのregistry API/checksum/VCS、204 Rust source files、5つの`Cargo.toml.orig`との一致は`tmp/verification/kcu021-first-release-20261028/public-preparation/registry-public021-attempt2/root-publication-accepted.json`で確認した。

公開後consumerは4 binaryすべてを構築し、8 phaseすべて実exit 0、registry-only graph/no override、KUC 0.6.0、source/lock/binary sealを確認した。公開版UI・StorybookのDocker/driverは実exit 0で、原画・baseline・capture hashを照合した。Storybookはprovider-freeであり、固定replyは実SDK通信を示さない。公開版Linux IMEも3 hostで確定1回・送信0・取消状態解除・draft保持を実測し、#7/#11の表示失敗だけを承認どおり後続化した。詳細な証跡と条件付き完了範囲は[リリース要件監査](release-requirements-audit.md)に記録する。

## 方針

`release/vX.Y.Z` ブランチから `main` または `master` へ取り込み依頼（Pull Request）を作る。
その取り込み依頼（Pull Request）では通常の品質ゲート（quality gate）とリリース前検査を必須にする。
取り込み（merge）後は自動実行基盤（GitHub Actions）がタグで公開元コミットを固定し、全5 crateをcrates.ioに公開する。取得確認後にGitHub リリース（GitHub Release）を作成する。

## 必須検査

### 0.2.1の既知のLinux IME表示制限

2026-10-28のユーザー承認により、Linux/X11の[GPUI未確定文字の重複（#7）](https://github.com/HiroyukiFuruno/katana-chat-ui/issues/7)と、[egui/Floem候補窓の重なり（#11）](https://github.com/HiroyukiFuruno/katana-chat-ui/issues/11)だけを後続版へ回す。この2種類の互換修正版公開は0.2.1の公開停止条件から分離し、両Issueは未解決のまま追跡する。公開説明にも既知の制限として記載し、解消・視覚合格とは報告しない。

実IMEの入力・変換・確定1回・誤送信なし・取消後の状態とdraft維持は検査する。入力欠落、二重確定、取消不良、他の描画不良、新しい失敗には、この後続化を適用しない。既存の品質・coverage・画像基準、最終HEADのレビューとCI、正規registry依存、Actions公開、公開版appと対話Storybookのagentによるbackground確認、公開後整理は維持する。過去の単色画像1件は従前の独立追跡を維持する。

### 通常の必須検査

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
