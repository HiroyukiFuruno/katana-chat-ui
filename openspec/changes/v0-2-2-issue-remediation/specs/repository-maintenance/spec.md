## ADDED Requirements

### Requirement: Issueと依存更新の検証

repositoryは非default branchの新commitにIssue参照を要求しなければならない（MUST）。依存manifestまたはlockの変更時は対応scopeと実データに結び付く証跡を要求し、既存の完全品質検査を先に実行しなければならない（MUST）。

#### Scenario: 品質検査の失敗

- **WHEN** 既存品質検査が非0で終了する
- **THEN** dispatcherは失敗を伝播し、後段の成功を理由にpushを許可しない

### Requirement: 公開後の安全な資源管理

repositoryは公開確認後の対象限定Git cleanupと、KCU所有cacheの対象限定Docker管理を提供しなければならない（MUST）。dirty・未統合・使用中worktree・停止中を含むcontainer参照・無関係volumeを保持し、remote削除には明示選択を要求しなければならない（MUST）。

#### Scenario: 停止中containerがcacheを参照する

- **WHEN** 対象volumeを停止中containerが参照している
- **THEN** removeは対象を保持し、拒否理由を明示する

### Requirement: 標準TLS検証の維持

両検証用Dockerイメージは、標準のTLS検証を有効にしたまま、正常対照でHTTP 2xxを受け取り、指定された拒否4ホストを今回実測した発行者取得失敗（OpenSSL verify code 20）で拒否しなければならない（MUST）。期限切れ・ホスト名不一致・その他の証明書エラー、DNS失敗、timeout、HTTPエラーを対象CAの拒否成功に数えてはならない（MUST NOT）。

#### Scenario: 拒否対象がHTTP応答する

- **WHEN** 拒否対象とのTLS接続が成功してHTTP応答に到達する
- **THEN** 検査は不合格となり、更新済みや修正済みと主張しない
