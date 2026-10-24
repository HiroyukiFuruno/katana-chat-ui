# ACP 互換性の確認記録

## 2026-10-25 の Copilot 無償範囲再確認

今回の無償範囲確認では、GitHub-hosted model の prompt、課金、trial、quota消費、契約種別を実行・推定していない。GitHub公式資料が示す BYOK/offline 経路だけを、既存のローカル Ollama model に限定して検証した。

- 公開source consumerの `acp_provider_probe github-copilot` は、`COPILOT_OFFLINE=true`、既存 `qwen3.5:4b-q4_K_M`、toolなし、token環境変数除外で exit 0 となった。`initialize`、`session/new`、local BYOK prompt、stream、完了応答を確認し、証跡は `tmp/verification/copilot-ime-acceptance/copilot/kcu-public-byok-agent103-probe-2026-10-24.json` に保存した。
- 同じ公開sourceの独立 cancellation consumer は、cancel pending 中の submit 拒否、draft/live conversation 保持、終端、次の `KCU_CANCEL_RESUMED` prompt 完了まで exit 0 となった。これは KCU host の cancel/resume 契約の証拠であり、wire `stopReason=cancelled` の確認とは分離する。証跡は `tmp/verification/copilot-ime-acceptance/copilot/root-cancel-actual-result.json` である。
- cloud 経路は promptなしの `initialize` / `session/new` のみを実行し、session ID、capability、config option、auth method の個数を取得した。model catalog、entitlement、Free quota、cloud generation の成功は確認していない。証跡は `tmp/verification/copilot-ime-acceptance/copilot/cloud-handshake-config-sanitized-2026-10-24T17-32-55-516Z.json` である。
- 実行イメージは全段階で `/opt/homebrew/Caskroom/copilot-cli/1.0.83/copilot`（SHA256 `15f218a936f693a6b73df248824b9f7f528c2c61949ff446e4ca6062ee48b084`）だった。同じ実体でも `COPILOT_AUTO_UPDATE=false` のcloud caseは CLI/ACP self-report 1.0.83、offline local BYOKかつauto-update未設定のcaseは CLI/ACP self-report 1.0.103 となった。これは runtime の self-report/選択差として記録し、別の有効な1.0.103実行imageが存在するとは扱わない。paired証跡は `tmp/verification/copilot-ime-acceptance/copilot/cloud-identity-postsample-2026-10-25T02-36-02.json` にある。

したがって、過去の Copilot Free prompt成功記録を現在の Free契約保証へ流用しない。現在確認できるのは「既存ローカル BYOK の実ACP接続・KCU host cancel/resume」と「cloud handshakeのみ」であり、GitHub-hosted model の無償利用可能性・quota・契約状態は未確認のまま残す。根拠は [Copilot ACP server](https://docs.github.com/en/copilot/reference/copilot-cli-reference/acp-server)、[Copilot authentication/BYOK/offline](https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli)、[CLI reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-command-reference) である。

## 2026-10-23 の再確認

Rust SDKを公開registryの3.5.0へ更新した。npm registryの最新版照合でCodex ACP 2.2.2、Claude Agent ACP 0.94.0を確認し、hostの固定起動profileを更新した。Node.js 26.12.1で、Codexはprotocol v1 initialize/session作成と同一sessionの2streamed応答に成功した（`tmp/verification/codex-acp-latest-real.log`）。Claudeはinitialize/session作成のみ成功した（`tmp/verification/claude-acp-latest-handshake.log`）。Claudeの実prompt・利用資格・認証成功までを証明するものではなく、再認証や有料契約、モデル取得は実施していない。

同日の環境再照合ではCopilot CLI 1.0.103とOllamaの `qwen3:4b` が導入済みだった。再buildしたKCU provider probeは、Copilotの保存済み認証による実応答（`tmp/verification/copilot-current-real.log`）とKatanAgentのdirect Ollama応答（`tmp/verification/katanagent-current-real.log`）に成功した。Claude Codeも、検証専用の `CLAUDE_CONFIG_DIR` と `permissions.defaultMode=default` を指定し、local Ollamaで厳密な応答検査に成功した（`tmp/verification/claude-local-default-mode-real.log`）。これはAnthropicのClaudeモデルではなくClaude Agent ACP 0.94.0とローカルqwen3:4bの接続である。設定は `tmp/verification/local-provider.env` とその参照先に残し、global Claude設定、追加ログイン・有料契約・model downloadは変更していない。

最初のClaude local応答には、確認用文字列の後へClaude Codeのauto mode互換性警告が付加されていた（`tmp/verification/claude-local-current-diagnosis.log`）。警告を除去して検査を通す処理は追加せず、公式の [CLAUDE_CONFIG_DIR](https://code.claude.com/docs/en/env-vars) と通常の [default permission mode](https://code.claude.com/docs/en/permission-modes) による設定分離で解消した。callbackの自動承認、`acceptEdits`への切替、検査の厳密一致条件の緩和は行っていない。

同じ更新後構成のManaged history実通信probeも成功した（`tmp/verification/acp-history-updated-real.log`、exit 0）。保存→新Host→Managed SelectHistory→model保持→会話継続を検査し、RPC metadataのCodex `session/prompt`は明示した3件だけで、履歴選択による再送信がないことを確認した。traceの場所は`tmp/verification/acp-history-updated-root.txt`に記録している。3hostの全UI操作や他providerの履歴互換性までの証拠には広げない。

2026-10-23の最終ACP修正後もreal history probeが成功した（`tmp/verification/acp-history-final-traced-real.log`）。recorderのCodex metadataはinitialize 5、session/new 3、session/set_config_option 2、session/prompt 3で、履歴選択によるprompt再送はない。traceは `acp-history-final-traced-root.txt` に記録した。最初の再実行はrecorder shimを付け忘れ、空traceをstrict検査が拒否したため、成功扱いせず実recorderを介して再実行した。

プロジェクト内の明示的なlocal起動設定は `.codex/local-provider.env` と `.codex/claude-local/settings.json` に保存し、Git管理から除外した。`just manual-ui-local-claude`（引数egui/floem/gpui）と `just provider-probe-local-claude` が同じ設定を子processへ渡す。後者からlocal qwen3:4bの厳密応答検査が成功した（`tmp/verification/claude-local-launcher-real.log`）。通常の起動やglobal環境は変更せず、認証・model取得・課金も自動実行しない。launcherの環境引渡し・無効設定拒否の7 unit contractはhost-support-checkに含める。

以下の確認日は旧adapterによる過去の記録である。現在のprofileと検証結果は本節および`openspec/release-tasks.md`を優先する。

確認日: 2026-09-05（過去の記録）

model変更の拒否は、SDKが通信断と区別できるInvalidRequest/MethodNotFound/InvalidParamsだけをrequest単位の失敗として表示し、通常接続・既存model・保留中permission・streamを保つ。InternalError等は従来どおり利用不可として扱う。履歴modelの復元は送信queueへの投入では完了にせず、要求したconfig IDとagent確定modelの一致を応答で確認してから利用可能にする。拒否・不一致では保存済みmodelを保ち、理由を表示する。

この修正後のreal history probeも成功した（`tmp/verification/acp-history-confirmed-real.log`、probe exit 0）。新規8 traceにcapture failureはなく、Codexはsession/new 3、session/set_config_option 2、session/prompt 3。保存・新Host・Managed SelectHistory・model保持・会話継続・履歴非再送のstrict検査が通った。`acp-history-confirmed-summary.json`へ集計を保存した。実行後のwrapperで予約済みshell変数名によるエラーが出たが、probeのexit 0とtrace保存後のもので、別の集計コマンドもexit 0で確認した。

## 結論

`katana-acp-client` に公式 Rust SDK 2.1.0 を使う ACP v1 stdio transport と内部contentのwire変換を追加した。
純KUC描画とは責務を分離するが、providerの接続可否・model候補・permission応答・streamingをUIへ反映する必要があり、影響なしとは扱わない。
実Codex ACP 1.10.0との`initialize`（protocol 1）と`session/new`（config 5件）は成功した。
全providerと全操作の互換性は未検証であり、接続成功だけでOpenSpec全要件を達成したとはしない。

本書はACP境界の確認記録であり、純KUC移行の完了を示すものではない。移行の未対応項目は
`openspec/release-tasks.md` で管理する。

multi-vendor 提供は provider-neutral な UI 契約と ACP / direct provider 境界によって
実現する。ACP 接続では各 agent の capability negotiation に従い、
対応していない機能を UI から要求しない。

## 現在の実装範囲

- ACPのToolKindを共通UIの作業状態へ投影する。Readは読み込み中、Edit/Delete/Moveは編集中、Searchは検索中、Fetchはweb検索中、Executeは実行中とする。種別を含まないToolCallUpdateでは現在の状態を保持する。変換とsessionへの反映はhost-supportの回帰テストで検証し、実provider全体の受入検証とは区別する。
- `crates/katana-acp-client/src/lib.rs` は provider-neutral な `AiProvider`、入力・出力型、`AcpError` を定義する。
- `src/protocol/transport/` はstdio起動、initialize、session/new、prompt/update/cancel、config変更、host選択を待つpermission応答を提供する。UI hostへの接続は移行中。
- `crates/katana-acp-client/src/ollama/provider.rs` は Ollama の HTTP API (`/api/chat`) を
  直接利用する provider であり、ACP agent 接続ではない。
- `crates/katana-chat-ui/src/input.rs` と `src/input/attachment.rs` は UI 入力から内部の
  `AcpContentBlock` を組み立てる。`AcpContentEncoder`が公式schemaへ変換してからtransportが送信する。
- `crates/katana-chat-ui/src/agent.rs` の `AcpProcess` は実行設定の型である。
  `tools/host-support` の `ProviderManager` が実 transport を起動し、接続結果を UI へ投影する。
  manual host は共有 `ManagedChatHost` への接続を進めている。CLI の存在だけでは利用可能にしない。

`AcpContentBlock` は UI の内部表現であり、ACP v1 の wire schema 型ではない。現状の
Serde 表現は `Text` などの Rust enum 名と `mime_type`、`data_ref` を使う。ACP v1 は
`type` discriminator と camelCase を使い、たとえば text は
`{"type":"text","text":"..."}`、resource は
`{"type":"resource","resource":{"uri":"...","mimeType":"...","text":"..."}}`
である。内部型の保存形式を維持し、wire用encoderで変換する。imageとembedded resourceは広告capabilityを要求し、不正base64やURLだけの画像は拒否する。

## ACP v1 の実装基準

公式の [ACP v1 overview](https://agentclientprotocol.com/protocol/v1/overview) は JSON-RPC
2.0 を定義し、`initialize`、`session/new`、`session/prompt`、`session/update`、
`session/cancel` の流れを示す。

- 接続後の最初の request は `initialize` とし、version と capability を交渉する。
  [Initialization](https://agentclientprotocol.com/protocol/v1/initialization) に従い、
  省略された capability は未対応として扱う。
- `session/new`、`session/prompt`、`session/update`、`session/request_permission`、
  `session/cancel` と `stopReason` を扱う。`session/load` と `session/resume` は、agent が
  capability を広告した場合にだけ送信する。
- prompt の content は [Prompt Turn](https://agentclientprotocol.com/protocol/v1/prompt-turn)
  と [v1 schema](https://agentclientprotocol.com/protocol/v1/schema) の `ContentBlock` を
  使用する。text と resource link は基準機能、image と embedded resource は agent の
  `promptCapabilities` を確認してから有効にする。
- agent が `session/request_permission` を送った場合、UI はユーザーの選択を得てから
  応答する。path は ACP の要件どおり絶対パスだけを扱う。

公式 Rust SDK は `agent-client-protocol` 3.4.0 であることを `cargo info
agent-client-protocol@3.4.0` と
[rust-sdk releases](https://github.com/agentclientprotocol/rust-sdk/releases) で確認した。
同 crate の MSRV は Rust 1.88.0 であり、この workspace の Rust 1.95.0 と互換である。
SDK API の v3 は v2 からの破壊的変更を含む一方、stable ACP v1 wire schema は維持される。導入時は
stable v1 のみを対象とし、`unstable_protocol_v2` を有効化しない。

## 検証条件

ACP transport の検証では mock による IT を使わない。実 ACP agent を起動して次を検証する。

1. `initialize` が最初の request であり、version 不一致時に接続を停止すること。
2. agent が広告しない capability の method、content、session 操作を送信しないこと。
3. `session/new` から `session/prompt`、message/plan/tool/usage update、permission 応答、
   `stopReason` までを実 transport 上で処理すること。
4. text、resource、resource link、画像（対応 agent のみ）が公式 v1 schema の
   discriminator と camelCase で送信されること。
5. `session/cancel`、permission 拒否、絶対パスでない入力を実 transport 上で扱うこと。

`session/load` / `session/resume`、terminal、filesystem、terminal authentication、logout、
elicitation、MCP-over-ACP は capability 単位の後続実装とする。

## vendor 実 probe の起動条件

利用可否は CLI の検出や package version では判断しない。各 command を stdio subprocess として
起動し、`initialize` の protocol/capability negotiation と `session/new` が成功した場合だけ利用可能とする。

- Codex: `npx -y @agentclientprotocol/codex-acp@1.10.0`。adapter は ChatGPT login または
  `CODEX_API_KEY` / `OPENAI_API_KEY` による認証を広告する。
  [公式 adapter README](https://github.com/agentclientprotocol/codex-acp) を根拠とする。
- GitHub Copilot: `copilot --acp --stdio`。GitHub Copilot CLI の認証済み GitHub account、または
  BYOK provider の設定が必要である。ACP は public preview であり、`--allow-all-tools` は
  programmatic 利用向けでもあるが、KCU の probe や通常 host では自動承認として使用しない。
  [GitHub の ACP server 文書](https://docs.github.com/en/copilot/reference/copilot-cli-reference/acp-server) と
  [CLI option 文書](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-command-reference) を根拠とする。
- Claude Agent: Anthropic の Claude Code CLI 自体が native ACP server である公式根拠は確認できない。
  一方、ACP 公式の agents 一覧は Claude Agent を Zed の SDK adapter 経由で掲載し、
  [公式 adapter](https://github.com/agentclientprotocol/claude-agent-acp) は Claude Agent SDK を
  ACP agent として公開している。実 probe は `npx -y @agentclientprotocol/claude-agent-acp@0.74.0` を使用し、
  Claude Agent SDK の利用可能な認証状態が前提となる。2026-09-05 に `npm view
  @agentclientprotocol/claude-agent-acp version` が `0.74.0` を返した。package の取得だけを
  接続成功とは扱わず、`initialize` / `session/new` と実 prompt の結果を別途記録する。

Claude の実 probe（`tmp/claude-acp-real-probe.log`）は protocol 1 initialize と
session/new（config 4件）まで成功し、prompt は OAuth session expired / authentication_failed
で終了した（exit 1）。既存ビルド済み probe を用いた adapter 接続の診断であり、最新差分の
release 検証や Claude の実会話成功の証拠には含めない。認証更新後の再検証が必要である。

## 2026-09-07 の過去のローカル Ollama 検証証拠

Claude Code の hosted OAuth を使わない構成として、`KCU_CLAUDE_OLLAMA_MODEL` を指定した
Claude provider profile だけに、次の子プロセス環境を渡す実装を追加した。host process 全体の
環境は変更しない。

- `ANTHROPIC_AUTH_TOKEN=ollama`
- `ANTHROPIC_API_KEY=`
- `ANTHROPIC_BASE_URL` は `KCU_OLLAMA_ENDPOINT`、未指定時は `http://localhost:11434`
- `ANTHROPIC_MODEL` と `ANTHROPIC_CUSTOM_MODEL_OPTION` は
  `KCU_CLAUDE_OLLAMA_MODEL` の値

`qwen3:4b` を指定した KCU の `acp_provider_probe claude-code` は、親processに
`ANTHROPIC_*` を設定せずに ACP の initialize、session/new、prompt、完了応答まで成功した。
ローカル推論は応答まで105秒を要したため、probeの完了上限は90秒から180秒へ更新した。これは
Claudeのホストサービス利用を示すものではなく、Claude Agent SDK adapterを通じたローカルOllama
互換APIの検証である。

同じKCU provider probeで、Codex CLI、Copilot Free、KatanAgent/Ollama の各prompt完了も成功した。
KatanAgentはACP adapterではないため、Ollama model catalogの取得、model選択、direct streamingの
同一host pipelineを検証した。Copilotは本人がGitHubのOAuth/TOTPを完了したFree accountを使い、
有料契約・trial・BYOK設定は使っていない。

## 現在の可用性と無課金・未導入環境の扱い

保存履歴はmessage本文・Text/File添付に加え、source_message_idで対応する構造化outputを渡す。生成ファイル本文と編集後本文を含む保存payload・種類・候補状態をACP/KatanAgent共通のescaped dataへ変換し、未適用候補を適用済みと扱わない。ファイルの再読込や承認操作は行わず、provider別の確定message位置・再試行・会話境界に従う。

KatanAgentの通常の再選択・他providerからの復帰では、executorが実際に選択済みのmodelをUIと保存履歴へ反映する。実行中・取消完了待ちの同一provider再選択にも、stream開始時の確定modelを保持する。新規または選択消去済みexecutorは未選択のままとし、履歴復元の未確認値は既存confirmationを待つ。

標準composerから`tmp/report.md を編集`のように既存Markdownを明示した現在の要求では、該当するFileResource添付がなければ安全に解決したworkspace/tmp内の既存UTF-8本文をModify contextへ渡す。従来の添付優先は維持し、未存在・境界外・symlink escape・親参照・不正UTF-8・複数対象は明示拒否する。これは明示された現在の編集要求に限定した読取であり、履歴の添付/outputを再読込しない契約は変更しない。

前節は2026-09-07時点の接続成功の記録であり、現在のprovider可用性を表すものではない。現在は次の扱いを契約とする。

以下は製品の自動動作の境界である。ユーザーが依頼したCLI設定・無料アカウント接続・ローカルモデルの再導入を、作業者が明示的に行うことは禁止していない。今回の依頼ではその環境設定が承認されている。有料契約・有料trialを開始せず、必要な本人認証だけは本人操作を要する。

- ClaudeとCopilotは、既存の認証済みCLIが`initialize`と`session/new`を完了してReadyになった場合だけ選択肢に入る。未認証・CLI未導入時に認証画面、課金、trialを開始しない。
- ClaudeのOllama互換profileは`KCU_CLAUDE_OLLAMA_MODEL`を明示した場合だけ子processへ環境を渡す。指定modelが導入済みでない状態を補うためのdownload処理は持たない。
- KatanAgent/Ollamaは`/api/tags`で導入済みmodelだけを列挙する。catalogが空なら利用不能とし、`/api/pull`、`ollama pull`、model downloadを実行しない。
- 認証なし・CLI未導入・Ollama catalog空の環境ではproviderを利用可能と表示せず、過去probeをrelease時点の実接続証拠へ流用しない。現在の環境の観測結果は冒頭の最新検証を優先する。
