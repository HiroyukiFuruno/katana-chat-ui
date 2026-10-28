# KCU 0.2.2: repositoryの安全な検証・公開・資源管理

## Why

KCU 0.2.1の公開と公開版受入は完了したが、Issue #3のIssue起点hook、Issue #13の検証用Dockerイメージの証明書更新、Issue #14の固定名Docker volumeの管理は未解決である。現在の証明書ストアは拒否対象4ホストを受け入れ、現在のworkflowも固定名volumeを生成していることを2026-10-29に実測した。過去の記録や公開の成功だけで、これらの完了を主張できない。

## What Changes

- 既存品質検査を先行するdispatcherとIssue・依存更新の検証を追加する。
- 公開後だけ動作する、対象を明示したGit cleanupと保持条件を検証する。
- 正規のディストリビューション更新でDockerのTLS信頼ストアを更新し、現物で拒否・正常対照を検証する。
- 既知のKCU cacheだけを対象にpreview/removeを提供し、参照中・無関係な資源を保護する。
- 公開済みの最新互換依存を評価し、ACP 3.6.0とAWS-LCの更新候補を全7lock・既存の完全gateで検証する。GitHub Actionsは公式の現行refと公開最新版を照合する。

## Scope

このchangeは0.2.2のrepository保守に対応する。全Issue解決という上位目標は継続し、#7/#11の実際のIME表示修正は既存changeで追跡する。修正版の未公開path/git override、表示文字列の削除による回避、共有desktop操作、課金や追加LLMモデル導入は行わない。#7/#11を閉じずに上位目標を完了にしない。

## Acceptance

担当が実行できる自動検証と実公開の証拠を用いる。hooks・実Git fixture・実Docker fixture・既存の完全gate・review・Actions公開・公開後cleanupを回収する。証明書検査ではHTTP応答をTLS拒否とみなさず、通信失敗は未判定として扱う。無条件のforce削除や全体pruneを使用しない。remote削除では期待tipに条件を付け、検査後の更新を消さない。
