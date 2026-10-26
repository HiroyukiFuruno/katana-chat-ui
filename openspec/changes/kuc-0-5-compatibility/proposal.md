# KUC 0.5互換性と次期公開版の受入

## なぜ必要か

2026-10-26に公開されたKUC0.5.0は、hard-breakの文字境界、RTL/LTR終端、soft-wrapの論理行、text layout高さの処理を更新している。KCUの純KUC描画・入力・hit/measurement契約を、公開版を使って再確認する必要がある。GPUIの公開依存不具合待ちを理由に、この独立作業を止めない。

KCUのChatUiTree::treeは公開KUC UiTreeを返す。KUC0.4から0.5への依存更新は外部型のcrate identityが変わるため、5公開crateをKCU0.2.0として準備する。MSRV1.95は変えず、UI/vendor/protocol境界や既存の0001〜0017要件を変更しない。

## 変更内容

- workspace/e2eの公開KUC依存を0.5.0へ更新し、全7lockを最新互換版で再解決する。既知のGPUI libc固定は根拠なしに解除しない。
- ACP3.5.0は既にlockで選択済みであり、直接manifestの下限も実検証版へ揃える。ACPの標準wire型をvendor SDK漏れと誤認して公開APIを作り直さない。
- 純KUCの既存境界・Unicode入力/geometry・3renderer conformance、coverage、AST lint、画像・native・Storybook gateを維持して検証する。
- 利用可能な既存ローカルモデルと無償範囲で実provider通信を再確認し、登録/単体契約/実通信/画面/公開の証拠を分ける。
- 修正版の公開依存を取得後、公開consumer/app/Storybookを専用display/runnerで背景検証し、KCUをGitHub Actionsから公開する。

## 対象外と制約

有償契約・trial、ユーザーの共有desktop/input操作、未公開path/git overrideの最終採用、検査の緩和は行わない。元の単発black captureは原因未確定のまま追跡し、診断追加や正常再試行で解決扱いしない。scope変更の質問に返答がないことを承認と解釈しない。

## 完了条件

既存必須gateと既存coverage実績を維持し、実行した検査のexitとsource/package/binの対応を回収する。公開版のGPUI重複解消・3host IME契約と公開consumer/app/Storybook受入、同一最終HEADのreview/指摘対応/required CI/merge、Actions公開・registry/Release確認と安全な整理までをrelease完了条件とする。本人の共有画面での確認は要求しない。
