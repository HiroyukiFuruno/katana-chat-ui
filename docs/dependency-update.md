# 依存関係更新の判断記録

## 2026-10-23 GPUI Linux の xattr 互換制約

GPUI 0.2.2 の公開依存は `gpui_http_client 0.2.2`、`zed-async-tar 0.5.0-zed`、`xattr 0.2.3` で固定されている。crates.io の最新公開版はそれぞれこの系列で、xattr 1.x対応を含む新しい公開版は確認できない。`xattr 0.2.3` は Linux/aarch64 で `libc::ENOATTR` を参照するが、libc 0.2.190 でこの alias が削除されたため、fresh resolveでも libc 0.2.191 なら失敗する。

そこでGPUI adapterのLinux targetだけに `libc = "=0.2.189"` を追加した。Androidは今回のサポート対象へ追加していない。公式libc sourceでは0.2.189がLinux `ENOATTR = ENODATA` aliasを持つ最後の公開patchであり、0.2.190以降は持たない。[0.2.189 source](https://raw.githubusercontent.com/rust-lang/libc/0.2.189/src/unix/linux_like/linux/mod.rs)、[0.2.190 source](https://raw.githubusercontent.com/rust-lang/libc/0.2.190/src/unix/linux_like/linux/mod.rs)。Cargo lockは全targetを解決するため、rootと`tools/manual-host-gpui`のmacOS側にも同じlibc 0.2.189が記録される。xattrを含まない他5 lockはlibc 0.2.191を維持する。

公式upstreamの現行async-tarは既に `xattr = "1.6"` を採用しているが、対応する新しい`zed-async-tar`公開版は存在しない。[公式async-tar manifest](https://github.com/zed-industries/async-tar/blob/master/Cargo.toml#L334-L346)、[xattr 1.6.1](https://docs.rs/crate/xattr/latest)。このpinは、GPUI/async-tarの公開更新版がxattr 1.xへ移行した時点で解除する。

## 2026-10-23 KUC 0.4.4の公開版採用

KUCのPR #106は最終54c1782の5回目cloud review・required checks成功後にmergeした。Release run [39930491269](https://github.com/HiroyukiFuruno/katana-ui-core/actions/runs/39930491269)はsuccessで、[v0.4.4](https://github.com/HiroyukiFuruno/katana-ui-core/releases/tag/v0.4.4)と独立workspaceでのcrates.io実downloadを確認した。KCUの直接manifest2箇所・全7lockfileはregistry版0.4.4であり、未公開path/git/fork依存は採用していない（`tmp/verification/kuc044-seven-locks-root.json`）。

全7構成の`just update-safe`と`just outdated`がexit0となり、cc 1.10.0、clipboard-win 5.4.3、uuid 1.29.0、webpki-root-certs 1.0.11を含む最新互換版へ更新した（`tmp/verification/{update-safe,outdated}-final-kuc044.log`）。残るindexmap/hashbrown/schemarsの旧系列は上流の互換featureが要求し、raw-window-handleはFloem/NDKが異なる系列を要求する。直接依存のmajor更新候補はなく、推移依存の系列を強制置換しない。

更新後のcoreは162/162、focused geometry/copyは5/5、strict lint・AST19・fmtは成功した。最初に検出した2失敗は、色付きglyph inkと下線の測定対象の混同、隣接行と共有する選択境界入力の誤りを数値診断し、既存のcontainment・actual hit内部pixel・厳密copyを維持して修正した。Linux GPUIの互換制約追加後、完全`just check`もexit0で既存5画像・native15画像・Storybook実window smokeが成功した（`kuc044-full-just-check-libc189.log`）。旧source整理後の最終release-check・coverage・公開後consumer受入はまだ未完了である。

## 2026-10-23 公開registry再照合

開発用OpenSpec launcherは、公式npm registryのlatestを照合して `@fission-ai/openspec` を1.3.1から1.14.3へ更新した。package指定と実CLIのversion出力が一致し、最新版による全6項目のstrict validationが成功した（`tmp/verification/openspec-latest-version.log`、`openspec-1-14-3-validate-final.log`）。最新版が検出した500文字超の要件文は、行高と空Cardの扱いを独立した要件へ分け、既存の動作・検査範囲を維持した。launcherの既存override経路は変更しない。根拠: [公式npm package](https://www.npmjs.com/package/@fission-ai/openspec)、[公式repository](https://github.com/Fission-AI/OpenSpec)。

CI Actionsも公式release/refを再照合した。checkout v7.0.1、upload-artifact v7.0.2、rust-cache v2.9.3、install-action v2.87.30が現行参照の互換系列にあり、toolchainはstableを追従する。既存のmajor/tool別参照方針を維持し、未使用のdownload actionやOIDC認証を追加しない。uploadはdirectoryを既存のZIP artifactとして保存し、release認証は既存のregistry token経路を維持する。公式根拠: [checkout](https://github.com/actions/checkout/releases/tag/v7.0.1)、[upload-artifact](https://github.com/actions/upload-artifact/releases/tag/v7.0.2)、[rust-cache](https://github.com/Swatinem/rust-cache/releases/tag/v2.9.3)、[install-action](https://github.com/taiki-e/install-action/releases/tag/v2.87.30)。mutable refのSHA固定は依存の旧版残存とは区別する。最終HEADのCI成功確認はリリースgateで行う。

`just outdated` を全7構成で実行し、`just update-safe` で互換範囲のlockfileを更新した（両方exit 0）。ACPは3.5.0、Tokioは1.54.1へ更新され、ICU 2.4系列などの推移依存も更新された。詳細は `tmp/verification/outdated-current.log` と `tmp/verification/dependency-update-current.log` に記録した。

更新後のACP clientは17テスト、host-supportは厳格lintと75テストが成功した（`tmp/verification/acp-client-updated-tests.log`、`tmp/verification/host-support-updated-check.log`）。全体の採否にはKUC互換版を取り込んだ後の完全release gateと既存画像matrixが必要であり、現時点ではリリース完了を意味しない。

KUC release run 39854764311の成功、GitHub Release v0.4.3、`cargo info`による公開registry取得を確認し、直接manifest2箇所と全7lockfileを0.4.3へ更新した。最終の全7構成 `just outdated` も成功し、新しい直接依存の更新候補はない（`tmp/verification/outdated-final-kuc043.log`）。残るindexmap/hashbrown/schemarsは上流が要求する旧系列、raw-window-handleはFloem/NDKの相反する系列であり、互換範囲外を強制しない。code rasterは本文System設定と分離し、macOSのLatin baselineに対応するSFNS.ttfをcodeの候補に指定した。既存5画像は基準を変えず個別に一致を確認した。最終release-checkによる採否確認は継続中である。

2026-10-23公開前の記録: KUC 0.4.3採用後の実consumer検査で#104/#105が再現したため、上流の公開修正版v0.4.4を待って採用する。上流の修正後1059テストは成功したが、上流の完全gate・公開とKCUでの最終再検証は未完了である。詳細はactive changeとrelease台帳を優先する。

## 2026-10-18 KUC 0.4.1 / ACP 3.4.0 再評価（上流互換性待ち）

作業ツリーでは `katana-ui-core 0.4.1` と `agent-client-protocol 3.4.0` を公開registry依存として更新し、ACPの直接機能は `process` featureだけを明示して既定機能へ依存しない構成にした。KUC 0.4.1 / ACP 3.4.0でstrict lint、workspace test、host support、adapter gate、Storybook smoke、E2Eまでは通過したが、`release-check`の既存スクリーンショットmatrixで停止した。

原因はKUC 0.4.1の公開 `Card` が非layout childへ固定16pxの子インデントを適用し、その無効化APIを提供していないことである。既存のwide本文560px契約と5画面の画像基準を同時に守るには、KUC上流が描画・scroll measure・hit testに一貫して適用する公開設定を提供する必要がある。上流Issueは [katana-ui-core#83](https://github.com/HiroyukiFuruno/katana-ui-core/issues/83) として記録した。未公開のpath/git/fork依存、baseline更新、品質基準の緩和はいずれも採用しない。

`eframe`、`egui`、`egui_kittest` は公開registryの0.37.0へ採用済みである。Egui adapter 13件、manual Egui host 7件、Storybook 6件、screenshot executor 12件、workspace 210件、Storybook native smoke、format、strict lint、依存treeを確認した。KUC互換版が公開された後、既存画像基準を含む同一release gateを再実行する。したがって、v0.1.0はまだPR・公開・リリース完了ではない。

## 純KUC移行後の再検証状況

以下は2026-09-07の `katana-ui-core 0.3.7` 採用時の検証証跡である。現在の依存採否とリリース状態は前節およびOpenSpec `kuc-0-4-1-card-compatibility` を優先する。rootの依存は `default-features = false` のregistry dependencyだけであり、path/git依存はリリース構成に含めない。

`just update-safe` 後、Crossbeam 0.8.5系、ipnet 2.12.2、libgit2-sys 0.18.8+1.9.7、lyon_tessellation 1.0.22、redox_syscall 0.9.4を含む互換範囲の推移依存を更新した。更新済みlockfileで `just lint`、`just unit-test`（185件）、`just ast-lint`（19件）、`just host-support-check`（75件）、`just host-e2e`（9件）、manual host gates（Egui 7、Floem 53、GPUI 6）、headless/native screenshot matrix、`just coverage`（82.86%、基準77%）を再実行した。

2026-10-18の全7構成の `just outdated` 再走査では、新しい直接依存の更新候補はない。`hashbrown`、`indexmap`、`schemars` の表示は上流依存が要求する旧系列、`raw-window-handle` の表示はFloem/NDKの0.5系と0.6系の相反要求である。`autocfg` のRemoved表示も更新候補ではない。互換性と検証基準を下げてまで統一しない。

2026-10-18に公式GitHub Releasesを再確認した。`actions/checkout`はv7.0.1、`actions/upload-artifact`はv7.0.2が最新である。workflowは両者を`@v7`で参照するため、互換性を保ったままv7系の最新patchを取得する。

## 2026-09-07

公開KUCを `0.3.6` から `0.3.7` へ更新し、root、screenshot、E2E host、3 manual host、host-support の全7 lockfileを同期した。E2E hostが直接宣言するKUCも `0.3.7` に更新した。KUCのscroll bounds/content bounds APIは新設ではなく0.3.6から公開済みであるため、KCU側のcomposer rendererで測定済みcontent boundsを同期し、KUCのscroll offsetをRGBA source cropへ反映した。`kuc::runtime::tests` は42件PASSし、composerの複数行入力と長文Markdown本文のscrollを実ピクセル差分で検証する。全workspace品質gateと依存再評価は上記のとおり更新済みである。

全7構成の `just outdated` 再確認52488（`tmp/thinking-width-outdated.log`）では新たな直接依存の更新候補はない。残存indexmap/hashbrown/schemarsは上流由来、raw-window-handleは0.5/0.6の相反要求、その他は条件付きtargetの上流依存である。調査成功を全推移依存が最新版である証拠とは扱わない。

下記の旧native画像hashに基づくegui採否と79.09% coverage、および過去の未検証記述は当時の証拠であり、最新の状態は上記とrelease-tasks.mdを優先する。

GPUI hostの新しい共有host依存をlockへ反映し、strict clippy/check/3 testsは通過した。
`block 0.1.6`と`proc-macro-error2 2.0.1`にはRustのfuture-incompatibility警告が残る。
`cargo tree --locked -i`で前者はGPUIのmacOS依存群、後者は
`gpui → stacksafe → stacksafe-macro`由来と確認した。警告の詳細は
`tmp/dependency-future-compatibility.log`に記録し、依存採否の未解決事項として扱う。

## 2026-09-05

全7構成の`just outdated`再実行後、ACPの直接依存base64を0.23.1へ更新し、全lockの参照を検証した。既存STANDARD.decode APIは互換で、ACPの17テストが通過（`tmp/acp-base64-update-test.log`）。他依存が要求するbase640.22系列は残る。e2e-host、manual-egui、manual-floem、screenshotのlockでredox_syscallを0.9.3から0.9.4へ更新した。最終全体gateは未完了。

GitHub Actionsも公式Releases APIで再評価した。`actions/checkout`と`actions/upload-artifact`はともに最新`v7.0.1`を確認し、既存major参照の規約に合わせて3workflowの9箇所を`@v7`へ更新した。Node24対応のGitHub-hosted runnerを使用し、checkoutのref/fetch-depthとartifactのname/path/失敗時保存を維持する。`Swatinem/rust-cache@v2`は最新v2.9.2系列、toolchainの`@stable`とinstall-actionのtool別参照は既存の追従契約を維持した。3 YAMLの構文と差分は検証済み。CIでの実行は未確認。
一次情報: [checkout v7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1)、[upload-artifact v7.0.1](https://github.com/actions/upload-artifact/releases/tag/v7.0.1)。

`just update` で workspace と各 host の依存関係を更新した。主な直接依存の更新は `comrak 0.52` から `0.54`、`syn 2` から `3`、`tokio 1.52.3` から `1.53.1` である。KUCは当時の公開版`0.3.6`へ依存する。

egui/eframe/egui_kittestは`0.36.1`へ宣言を更新した。前回のAPI互換調査は`ImeEvent`と`RawInput.modifiers`について誤りで、0.36.1ではdeprecated `Enabled`/`Disabled`を使わず、`Preedit { text, active_range_chars }`へ移行し、`RawInput`からmodifiersを除去する修正が必要だった。Cmd+EnterはKey event自身のmodifiersを使い、IME caretは文字indexのactive range終端をUTF-8 byte offsetへ変換して使う。egui 0.36の`IMEOutput`必須fieldに対応したが、root/manual-egui/screenshotのlock再解決は完了した。compileと全workspace unit170/E2E9 testsは通過した。3 manual hostとpure KUC screenshot quality gateは更新後の全検証が未完了である。旧native画像hashで0.36を不採用とした記録は移行前の証拠であり、新baselineの採否は現在の純KUC経路の品質gateで決める。更新対象lockはroot、`tools/manual-host-egui`、`scripts/screenshot`である。

現在のFloemは公開registry `0.2.0`、GPUIは公開registry `0.2.2`を使用する。以下のgit pinとnaga明示依存の説明は旧native構成の経緯であり、現manifestには残っていない。

### 移行前の更新履歴

Floem の git pin は `f2ce49556606db6d8f025443f834f04b7a6b4796` を維持した。upstream 最新 `778bb5f2aa08429e579ee2e6ac97e84fbf18b618` と、最初に wgpu 27 へ移行した `de743f4e7b3926f5f576585c1e749a6b11a78543` を read-only で評価したが、いずれも `taffy ^0.9.1` 以上を要求する。GPUI crates.io 最新 `0.2.2` は `taffy =0.9.0` を固定しており、workspace の依存解決が成立しないため採用しなかった。

旧Floem pinでは `wgpu 26` が `naga 26.0.0` を使用する。GPUI由来の `codespan-reporting 0.12` の `termcolor` feature と naga 26 の診断 writer feature が分離していたため、`String: WriteColor` エラーが発生した。`katana-chat-ui-floem` に `naga = "=26.0.0"` の `termcolor` feature を明示し、同じ lock graph の feature を統一する。cargo-upgrade による naga 30 への置換を防ぐため、Floem の wgpu 26 と同じ版を exact pin する。これは診断出力の互換修正であり、検査や品質基準の緩和ではない。

`just update`はexit 0。その後、画像基準を満たさないegui系列更新を戻し、関連lockを`cargo update`で再解決した。更新時の6つのmanifestの`cargo metadata --locked`では、依存先が宣言する`rust_version`はすべてworkspaceのMSRV 1.95以下だった。実際のコンパイル・テスト結果はOpenSpecのタスク台帳で管理する。

互換修正としてsyn 3のtrait impl構造へ対応した。Rust 1.97が検出するFloem・eguiの数値型推論警告は、値を変えず`f32`を明示して解消した。

screenshot runnerは`cargo run --release --locked`を使い、`CARGO_TARGET_DIR`指定時にも今回のbuildを実行する。matrixは最初の失敗で非zero終了する。画像基準と検証対象は変更していない。

最終構成で`just check`はexit 0、`just ast-lint`は19件PASS、`just coverage`はline coverage 79.09%で既存77%基準を満たした。これは既存提供経路と追加した共通treeの検証であり、純KUCへのnative接続完了を意味しない。
