## 最新ユーザー指示

KCUを更新する。純KUC依存で開発し、提供はmulti-vendor。ACP仕様の影響を確認し、依存関係も最新化する。既存 `openspec/v0-1-0-scope.md` の0001〜0017を満たしてリリースまで行う。最新の要件別証拠と公開手順は `openspec/release-tasks.md` を一次台帳とする。

## Branch Rule

- 現在の作業ブランチ: `release/v0.1.0`
- リリース対象: v0.1.0（change名は管理識別子であり公開versionではない）
- taskブランチ: `<change-name>-task<N>`
- feedbackブランチ: `<change-name>-fixes`
- 本turnは未コミット差分として検証し、既存の`.serena/project.yml`と`.DS_Store`を変更しない。

## 1. 調査と設計

- [x] 現状の構成・既存検証入口を読む
- [x] 公開KUC最新版を確認する（crates.io 0.3.7）
- [x] KUCの公開APIとmulti-vendorの移行可否を確認する
- [x] ACP公式仕様と現実装の差をソース付きで記録する
- [x] proposal/design/specを整備しvalidateする

### Definition of Done

- [x] 実行可能な移行設計と上流不足を区別できている
- [x] 要件を縮小せず未対応を記録している

## 2. 実装と依存更新

### Definition of Ready

- [x] 前taskの調査と設計が完了している
- [x] 書き込み範囲を分離している（依存更新はmanifest/lock、KUC実装はcore/kuc、ACPはdocs/README）
- [x] 他者差分をgit statusで確認している

- [x] KUCのみを使用する共通UIを実装する（3 hostがKUC runtime/raster/action bridgeを使い、headlessとLinux native matrixで確認）
- [x] native接続前の共通UiTree変換・型付きaction mapperを追加する（core test 81件PASS、純KUC移行完了ではない）
- [x] egui/Floem/GPUI提供経路を共通UIへ接続する（3 manual host gate、E2E conformance、headless/native matrixで確認）
- [x] 入力・送信・キャンセル・履歴・設定・vendor選択・出力の既存契約を維持する（番号別のKUC/host回帰と実provider probeで確認。物理IME受入は0004、最終手動UI確認は0016に残す）
- [x] 最終の純KUC構成で全manifest/lockの最新互換版を再評価し、品質gateを維持して採用する（KUC0.3.7、`just update-safe`、`just outdated`、更新後の品質gateを確認）
- [x] ACP通信実装と影響文書を最終構成で検証する（Codex、Claude Code経由のlocal Ollama、Copilot Free、KatanAgent/Ollamaでinitialize/session/new/prompt/完了応答を実確認。Capability、permission、content変換の回帰は既存transport/host testで維持）
- [x] provider-free Storybookを追加する（`just storybook`のrelease native起動、4 provider fixture、入力/送信/履歴/model選択、`storybook-check`のentrypoint・5契約test・window smokeを確認。物理IMEと最終人間UI確認は別DoD）

### Definition of Done

- [x] 要件を満たす実装・artifactが存在する（0001は本changeをarchiveしてactiveの`v0-*`を残さないことで充足する。0004の物理macOS日本語IME受入は、現在未利用の初回リリースではユーザーが明示的に延期し、native adapter回帰を維持する。0016は実Storybook 5画面をagentが目視し、表示破綻なしを確認）
- [x] 回帰検証が追加されている
- [x] 最新純KUC差分の自己レビューと指摘修正を完了する（Claude fixtureの有償model表示とACP wire未提供の旧specを修正し、残存P0/P1なしをreview.mdへ記録）
- [x] lint/AST検査やcoverageを緩和していない

## 3. User Review

- [x] 実装結果と検証結果をユーザーへ提示する
- [x] ユーザーフィードバックをrelease-tasks.mdへ記録する（対応済みは`[/]`）
- [x] defer指定のないフィードバックを解消する（要件照合、OpenSpec確認、Storybook実起動・目視確認を反映。物理IME受入の延期はユーザー明示の判断として`release-tasks.md`に記録）

## 4. Final Verification

- [x] 最新lockfileで `just check` と `just VERSION=v0.1.0 release-check` を最終再実行する（2026-10-18に`rtk proxy env CARGO_INCREMENTAL=0 just VERSION=v0.1.0 release-check`が成功。fmt、strict clippy、workspace/native/E2E、manual Egui 7・Floem 53・GPUI 6、Storybook 6、5画面、coverage 81.85%（基準77%）、package/publish dry-run、Linux matrix、未公開crate確認を通過）
- [x] 現在差分で `just ast-lint` が通る（2026-09-07、19 tests PASS）
- [x] 最新純KUC構成で `just coverage` が既存77%基準を満たす（2026-09-07、line 82.86%）
- [x] OpenSpec validateが通る
- [x] `just storybook-check` がentrypoint、fixture契約、release native window smokeを確認する
- [x] OpenSpec照合でCriticalがない（`review.md`の最新差分レビューでP0/P1なし。0004の物理IMEはユーザー明示で初回リリース対象から延期し、0016は実artifact目視で確認）
- [x] 未検証や上流境界を完了扱いしていない
- [x] 最終差分のcommit前に結果を報告する（2026-10-18の最終release-check、strict OpenSpec validate、`git diff --check`、5画面のagent目視結果を台帳とユーザー向け進捗で報告）

## 移行着手時の検証・未対応記録（過去構成）

以下のPASS、最終構成、確認中などは移行着手時点の履歴であり、現在の完了判定には使わない。最新状態はrelease-tasks.mdに記録する。

### 独立レビュー指摘

- [x] P1: disabled vendor controlの子optionもKUC action plan上で無効にする
- [x] P1: messageとhandoffのoutput node IDを表示場所ごとに一意にする
- [x] P2: vendor key/valueの単純連結によるnode ID衝突を防ぐ
- [x] 以上の回帰テストを追加して最新差分で再検証する（core81件PASS、独立read-only再レビュー済み）

- 初回`just update`は一時的なディスク容量不足で失敗。空き容量回復を確認して再実行しexit 0。
- syn 3で`ItemImpl::trait_`が2要素に変わったためACP lint visitorを適合。既存AST検査17件PASS。
- 6manifestの`cargo metadata --locked --no-deps`、`cargo tree --locked --depth 0`はPASS。
- ACP/providerの既存test9件とsyn3向けACP lint4件PASS。
- 初回`just check`はnaga26診断feature不一致で失敗。naga26 termcolorをFloem側で明示して解消。
- 最新FloemはGPUI stableのtaffy固定と競合するため不採用。詳細は`docs/dependency-update.md`。
- Floem・eguiのf32推論警告は型を明示して解消し、既存のソース検査も型明示に追従した。egui 0.36専用のtexture delta/drop API互換修正は、同版の不採用に伴い取り消した。
- 最終構成でworkspace 212 test、3 native host、E2E、5画面のscreenshot matrixを含む`just check`がexit 0。egui 0.36は既存hash不一致のため不採用とし、0.34.3を採用した。ログは`tmp/kuc-update-check-final.log`。
- screenshot runnerは`CARGO_TARGET_DIR`指定時にも今回ビルドした実行ファイルを使うよう`cargo run --release --locked`へ修正し、修正後のmatrixを完了確認した。
- screenshot matrixは途中の失敗を最終成功で隠さないよう`justfile`で失敗時に即終了する。baseline hashは変更しない。
- egui 0.34.3でstandard-chat/empty-chatの既存hash一致を確認。続くbuildがディスク容量不足で停止したため、完了済みcoverage専用`target/llvm-cov-target`だけを`cargo clean --target-dir`で整理し5.7GiBを回復、全checkを再開した。coverage合格ログは保存済み。
- `just ast-lint`は新規回帰を含む19件PASS（`tmp/kuc-update-ast-lint-final.log`）。
- egui 0.34.3を含む最終構成の`just coverage`はline coverage 79.09%でexit 0（`tmp/kuc-update-coverage-final.log`）。`CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0`を使用し、対象・77%閾値は維持した。
- KUC 0.3.6の既製raster hostはTextSurface入力描画と未統合だが、公開Canvas/TextRendererでKCU内に共通rendererを実装することは可能。技術的な移行不能とは扱わない。共通tree変換だけでは純KUC移行完了としない。
- 共通rendererをKUCへ集約して実装・公開するか、KCU内で保守するかをユーザーへ確認中。先の「上流追加が必須」という説明は訂正済み。回答前に別repoを変更しない。
