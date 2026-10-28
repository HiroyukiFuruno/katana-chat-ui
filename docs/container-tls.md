# コンテナのTLS受入検査

`scripts/security/check-container-tls.py` は、KCU Issue #13に関係するCAストアと5ホストを検査する。`scripts/security/check-images.py` は、既存のローカルLinux/arm64 imageをimmutable IDで起動し、probeの結果と所有containerの消滅を確認する。imageやDockerfileのbuildは行わない。

wrapperは `shutil.which("rtk")` で実行環境を判定する。RTKがあればその絶対pathに `proxy docker` を続けて使い、なければ `docker` CLIを直接呼ぶ。CI runnerにRTKを追加インストールする必要はない。TLS検査条件と失敗判定はどちらの実行経路でも同じである。

## TLS受入条件

- 正常対照 `https://emudhra.we-have-a-problem.com/` は、標準TLS検証を通過して2xx HTTP statusを返す。4xx、5xx、redirectは合格にしない。
- G3、AAA、noCT、AAA + noCTの4ホストは、実測済みのOpenSSL code 20（local issuerを取得不能）で拒否する。各失敗の`verify_code`と`verify_message`を結果へ保存する。code 20以外は、チェーン信頼エラーに見える場合でも未判定とする。
- DNSエラー、timeout、接続拒否、その他の接続失敗は未判定とし、TLS拒否の合格へ読み替えない。HTTP応答は拒否対象の不合格とする。
- hostname mismatch、期限切れ、用途不一致など、許可したチェーン信頼拒否コード以外の証明書エラーは未判定とし、問題の信頼経路が拒否された証拠として扱わない。
- TLS検証を無効化しない。proxyとredirectを使わず、次の5ホスト以外へ接続しない。

## 信頼ストア検査

probeは `ssl.create_default_context()` が読み込んだCA一覧のsubject、issuer、DER SHA-256、SPKI SHA-256を保存する。標準contextのCA一覧hashと、OpenSSLが選んだnative cafileの実バイトhashを別々に記録する。cafileを読めない場合はhashを作らず未判定にする。

証明書データの一次出典は [eMudhraの影響中間証明書一覧](https://emudhra.we-have-a-problem.com/impacted-intermediates.json) である。16証明書に含まれる8つのunique SPKI SHA-256を照合し、2つのdistrusted trust-anchor DER fingerprint `86a1ecba…` と `d7a7a0fb…` も照合する。いずれかがtrust storeにあれば、5ホストすべてが通ってもaggregateを不合格にする。probeの結果JSONには出典URL、取得済みsnapshotのSHA-256、8つのSPKI fingerprint、照合結果を記録する。

## 実行方法

出力先は新規の `tmp/` ディレクトリにし、1 imageまたは2 image refを渡す。wrapperはimageをinspectし、ID、OS、architecture、RepoTags、RootFS、SSLストアoverrideの有無だけを証跡に残す。image環境変数全体は保存しない。inspectしたimmutable IDを `--pull=never` で起動する。invocationとfinal receiptにはprobe/wrapperと2つのDockerfileのSHA-256、Git routing環境を除去して取得した検査時checkoutのHEAD、dirty状態、status hashを記録する。これは検査時checkoutの識別情報であり、imageがそのDockerfileからbuildされたことを単独で証明しない。build logとimage IDを別途結び付ける。

```sh
rtk proxy python3 scripts/security/check-images.py \
  --output-dir tmp/verification/all-open-issues-20261029/issue13-source-tls/run-1 \
  kcu-screenshot-linux-arm64:issue13-ca-20261029-local \
  kcu-ime-linux-arm64:issue13-ca-20261029-local
```

各containerには一意なowner/run labelとfresh CID fileを付ける。wrapperはCIDが64桁hexであることを検査し、`docker ps --all --quiet --no-trunc --filter id=<CID>` がexit 0かつ空を返した場合だけ消滅を認める。containerが残る場合は、CID、immutable image ID、owner/run labelがすべて一致する自分のcontainerに限り、forceなしの通常 `docker stop` を行い、同じ完全CID照会で消滅するまで確認する。SIGTERMを受けた場合は稼働中のDocker CLIを止めてから、CID fileにある完全CIDを同じ条件でcleanupし、終了値を失敗としてreceiptへ残す。cleanup中のSIGTERMはcleanupを中断しない。daemon照会に失敗した場合は未判定であり、他containerには触れない。

containerは `--rm`、default network、read-only root filesystem、tmpfs `/tmp`、全capability drop、no-new-privilegesで起動する。唯一のbind mountは検査用Pythonファイルのread-only mountで、SSL環境変数overrideを渡さない。probe failureはcleanup成功で合格に変わらない。wrapper exit 0は、Docker実exit 0、probe exit 0、5ホストPASS、immutable image ID一致、CA inventory条件、所有CIDの消滅をすべて満たす場合だけ。

## Unit test

純粋な分類・受入判定は次で確認する。

```sh
rtk proxy python3 -B scripts/security/test_container_tls.py
```
