# Tasks: v0.2.0 — secure connector and account usage

> ACP 接続簡略化、ACP 非対応 provider の secret 管理、account / usage 表示を固定する。

## Branch Rule

本 change では、以下のブランチ運用を適用する。

- 統合ブランチ: `v0-2-0-secure-connector-and-usage`
- task ブランチ: `v0-2-0-secure-connector-and-usage-task<N>`
- feedback ブランチ: `v0-2-0-secure-connector-and-usage-feedback`

base branch を固定で決めず、PR 作成時に `/create_pull_request` で確認する。

---

## 0. ACP Connection Contract

### Definition of Ready

- [ ] v0.1.0 の chat UX foundation が完了している
- [ ] ACP stdio JSON-RPC、initialize、session/new、session config options の仕様を確認している
- [ ] `git status --short` で既存差分を確認している

### Definition of Done

- [ ] ACP agent command config と stdio transport を表現する型を定義している
- [ ] initialize request / response の protocol version と capability negotiation を実装している
- [ ] session/new の cwd、MCP servers、configOptions を扱っている
- [ ] model / thinking / permission は ACP session config options から UI に渡せる
- [ ] mock JSON-RPC test が initialize と session/new を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 1. Secret Store Boundary

### Definition of Ready

- [ ] Task 0 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] `SecretStore` / `SecretRef` / `SecretLease` を定義している
- [ ] secret は settings JSON に入れず、secret reference id だけを保存する
- [ ] direct connector は LLM request ごとに `SecretStore::acquire_secret` を呼ぶ
- [ ] `SecretLease` は drop 時に secret memory を破棄する
- [ ] OS credential store 優先、host-provided store、encrypted file store の順序を docs に明記している
- [ ] 平文 fallback を実装していない
- [ ] unit test が secret の保存拒否、request ごとの復号、drop 後破棄を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 2. Provider Settings, Usage, and MCP

### Definition of Ready

- [ ] Task 1 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] `ProviderConfigOption` が model / thinking / permission / endpoint を表現できる
- [ ] ACP session config options と direct connector settings を同じ UI model に写像している
- [ ] host が non-null settings JSON reference を渡す contract を定義している
- [ ] `SettingsMergeIntent` が provider connection、runtime、MCP server、usage 表示設定を typed settings patch として表現している
- [ ] settings 保存先未指定は invalid configuration になり、in-memory fallback をしない
- [ ] MCP server settings が server id、command または transport、env reference、enabled state、disabled reason を持つ
- [ ] provider usage が request token、response token、context usage、window/reset label、unavailable reason を持つ
- [ ] `AccountUsageSnapshot` が auth method、account、organization、plan、quota rows、reset、external URL を持つ
- [ ] usage が取得できない provider は `Unavailable(reason)` を返す
- [ ] UI は添付画像のような account / usage view を構成できる render model を持つ
- [ ] unit test が settings JSON merge、secret 混入拒否、MCP server validation、available / unavailable / partial usage を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 3. User Review

> ユーザーレビューの指摘は `[/]` で閉じる。通常 task の `[x]` と混ぜない。

- [ ] 3.1 実装結果と検証結果をユーザーへ提示する
- [ ] 3.2 ユーザーからのフィードバックをこの tasks.md に追記する
- [ ] 3.3 defer 指定がないフィードバックをすべて解消する

---

## 4. Final Verification

- [ ] 4.1 `just check` が通る
- [ ] 4.2 `just ast-lint` が通る
- [ ] 4.3 `npx -y @fission-ai/openspec validate "v0-2-0-secure-connector-and-usage"` が通る
- [ ] 4.4 `/openspec-verify-change` で Critical がない
- [ ] 4.5 PR 作成が必要な場合は `/create_pull_request` を使う
- [ ] 4.6 merge 後、必要なら `/openspec-archive-change` を使う
