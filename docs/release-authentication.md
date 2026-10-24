# Release authentication

## Publication route

crates.io への公開は GitHub Actions の `release.yml` に限定する。`Publish crates.io` step は `secrets.KCU_CARGO_REGISTRY_TOKEN` を `CARGO_REGISTRY_TOKEN` として `scripts/release/publish-crates.sh` に渡す。専用 Secret への切り替えに fallback は設けない。

KCU 1 repository の2つの GitHub Actions Secret には、token `242277`（`katana-chat-ui-actions-v0.1.0`）として作成した同一の専用 token を登録済みである。token の endpoint scope は publish-new/update、exact package allowlist は KCU 本体と ACP client、3 adapter の計5 package、expiry は 2027-01-22 である。token 値はファイルやログへ保存しない。

## Existing credentials and boundaries

既存の shared crates.io token `236436` は endpoint scope が5種で、exact crate filter は20件（`katana-chat-ui` を含む）だが、`katana-acp-client` と3 adapter は含まない。shared token の pattern は拡張も revoke も行わない。generic updater の source は `KATANA_SERIES_GENERIC_TOKEN` であり、KCU は既に対象に含まれる。

immutable な tag/source に対する既存の release workflow attempt 2 は `CARGO_REGISTRY_TOKEN` を参照したままなので、その tag/source は変更しない。新しい専用 Secret と workflow route の実効性は、同じ tag を付け替えずに行う後続 Actions 実行で確認する。

## Evidence boundary

official `auth.rs` による scope gate と、管理画面で観測した exact crate allowlist は確認済みである。ただし、旧 Actions Secret と管理画面の token row が暗号学的に同一だとは主張しない。

release workflow attempt 1 では、`katana-acp-client` の最初の upload が `403 Forbidden: this token does not have the required permissions` で失敗した。`workflow_dispatch` の `verify_registry_auth=true` は、通常の admission/release を全て skip して新旧 Secret 名を read-only で比較する診断経路である。比較は両方の値が非空で `hmac.compare_digest` が一致することだけを確認し、値・長さ・ハッシュを出力しない。この診断経路は fallback ではなく、公開も行わない。

新しい credentials の採用完了は、attempt 2 による5 package の公開・registry確認と、後続Actions診断による専用Secret名の非空・公開に使用した値との一致を組み合わせて判定する。画面上の Secret 登録状態や workflow 定義、診断 job の成功だけでは完了扱いにしない。
