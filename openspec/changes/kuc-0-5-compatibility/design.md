# 移行設計

## 既存境界を維持する

katana-chat-uiのchat stateとKUC tree/runtime、ACP client、各vendor adapterは既存責務を保つ。3adapterは同じ公開KUC描画/入力契約を使い、KCU内で上流decoderの文字重複を削除する回避は加えない。ACP SessionUpdateは標準プロトコル境界の型であり、特定vendor SDKやUI framework型とは区別する。

## 依存と公開版

KUC0.5.0は公開registryのchecksumを確認して採用評価する。KCUが返すKUC型の依存identity変更に合わせ、公開5crateと内部version条件を0.2.0へ揃える。CLI/検証toolの独立versionは便乗更新しない。7構成のlockを更新し、単なるmetadata成功をbuild/画像/実入力の成功にしない。

既存GPUI0.2.2→zed-xim0.4.0-zed→ctext0.3.0の重複は別の公開依存課題である。公開ctext0.4.1にもASCII欠落が判明したため、その候補は撤回済み。Riey/xim-rs#55の採否/公開と正規GPUI maintenance公開線は外部待ちとして分離する。修正版の実registry取得前にこのchangeやrelease全体を完了にしない。

## 検証単位

1. 既存core/host/adapter/AST/Unicode・geometry契約と全manifest/lock。
2. 現sourceのbackground app/Storybook・原画と機械契約。最終公開版の証拠には代用しない。
3. 現環境で利用可能な無償/ローカルのprovider実接続、model、stream/終端・取消。実施できないvendor操作を全件合格に拡大しない。
4. 修正版の公開依存とKCU新公開版を使う独立consumer/app/Storybook・3host実IME、Actions公開とhygiene。

原画だけで正しさを判定せず、入力/状態・action・callback・freshnessの既存契約と、source/package/binary SHAを組み合わせる。失敗の元ログも保持し、古い別sourceや不一致packageの成功を最新候補へ流用しない。
