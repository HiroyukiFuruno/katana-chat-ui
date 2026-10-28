# Dockerネイティブ検査キャッシュの扱い

KCUのLinuxネイティブ検査は、Dockerデーモン上の次の名前付きvolumeを使います。

| 名前 | 内容 | 現在の利用箇所 |
|---|---|---|
| `kcu-v010-linux-target` | CargoのtargetファイルとLinux IME driverが使うrelease host binary | `scripts/screenshot/linux/run-matrix.sh`、release workflowのnative-host build、`scripts/manual-ui/ime/run-background.py` |
| `kcu-v010-linux-registry` | Cargo registryのダウンロード済みデータ | `scripts/screenshot/linux/run-matrix.sh`、release workflowのnative-host build |

この2つの名前は現在のmasterのrelease workflowでも使われています。`v010`は古い命名であり、volumeが未使用という意味ではありません。同じホストのDockerデーモンを検査とrelease jobが共有するため、これらはcontainer単体ではなくデーモン全体の資源です。`docker run --rm`はcontainerを削除しますが、名前付きvolumeは削除しません。

## 保持方針

通常の検査では両volumeを保持し、Cargoがbuild出力と取得済みcrateを再利用できるようにします。検査処理はvolumeを自動削除しません。キャッシュ復旧または明示的なリセットが必要な場合に限り、名前を指定して削除します。このツールの対象外にある未知のvolume、他repositoryのvolume、停止中containerが参照するvolume、検証・証跡用volumeは保持します。

既存のvolumeには所有者labelがありません。そのため、ソース管理された固定2名のallowlistに完全一致した場合だけ、従来volumeとして認識します。想定外のlabel、driver、scope、mount option、container参照があれば保持します。今後のcache volumeには `io.katana-chat-ui.cache-owner=katana-chat-ui`、`io.katana-chat-ui.retention=cache`、用途labelを付け、allowlistを明示的に更新してください。名前のprefixだけでは所有者と判断しません。`io.katana-chat-ui.retention=evidence`が付いたvolumeは常に保持します。

Dockerの名前付きvolume inspectは不変のvolume IDを公開しません。preview JSONではDockerがIDを返さない場合に `volume_id: null` とし、名前、label、作成metadataとinspect metadataのSHA-256 fingerprintを記録します。このfingerprintは証跡用checksumであり、Docker volume IDではありません。

## previewと明示的な削除

repository rootから実行すると、既定ではDockerの状態を変更せずJSON previewを表示します。

```sh
rtk proxy python3 scripts/maintenance/docker-cache.py
```

previewには各allowlist volumeの有無、所有者根拠、inspect identity、label、実行中・停止中containerからの参照を記録します。volumeをinspectできない場合やDocker照会が失敗した場合は、未使用と判断せず非0で終了します。

削除するには、allowlist内の名前を完全一致で個別に指定します。

```sh
rtk proxy python3 scripts/maintenance/docker-cache.py \
  --remove kcu-v010-linux-target \
  --remove kcu-v010-linux-registry
```

各削除の直前に、所有者根拠と実行中・停止中containerからの参照を再確認します。参照があるvolumeは保持し、失敗として報告します。削除にはforceを付けない通常の `docker volume rm` を使います。最後の確認後にcontainerがvolumeをattachした場合は、Dockerの使用中拒否と終了値をそのまま返します。wildcard、任意のvolume名、force option、全体pruneは受け付けません。

## 回帰検査と再生成検査

実Docker safety fixtureは次のように実行します。

```sh
rtk proxy python3 scripts/maintenance/test-docker-cache.py
```

検査はローカルの `kcu-screenshot-linux-arm64:local` imageを完全な不変image IDへ解決し、そのIDを `--pull=never`、`--platform=linux/arm64`、`--network=none` とともに使います。作成するのは一意な実行IDを持つfixture volumeとcontainerだけで、各々に所有者・テストlabelを付けます。テスト用policyは内部関数にだけ渡され、本番CLIの固定allowlistを置き換えません。

fixtureは未参照cacheが削除候補になること、実行中・停止中containerからの参照が削除を止めること、用途違いとallowlist外volumeが保持されることを検証します。別の実Dockerケースでは、未参照preview後に停止containerをattachして通常のDocker削除を試し、daemonが使用中として拒否することを確認します。その後、削除したfixture volumeを再作成し、別containerから書いたcache artifactが読めることを確かめます。tool内部の削除直前再確認とこの別daemon拒否ケースは異なる検査であり、このfixtureはtool内部の二段目scanへraceを注入したとは主張しません。

証跡receiptには完全なcontainer ID、fixture volume名とlabel、commandの終了値、cleanup結果、検査出力、file hashを保存します。Docker command logにはcommandと時刻、実終了値、stdout/stderrの長さとSHA-256を記録し、inspect出力の本文は保存しません。signalまたは例外時のcleanupも、その実行IDに一致する名前とowner/test labelを確認したfixtureだけを対象とします。repository cacheや証跡用volumeは削除しません。

このfixtureはcleanup policyと一般的なvolume再利用を検証します。共有volumeを削除した後のcold-cache KCU受入検査の代わりにはなりません。その受入検査は使い捨てDocker環境または明示的に隔離した環境で実行し、まず `just harness-native-linux-matrix-check`、続いて現行release workflowのlocked native-host buildと実Linux IBus/Mozc受入を行います。build log、host binary hash、native matrix出力、IME commandの終了値、入力traceを保存してください。cache削除に合わせて品質条件を変更してはいけません。
