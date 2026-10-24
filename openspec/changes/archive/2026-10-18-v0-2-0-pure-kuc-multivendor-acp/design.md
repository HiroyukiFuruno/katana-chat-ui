## Context

KCU 0.1.0はframework非依存のChatUiSurfaceとegui/Floem/GPUI個別実装を持つ。公開KUC 0.3.6にはUiTree、描画、操作の共通契約がある。既存multi-vendorにはUI frameworkとLLM providerの両方が含まれ、今回の更新では両境界を維持する。

## Goals / Non-Goals

**Goals:** KUCをUIの実装基盤にする。公開版への依存で3vendor提供を維持する。ACP仕様の影響と依存更新を検証する。

**Non-Goals:** 別repo編集、既存品質基準の変更。最新指示によりリリースまでを対象とする。ACP通信も既存P0要件の達成に必要な範囲は対象とし、未実装を理由に要件から除外しない。

## Decisions

- チャット状態とACP/provider層をUI frameworkから独立させる。KUCのUiTreeを共通表示境界として使用する。
- native hostは表示とOSイベントの転送を担当し、チャットUIを独自再実装しない。既存UIから機能を落として移行済みとは判定しない。
- 純KUC依存はUI実装の依存境界を意味する。serde等の一般ライブラリやACP側の依存まで排除するものではない。
- ACPは内部モデルとwire schemaを分離する。公式SDKによるstdio transportと明示的なcontent変換を使用し、接続・capability・model・permission・作業状態を共有hostへ投影する。内部enumのserde表現変更で既存保存互換性を壊さない。実接続と全操作の検証範囲は `docs/acp-compatibility.md` とリリース台帳へ記録する。
- 依存更新は既存`just update`でmanifestとlockをそろえ、breaking更新はビルドと既存検証を満たす場合に採用する。
- Markdown見出しはKUC 0.3.6が認識する`heading`、`heading-2`、`heading-3`を使用する。H4-H6の表示はH3と共通とし、元の見出しlevelはnodeのstyle classに保持する。6段階の視覚的な同等性を実証したとは扱わない。

## Risks / Trade-offs

- KUCの既製raster hostはTextSurfaceの入力描画と未統合 → 公開Canvas/TextRendererによる共通renderer追加が必要。KCU内実装は可能であり、KUCへの集約は設計上の選択として区別する。他repoを無断修正しない。
- framework更新がnative hostに破壊変更を含む可能性 → 既存host検証を維持して修正または不採用理由を記録する。
- UiTree変換だけで完了を誤認する可能性 → 3vendorで実際に同じKUC描画・操作経路を通ることを完了条件に含める。

## Migration Plan

公開API調査、共通モデル・操作境界実装、vendor host接続、依存更新、全ゲートの順で検証する。既存未コミット差分は保全し、検証前に旧実装を一括削除しない。

## Open Questions

公開KUC部品を使用してKCU内に共通rendererを実装する。3vendorの実入力経路を検証し、`openspec/release-tasks.md`の既存P0要件とリリース条件を満たすまで継続する。
