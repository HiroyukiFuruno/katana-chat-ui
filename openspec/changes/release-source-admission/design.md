# Design

`release-admission` は、workflow eventに応じてcheckoutされたHEADとsource SHAを比較し、通常PRではmerged・同一repository・`release/vX.Y.Z`・merge commit SHAを検証する。手動復旧では `refs/tags/vX.Y.Z` だけを受け付け、入力版（省略時はmanifest版）を既存tag名と一致させ、tag commitがcheckout HEADを指すことを確認する。版番号の形式とmanifest一致は既存 `verify-version.sh` を一度だけ呼び出して確認する。

admissionのoutputはsource SHAとversionとし、macOS/Linux/aarch64検査はそのsource SHAを `fetch-depth: 0` でcheckoutする。公開jobはadmission、両OS検査の成功をneedsで要求し、公開前のtag検査と既存のcrate順序を保持する。GitHubの値はstepのenvへ渡し、shell内で値を直接展開しない。

回帰scriptはrepository内のignoredなtemporary Git repositoryを保持し、削除操作を行わない。good tag、input省略、任意branch、missing tag、tag commit不一致、version不一致、unmerged/wrong-branch PR、fork、wrong merge SHA、unknown eventを実際のguard processのexit codeで検証する。`GITHUB_OUTPUT` はfixture内へ隔離し、versionとsource SHAの出力値も確認する。

fixture検査は呼び出し元の全`GIT_*` routing/config環境変数を破棄し、Git init直後に`--show-toplevel`がfixture root自身であることを確認してからconfig/add/commitを行う。別のforeign Git repositoryをrouting先にした恒久Python回帰で、foreign HEAD/index/config/statusの不変と実Bash回帰のexit 0を検証する。旧版のforeign config変更redはignored sandboxで一回限りの証跡として保持し、恒久回帰はGit履歴に依存しない。
