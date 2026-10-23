## Why
最終ACP差分reviewで、承認要求のtool_call内容がUIへ届かないP1と、model変更の成功応答に含まれる確定configを捨てるP2を検出した。v0.1.0公開前に既存の承認・model保持要件を満たす。

## What Changes
- 承認対象の提供済みtitle・操作内容・対象pathをvendor-neutralなUIへ渡して表示する。
- 長い承認内容も確認でき、既存の選択肢検証・キャンセル・modal境界を維持する。
- model変更応答のconfig_optionsを既存のconfig更新投影へ渡し、通知を別送しないagentでも確定modelを保存する。
- model変更がInvalidRequest・MethodNotFound・InvalidParamsで拒否された場合は理由を表示し、選択済みmodel・provider readiness・pending permission・inflight streamを維持する。通信断やInternalError等は従来どおりfail closedにする。
- ACP history復元では要求のenqueueとagentの確認応答を分け、確認値が要求modelと一致するまでproviderをMissingに保つ。拒否・不一致は保存済みmodelを維持してBlocked/Missingにし、理由を表示する。

## Impact
ACP transport、host projection、共通KUC permission surface、model config応答。課金・認証・自動承認や品質gateの緩和は追加しない。
