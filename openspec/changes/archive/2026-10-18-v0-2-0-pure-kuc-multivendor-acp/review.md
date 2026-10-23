# Self Review: pure-kuc-multivendor-acp

## 結論

PASS（実装レビュー）。3 host は公開 KUC 0.3.7 の共通 runtime / raster / action bridge を通り、Codex、Claude Code adapter、GitHub Copilot、KatanAgent の provider 境界を維持する。ACP v1 transport と direct Ollama は責務を分離し、内製 native widget 経路は標準 UI に含めない。

リリースの実行手順（OpenSpec archive、Draft PR、review、CI、公開 artifact）は別判定とする。0004 の物理 macOS 日本語 IME受入は現在未利用の初回リリースではユーザーが明示的に延期した。0016はagentが実Storybook 5画面を目視し、レイアウト、composer、出力カード、処理表示の破綻なしを確認した。

## 指摘

- 解消済み: disabled vendor controlの子optionが有効なaction planになる問題。子optionにもdisabledを適用し、KUCの`collect_from_tree`で回帰検証した。
- 解消済み: 同一outputを複数の表示場所へ配置すると node ID が衝突する問題。表示場所を含む ID とした。
- 解消済み: vendor key/value の単純連結による node ID 衝突。構造上の index を使い、payload には元の値を保持した。
- 解消済み: 長文 composer / thread の scroll が content extent を使わず、末尾を表示できない問題。公開 KUC の measured bounds で range を同期し、raster crop に offset を反映する回帰を追加した。
- 解消済み: Claude fixture が hosted 有償モデルを示す問題。provider-free Storybook は Claude / KatanAgent とも local `qwen3:4b` だけを表示する。

最新差分の読み取りレビューでは、残存する具体的な P0/P1 を検出しなかった。公開手順は自動検証で代替しない。物理IME受入の延期とagentの画面目視は、ユーザー判断と一次台帳に分けて記録する。

## 検証

- `just fmt`、`just lint`、`just unit-test`（185件）、`just ast-lint`（19件）、`just host-support-check`（75件）、`just host-e2e`（9件）、3 native host gate、headless screenshot matrix、Linux native matrix: PASS。
- `just coverage`: line coverage 82.86%、既存77%基準を維持。
- `just storybook-check`: entrypoint、6 fixture 契約、release native window smoke: PASS。
- 実 provider probe: Codex ACP、Claude Code adapter 経由の local Ollama `qwen3:4b`、Copilot Free ACP、KatanAgent direct Ollama の initialize / session / prompt / completion: PASS。
- `cargo package` と ACP client の `cargo publish --dry-run`、OpenSpec validate、`git diff --check`: PASS。
