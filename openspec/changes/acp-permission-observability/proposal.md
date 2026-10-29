# Why

公開KCU 0.2.2のClaude応答検証で、180秒timeoutとなった元runはWrite選択まで確認できたが、当時のpermission request/response wireが保存されていないため、承認待ちがtimeoutの直接原因だったとは断定できない。別の成功runを元runの説明へ流用せず、実際に受信したACP permission要求と対応応答だけを機密値なしで観測できる検証部品を追加する。

# What Changes

- ACP 3.6.0 / schema 1.13.0の実JSON-RPC構造から、agentからの`session/request_permission`要求と、それに対する反対方向・同一IDのJSON-RPC応答を観測する。
- 対応は内部で一時的に行い、外部へは受信順のordinal、観測時刻、未回答件数だけを記録する。要求IDやsession IDそのものは出力・保存しない。
- `tool_call`だけではpermission要求や承認待ちを推定しない。通常終端、取消、無関係なRPC応答もpermission要求の対応関係を作らない。
- 既存のmanual UI検証導線とCI検査から利用する純粋なmetadata解析部品を追加する。importや解析からCLI、provider、process、networkを起動しない。

# Scope

このchangeはIssue #20の観測部品、境界テスト、既存検査導線と説明の更新だけを対象にする。raw request ID、session ID、option ID、prompt/response本文、tool引数、path、header、auth値を出力へ含めない。

元の180秒timeout、marker判定、承認policy、商用コード、依存、UI描画は変更しない。既存の厳格な品質検査、Draft review、指摘対応、CI、通常mergeを維持する。#7/#11はOpenのまま追跡し、IME表示修正gateや上位の全Issue完了条件を変更しない。

# DoR

- Issue #20の本文と対象repositoryの現行ACP境界を確認し、ACP 3.6.0 / schema 1.13.0のwire shapeに限定する。
- 元Claude runで確認済みの事実と、保存されていないrequest/response因果を区別する。

# DoD

- 実wire shape、要求と反対方向の同一ID応答、ordinal、時刻、未回答数、順不同・並行・未知ID・不正入力を自動テストする。
- tool_callだけでは未回答permissionとして計上せず、JSON-RPC応答以外で要求を解決しない。
- 出力の機密値非包含とimport/解析時の外部process・network非起動を自動テストする。
- 既存の検査導線、OpenSpec、説明文書を更新し、Issue #20所定の品質検査、Draft review 2回、指摘対応、CI、通常mergeを完了する。
- 元runの因果はrequest/responseの実記録がない限り未確定のままとし、Issue #7/#11と上位目標は別の未完了事項として維持する。
