## Branch Rule

本 change は親タスクで許可された既存 `release/v0.1.0` 上で進める。共有差分を持つため、追加の branch / worktree は作成しない。

## 1. 履歴 overlay と独立スクロール

- [x] 1.1 公開 KUC runtime regression で History action、長い会話、複数履歴行の red を確認する。初回 red は `node_hits` に selection action が含まれる前提の誤りで無効とし、`action_hits` と描画 pixel を使う red を取り直した
- [x] 1.2 履歴一覧を thread の message ScrollArea から root-level overlay へ移し、表示領域を header と composer の間に制限する
- [x] 1.3 実際の KUC renderer の action/node hit bounds から履歴コンテンツ高を測り、専用の history scroll offset と content/viewport 高を構成する
- [x] 1.4 履歴内 wheel を専用 offset へ routing し、close/list change でリセット、resize で最大値へ clamp する
- [x] 1.5 履歴の全行が最後まで表示・選択可能であり、thread offset と header/composer bounds が保たれる green regression を確認する
- [x] 1.6 履歴 open の最終描画 tree から conversation thread subtree を省略し、measurement pass には thread を保持して thread scroll state を維持する
- [x] 1.7 empty label を空一覧に限定し、履歴 row の title/provider/updated/preview を wrapped KUC text child として表示する
- [x] 1.8 wrapped child の実測最大 bottom から row 高を測り、final SelectionList action bounds が title/meta/preview 全体を覆うことと preview 下段選択を確認する。初回 red は action 高 20px に対し title 高 40px だったため、final row に実測高を適用して解消した。content height は row 高の合計ではなく、既存KUC hit bounds全体のglobal max bottomを保持して末尾欠落を防ぐ。旧 SUM 実装へ一時復元した検証も content height 1320 対 unbounded hit extent 1342 の red となり、global max 復元後に同一テストが green になった

### Definition of Ready

- [x] 前提となる長い会話での履歴不可視が実 runtime regression で確認されている
- [x] 書き込み対象が KCU KUC frame/runtime、関連 regression、OpenSpec delta に限定されている
- [x] 親タスクの共有差分を保持し、別 branch/worktree を作成しない

### Definition of Done

- [x] Storybook 履歴が長い会話でも header 直下の viewport に表示される
- [x] 長い履歴一覧を専用 wheel routing で全行選択でき、thread offset が変わらない
- [x] regression は公開 KUC renderer による可視 hit bounds と実描画 pixel を検証する
- [x] `/self-review` と `/lint-and-ast-lint` 方針に従い、必要な focused 検証を通す

---

## 2. OpenSpec と検証

- [x] 2.1 provider-free Storybook requirement の履歴 overlay / scroll scenario delta を追加する
- [x] 2.2 formatter、focused runtime tests、strict package Clippy、AST lint、strict OpenSpec validation を実行する
- [x] 2.3 light/dark の履歴 viewport pixel 独立性と背後 message action 非表示の regression を追加する

### Definition of Ready

- [x] Task 1 の red proof と fix scope が記録されている
- [x] 新規要件が履歴 overlay と専用 scroll routing に限定されている

### Definition of Done

- [x] proposal、design、spec delta、tasks が実装と一致する
- [x] 実行した全検証ログを `tmp/verification/` に保存する
- [x] 未実行の full release gate を完了扱いしないことを確認する

---

## 3. User Review

- [x] 3.1 実装結果と検証結果をユーザーへ提示する
- [x] 3.2 ユーザー manual review waiver と、agent による最新 native PNG の実確認結果をこの tasks.md に追記する
- [x] 3.3 defer 指定がないフィードバックをすべて解消する

  - User review record: root の manual review waiver により追加承認待ちは不要。agent が `history-open.png` と `history-selected.png` を実画像で確認し、タイトル・provider/updated・preview の重複・重なりがなく、選択後に初期 fixture へ復元することを確認した。

---

## 4. Final Verification

- [x] 4.1 `just ast-lint` が通る
- [x] 4.2 `scripts/openspec validate v0-1-0-history-panel-overlay --strict` が通る
- [x] 4.3 requirement、implementation、regression、design、tasks の対応を照合し Critical がないことを確認する
- [x] 4.4 親タスクが指定した全 gate を通過してから change を archive する（2026-10-24: CLI archive実終了0、main spec同期、active directory消失を確認）。
