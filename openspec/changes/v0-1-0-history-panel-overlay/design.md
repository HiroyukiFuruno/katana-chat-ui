## Context

`chat.frame.thread` はヘッダー下のメッセージをスクロールする。現在は履歴パネルも同じ ScrollArea の末尾へ追加される一方、`content_height` はメッセージ行だけから計算されるため、履歴が会話の長さとスクロール位置に依存して見えなくなる。履歴の表示と選択は既存の provider-free Storybook 操作要件に含まれる。

## Goals / Non-Goals

**Goals:**

- 履歴をヘッダー直下のスレッド領域に固定し、開いた時点で表示する。
- 履歴行を専用スクロール領域に置き、長い一覧をスレッド領域内で最後まで選択できるようにする。
- 履歴スクロール位置を会話スクロール位置から分離し、会話の位置を維持する。
- 実際の KUC renderer を使った可視 action-hit bounds、描画 pixel、wheel routing、選択 action の回帰を追加する。

**Non-Goals:**

- 履歴の検索、並べ替え、保存方式、セッション選択後の host 動作を変更しない。
- 公開 KUC/KCU API、framework adapter、品質 gate、画像 baseline を変更しない。

## Decisions

- 履歴パネルを `chat.frame.thread` の子から root-level absolute overlay へ移し、上端をヘッダー高、下端を composer 上端に合わせる。これにより、会話 thread の `tree_scroll_y` が変わっても履歴の表示位置は変わらない。
- 履歴行を overlay 内の KUC ScrollArea に置く。runtime は公開 renderer の action/node hit bounds から履歴の実コンテンツ高を測り、thread viewport に収まる表示高と完全なコンテンツ高を分けて設定する。文字幅や行高の推測値は導入しない。
- runtime は専用の private `history_scroll_y` を持つ。履歴 ScrollArea 内の wheel だけをこの値へ適用し、他の場所の wheel は既存の thread/composer routing を保つ。履歴を閉じるか session ID の一覧が変わった場合に offset を 0 へ戻し、同一一覧の resize では最大値に clamp する。
- overlay は thread の上、header/composer の下に描画する。履歴の selection action hit は overlay の見える領域に限られ、header と composer の hit bounds は既存位置に残す。
- 履歴 overlay を描画する最終 tree では conversation thread subtree を省略し、geometry と message row 高の measurement pass では thread を保持する別 tree を使う。これにより会話の scroll state と履歴の実測値を保持しながら、viewport 背景に背後の会話文字と action hit を残さない。private `hide_thread` はこの final/measurement tree の分離を選択する runtime build context であり、公開 renderer の visibility flag に依存しない。
- 履歴が開いている間も header と composer は同じ tree に残す。履歴行は title、provider/updated、preview を公開 KUC の wrapped `Text` child として描画し、selection action と完全な accessibility label は行コンテナに保持する。`empty_label` は session が空の場合だけ描画する。

## Risks / Trade-offs

- [Risk] KUC renderer の通常の node hit bounds に selection action が含まれない場合、全行の正確なコンテンツ高が取得できない → 同じ renderer の unbounded viewport-interaction hit query を使い、action と node bounds の最大値を測る。必要な bounds が欠ける場合は行を不可視のままにせず、明示的な runtime failure と回帰テストで検出する。
- [Risk] overlay の wheel が thread に漏れるとユーザーの会話位置が予期せず変わる → 履歴 viewport bounds を優先判定し、独立 offset と thread offset の両方を実 runtime test で比較する。

## Migration Plan

保存形式や公開 API の移行は不要。履歴 overlay と専用スクロール状態を同じ runtime 内で切り替える。

## Open Questions

なし。
