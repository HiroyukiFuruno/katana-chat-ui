# 設計

## 責務

Issue gateはcommit/pushの変更範囲・参照・依存証跡を扱い、dispatcherは既存品質検査の実終了値と実行順序を扱う。Git cleanupは公開確認と対象指定に限定する。Docker volume管理は固定allowlistと全container参照に限定し、Git cleanupとは共有しない。TLS検査は製品のUIや通信処理を変更せず、検証イメージの標準TLSクライアントと標準CAストアを測定する。

## 安全条件

remote branchの削除は明示的な選択がある場合だけ行う。default、未統合、使用中worktree、dirtyの対象を消さない。Dockerでは停止中containerの参照も含め、削除直前に再検査し、forceなしのdaemon操作で競合時の拒否を伝播する。テスト用callbackや本番CLIの検査回避用環境変数を追加しない。

## 証明書

Root G3単独の除外ではAAA cross-sign経路が残ることを同じTLS probeで確認した。独自に広範囲のCAを除外する前に、Debian公式security updateと最低修正版の確認を用いる。最終採用はsource Dockerfileから再構築した両イメージで正常対照・拒否4ホスト・信頼ストア在庫を測定して判断する。診断用candidate bundleの結果を本番イメージの結果へ流用しない。

## 並行作業

rootは設定統合、証明書修正方針、#7/#11の正規公開依存の調査、レビューと公開を担当する。補助担当はhooks、Docker cache、証明書検査の重ならないファイルだけを扱う。既存issue-3 worktreeの5差分は、内容の採否と検証が確定するまで保持する。
