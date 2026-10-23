## Context

KCUは公開KUCのみを共通UI runtimeとして利用する。KUC 0.4.2のCard child indentは本文を既存位置へ描画できるが、同じstable nodeのviewport hit幅を544pxに縮める。KCU側に公開APIだけで描画とhitを分離する手段はなく、描画・測定・hit testを同時に直すにはKUC上流の変更と公開が必要である。さらにcode spanは `monospace = true` をmodel契約として保つ一方、既存基準の物理font faceは本文互換であるため、bodyとcodeのraster設定を独立できる公開renderer APIが必要である。v0.4.3では分割raster runをまたぐbidi選択のgeometryと段落区切りも公開挙動として固定する。

## Goals / Non-Goals

**Goals:**

- 公開KUCだけを用いて、wide/narrowのmessage幅、描画位置、hit test、code spanのmodel/raster契約を一貫させる。
- bidi本文と独立rasterのcode spanで、selection、link hit、copyの段落区切り契約をKUC v0.4.3の公開挙動として受け入れる。
- 既存スクリーンショット基準と品質ゲートを維持したまま、互換版KUCでKCUを再検証する。
- 上流修正のIssue、公開crate、KCU側の採用を追跡可能にする。

**Non-Goals:**

- KCUへpath/git依存やKUCのforkを導入しない。
- 画像基準・coverage・lint・AST検査の閾値を下げない。
- KUC上流の未公開修正をKCUリリースに含めない。

## Decisions

- KCUはCard child indentを16pxにし、本文stable nodeへ `hit_outset_left(16)` を設定する。描画・折返し・scroll測定を変えず、同一nodeのviewport hit幅だけを560pxへ戻せる公開契約である。
- KCUは本文rendererをSystem faceのまま維持し、`monospace = true` のcode spanだけへCandidateChainを用いた独立raster設定を指定する。draw、measurement、wrap、hitが同じcode rendererを共有する公開契約である。
- KCUの画像基準を更新して0.4.1のずれを受け入れない。依存更新の回帰検出として既存基準を維持する。
- KUC v0.4.3はKUC #85の `hit_outset_left`、#86の `with_code_text_raster_config`、#98の共通rounded raster geometry、#100のCR/LF/CRLF/U+2028/U+2029段落区切りを含む公開版だけを採用対象とする。描画・scroll測定・hit test・selection/copyが各契約を一貫して解釈することを必須とする。
- KCUは上流の公開crateを取得できるまでreleaseを外部待ちとして扱う。

## Risks / Trade-offs

- [上流APIが別設計を採用する] → KCUでpublic APIを再評価し、5画面基準と操作回帰を通った版だけを採用する。
- [公開まで時間がかかる] → KCUの依存更新・リリースは#85/#86/#98/#100を含むv0.4.3のregistry公開に依存するblocked状態として明示する。
- [描画だけを修正しhit testがずれる] → 描画、layout、hit test、native host/E2E、画像matrixを同じrelease gateで検証する。

## Migration Plan

1. KUC #85/#86/#98/#100を含むv0.4.3のGitHub Releaseとcrates.io公開を確認する。
2. KCUで公開registry版v0.4.3へ更新し、baselineを変更せずfocused layout、bidi copy/hit、画像matrixを検証する。
3. 完全release gate、Draft PR review、CI、公開、downstream確認を順に完了する。

## Open Questions

- KUC v0.4.3のregistry公開と全7lockへの採用は完了。実consumer検査で#104/#105が見つかったため、同一契約を満たすv0.4.4の公開、registry採用、全品質gateを残す。
