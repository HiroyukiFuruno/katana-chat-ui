# Tasks: v0.3.0 — vendor adapter expansion

> provider を増やす前に、接続種別、support state、capability model を固定する。

## Branch Rule

本 change では、以下のブランチ運用を適用する。

- 統合ブランチ: `v0-3-0-vendor-adapter-expansion`
- task ブランチ: `v0-3-0-vendor-adapter-expansion-task<N>`
- feedback ブランチ: `v0-3-0-vendor-adapter-expansion-feedback`

base branch を固定で決めず、PR 作成時に `/create_pull_request` で確認する。

---

## 0. Vendor Classification

### Definition of Ready

- [ ] v0.2.0 の secure connector と account usage contract が完了している
- [ ] Claude Code / Codex / GitHub Copilot / Bedrock / Vertex AI の接続種別を design.md でレビュー済みである
- [ ] `git status --short` で既存差分を確認している

### Definition of Done

- [ ] `ProviderDescriptor` が support mode、connection kind、setup state、capabilities を持つ
- [ ] `local-runtime` / `openai-compatible-direct` / `cloud-direct` / `acp-agent` / `unsupported-direct` を分類している
- [ ] Claude Code、Codex、GitHub Copilot を direct provider として登録しない test がある
- [ ] unsupported reason が UI model に表示できる
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 1. Ollama Local Runtime

### Definition of Ready

- [ ] Task 0 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] Ollama は agent provider selector に登録されない
- [ ] Ollama runtime が endpoint、availability、model list、selected model を扱う
- [ ] Ollama runtime は permission mode、file editing、command execution を capability として返さない
- [ ] usage / account usage が取得できない場合は `Unavailable(reason)` を返す
- [ ] local runtime として secret store を要求しない
- [ ] unit test が selector 非表示、endpoint normalize、model list、unavailable state を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 2. ACP Agent Adapter Hooks

### Definition of Ready

- [ ] Task 1 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] Claude Code adapter entry は ACP agent adapter として扱う
- [ ] Codex adapter entry は ACP agent adapter として扱う
- [ ] GitHub Copilot は adapter / extension surface がない場合 `Unsupported(AdapterMissing)` になる
- [ ] ACP agent adapter は auth を agent login flow に委譲し、kcu secret store に product token を保存しない
- [ ] config options を model / thinking / permission UI に写像できる
- [ ] mock adapter test が registration と unsupported state を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 3. Cloud Direct Adapter Contracts

### Definition of Ready

- [ ] Task 2 の実装、自己レビュー、検証、報告が完了している
- [ ] base branch が最新で、今回の書き込み範囲が明確になっている
- [ ] 他者の差分と衝突しないことを `git status --short` で確認している

### Definition of Done

- [ ] Anthropic API、Vertex AI、Bedrock は cloud-direct として分類している
- [ ] secret value ではなく `SecretRef` だけを settings に保存する
- [ ] OpenAI-compatible endpoint は endpoint、model、secret ref、streaming support を持つ
- [ ] Ollama 経由 cloud router は compatible endpoint がある場合だけ openai-compatible-direct として扱う
- [ ] unit test が secret value の混入を拒否している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 4. Agent Capability Catalog and Slash Entries

### Definition of Ready

- [ ] Task 3 の実装、自己レビュー、検証、報告が完了している
- [ ] v0.1.0 の slash launcher contract と v0.2.0 の settings / MCP contract が完了している
- [ ] provider / adapter / host のどこが entry source になるか design.md に明記している

### Definition of Done

- [ ] `AgentCapabilityCatalog` が prompt、skill、workflow、command、hook、MCP を kind 付き entry として扱う
- [ ] entry は id、display label、source、enabled state、disabled reason、required capability を持つ
- [ ] slash launcher は launchable な prompt、skill、workflow、command だけを候補として返す
- [ ] hook と MCP は adapter が launchable と返した場合だけ slash launcher に出る
- [ ] 選択結果は raw text ではなく `CommandLaunchIntent` として provider / adapter / host へ渡る
- [ ] 未対応 entry は実行可能に見せず、disabled reason を持つ
- [ ] unit test が catalog generation、disabled entry、slash candidate filtering、intent generation を検証している
- [ ] `/self-review` を実行し、指摘を解消している
- [ ] `/lint-and-ast-lint` の方針に従い、必要な検証を通している
- [ ] ユーザーへ結果を報告し、コミット前に停止している

---

## 5. User Review

> ユーザーレビューの指摘は `[/]` で閉じる。通常 task の `[x]` と混ぜない。

- [ ] 5.1 実装結果と検証結果をユーザーへ提示する
- [ ] 5.2 ユーザーからのフィードバックをこの tasks.md に追記する
- [ ] 5.3 defer 指定がないフィードバックをすべて解消する

---

## 6. Final Verification

- [ ] 6.1 `just check` が通る
- [ ] 6.2 `just ast-lint` が通る
- [ ] 6.3 `npx -y @fission-ai/openspec validate "v0-3-0-vendor-adapter-expansion"` が通る
- [ ] 6.4 `/openspec-verify-change` で Critical がない
- [ ] 6.5 PR 作成が必要な場合は `/create_pull_request` を使う
- [ ] 6.6 merge 後、必要なら `/openspec-archive-change` を使う
