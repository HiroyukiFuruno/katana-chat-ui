# Issue #20: ACP permission metadata観測

## DoR

- [x] Issue #20本文を読み、対象がACP permission request/responseの機密なし観測と既存検査への接続であることを確認した。
- [x] ACP 3.6.0 / schema 1.13.0を対象に固定し、timeout、marker、承認policy、商用コード、依存、描画を変更しない境界を確定した。
- [x] 元Claude runはWrite選択まで確認済みだがrequest/response wireがないため、承認待ちが原因とは断定できないことを確認した。別strict original binary runは元runの説明や置換に使わない。

## 実装・検証

- [x] 1. ACP 3.6.0 / schema 1.13.0のJSON-RPC実shapeを使い、受信した`session/request_permission`要求と反対方向の対応応答を同一入力ストリームで処理する。
- [x] 2. 実際に受信した要求だけに到着順ordinalを割り当て、要求・応答時刻と未回答件数を記録する。応答は一時的な同一JSON-RPC ID対応でordinalへ結び付け、raw IDを出力・保存しない。
- [x] 3. `tool_call`単独、通常prompt終端、cancel通知、無関係RPC、未知ID、duplicate/順不同/並行要求、不正JSON-RPC、result/error応答を回帰テストする。対応する同一ID応答だけが該当要求を解決する。
- [x] 4. ID/session ID/option ID、prompt/response本文、tool引数、path、header、auth値が出力・保存されないことをfixtureとunit testで検証する。
- [x] 5. importおよび解析経路がCLI/provider/process/networkを起動しないことを自動検査する。
- [x] 6. 旧解析器で確認済みの3つの不具合は修正前REDを保存し、新部品の回帰検査でそれぞれ再現を防ぐ。
- [x] 7. 既存manual UI/CI検査導線と本changeのOpenSpec、`docs/acp-compatibility.md`を更新する。歴史上の元runを成功・原因確定へ書き換えない。
- [ ] 8. 既存の完全品質検査、Draft PR、2回のreview、指摘対応/resolve、required CI、通常mergeを実際の同一sourceで回収する。

## DoD

- [ ] Issue #20のclose条件を満たす純粋なmetadata解析部品と自動回帰検査を公開・mergeする。
- [x] 解析は実受信のpermission requestと反対方向の対応応答だけをordinal相関し、`tool_call`から承認待ちを推定しない。
- [x] 180秒timeout、marker判定、承認policy、商用/API/依存、描画、全既存gateを変更・緩和していない。
- [x] #7/#11は未解決IssueとしてOpenを維持し、IME表示gateと上位goalは未完了のまま追跡する。
