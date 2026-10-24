# API-only 完了扱いの認識齟齬レポート

## 結論

認識齟齬の原因は、v0.1.0 の計画が「標準 UI を提供する」ことを完了条件として固定できておらず、`render model`、`reference surface`、`host maps model` という表現が API-only 実装を完了扱いできる余地を残していたこと。

## 起きたこと

- `katana-chat-ui` を UI 込みで提供する前提を、API と状態モデルの提供として解釈した。
- `manual-host-*` を標準 UI を載せる枠ではなく、host 側の検証画面として先に作った。
- 非画面系検証が pass したため、標準 UI が存在しない問題を検出できなかった。
- output は利用側へ返す data / action intent で十分なのに、GUI 上の操作部品として見せる方向へ寄せた。

## 直接原因

- OpenSpec の `katana-chat-ui-floem` が「reference surface」「view descriptors」と書かれており、実 widget 必須と読めなかった。
- tasks.md の DoD が `ChatRenderModel`、descriptor、smoke test に寄っていて、実 UI の表示確認が完了条件になっていなかった。
- manual host の DoD が「起動できる」中心で、標準 UI を載せるだけという制約がなかった。
- `just check` / `manual-ui-check` は compile と状態検証であり、「chat-ui として人間が触れる標準 UI」を検証していなかった。

## 再発防止

- OpenSpec に「API-only は customization path であり、v0.1.0 完了条件ではない」と明記した。
- `katana-chat-ui-floem` を標準 Floem UI crate として再定義した。
- SVG button override、i18n text catalog、vendor UI capability を v0.1.0 の明示要件へ戻した。
- manual host は独自 chat UI を実装せず、標準 UI を載せる枠と定義した。
- `kcu-linter` に OpenSpec と Floem crate の guard を追加し、API-only / descriptor-only への退行を検出する。

## まだ自動化できていない残り

- 実際の UI が人間にとって妥当かは、スクリーンショットまたは手動確認が必要。
- SVG override の runtime 反映、i18n、vendor capability の UI 表示は、実装後に追加の widget test / screenshot gate へ昇格する。
