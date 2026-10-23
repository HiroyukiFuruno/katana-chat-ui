## Why

KUCの公開済み実装を利用してKCUのUI実装を共通化し、vendor別の重複を減らす。最新のACP仕様との境界を確認し、依存関係を更新する。

## What Changes

- UI構成・描画・操作の共通実装を公開レジストリのKUCへ依存させる。
- egui・Floem・GPUIのmulti-vendor提供と、LLM provider非依存の契約を維持する。
- 同じ公開KUC runtimeを実際に操作できるprovider-free Storybookを提供し、native hostや画面差分検証と役割を分ける。
- ACP公式仕様と現実装の差を調査し、実装済み機能と未提供機能を明示する。
- workspaceおよび検証hostの依存関係・lockfileを最新化し、既存ゲートで検証する。

## Capabilities

### New Capabilities

- `pure-kuc-chat`: KUCを共通UI実装とするmulti-vendorチャット提供。
- `acp-compatibility-boundary`: ACP公式仕様に対する現実装の対応範囲とwire境界。

### Modified Capabilities

なし。

## Impact

`crates/katana-chat-ui*`、検証host、依存manifestとlockfile、検証・仕様文書が対象。他リポジトリは読み取りのみ。既存の未コミット変更は維持する。最新のユーザー指示により、0001〜0017の要件達成、品質検証、commit・push、Draft PRからのレビュー、リリースと公開確認までを対象とする。手順と未完了事項は `openspec/release-tasks.md` を一次台帳とする。
