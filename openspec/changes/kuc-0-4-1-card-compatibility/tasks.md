## 1. 現状と上流契約

- [x] 1.1 KUC 0.4.1でCard child indentがwide message幅560px、既存5画面基準、hit testを両立できないことを公開APIと実画像で再現する。
- [x] 1.2 上流Issue [katana-ui-core#83](https://github.com/HiroyukiFuruno/katana-ui-core/issues/83)へ必要な公開APIと再現条件を記録する。
- [x] 1.3 KUC 0.4.2の公開 `child_indent` を評価し、既存描画は満たしても本文viewport hit幅が544pxとなることを確認する。
- [x] 1.4 必要な公開契約を [katana-ui-core#85](https://github.com/HiroyukiFuruno/katana-ui-core/issues/85)、[katana-ui-core#86](https://github.com/HiroyukiFuruno/katana-ui-core/issues/86)、[katana-ui-core#98](https://github.com/HiroyukiFuruno/katana-ui-core/issues/98)、[katana-ui-core#100](https://github.com/HiroyukiFuruno/katana-ui-core/issues/100)へ記録する。
- [x] 1.5 KUC #85/#86/#98/#100を含むv0.4.3がGitHub Releaseと公開registryで利用可能になり、Issue解決と公開versionを確認する（Release run 39854764311成功、GitHub Release v0.4.3公開、cargo infoでregistry取得。#103も公開後close）。
- [x] 1.6 v0.4.3の実consumer回帰で見つかった [KUC #104](https://github.com/HiroyukiFuruno/katana-ui-core/issues/104)（soft wrap copyへLF混入）と [#105](https://github.com/HiroyukiFuruno/katana-ui-core/issues/105)（RTL glyphへ4096px幅漏出）を上流で修正し、v0.4.4の公開を確認する。上流修正はPR #106の5回目review・全required checks後にmergeし、Release39930491269 success、GitHub Release v0.4.4、独立workspaceで公開registry版実downloadを確認した。#104/#105/#107は公開証跡付きでcloseし、local master同期・clean/stash0・当該local release branch整理も完了した。

## 2. 公開依存への適合

- [x] 2.1 #85/#86/#98/#100を含む公開KUC v0.4.3だけを全manifest/lockへ採用し、path/git/fork依存を残さない（直接manifest2箇所、全7lockのversion/sourceを検査）。
- [x] 2.2 Card child indent 16と本文 `hit_outset_left(16)` をKCUのmessage treeへ適用し、描画・layout・hit testが同じ560px幅契約を満たすよう実装する（120/320/560pxのmessage幅回帰PASS。詳細geometry/pointer回帰は3.1で継続）。
- [x] 2.3 eframe、egui、egui_kittestを0.37系へ同時更新し、API/lockfile移行を完了する。
- [x] 2.4 本文System faceを変えず、`monospace = true` を保つcode spanだけに公開code raster設定を適用し、bidi本文をまたぐinline-codeのselection、hit、copyで段落区切りを保持する（公開KUC0.4.4、core162/focused5、完全check exit0）。
- [x] 2.5 #104/#105を修正した公開版だけを全manifest/lockへ採用し、未公開のpath/git/fork依存を残さない。
- [x] 2.6 GPUIのLinux依存でxattr 0.2.3とlibc 0.2.191のENOATTR非互換を再現し、公開upstream更新がない間はlibc 0.2.189をadapter限定でexact pinする。Androidは今回のサポート対象へ追加しない。xattr 1.x採用版公開後にpin解除を再評価する。

## 3. 契約検証

- [/] 第15回Ready後P2: 標準host surfaceで未対応のOpenPreview等を有効表示せず、実装済みCreateFile/ApplyDiff/UndoChangeだけを提示する。message/list/handoffの3投影、全output kind/status、公開intentとpayload保持、適用取消の実行性を恒久回帰で検証する（discussion_r4365716930）。完全gate・正式固定HEAD実受入・review/公開以降は親台帳で追跡する。

- [x] 同じ入力経路のBackspace選択削除を最小修正し、恒久回帰とfocused検証を通した。旧版は新2回帰のうち選択範囲assertが実失敗101、未選択のgrapheme削除は成功した（`pr4-review14-backspace-selection-red-attempt1.log`）。選択ありだけ公開KUCの空文字置換へ分岐し、両修正のfocused core169/ACP42/host153/recorder2/AST19・strict fmt/lintは実exit0（`pr4-review14-delete-selection-focused-attempt1.log`）。完全gate・正式固定HEAD実受入は3.4/4と親台帳で継続する。

- [/] 第14回Ready後P2: 共通composerのforward Deleteで末尾・空文字を保護し、現在の選択範囲または次のgraphemeだけを公開KUCの操作で削除する最小修正を追加した。前後向きselection・結合文字/emoji・改行の恒久回帰を追加し、旧版は1成功/2assert失敗101、修正後focusedはcore167/ACP42/host153/recorder2/AST19とstrict fmt/lintが実exit0（`pr4-review14-forward-delete-focused-attempt1.log`、discussion_r4365608820、0004/0014）。完全gate・正式固定HEAD実受入・review/公開以降は親台帳で追跡する。

- [x] 3.1 wide/narrow messageの幅、本文座標、right-aligned user message、pointer/hit testの回帰を追加または更新する（core162、adapter13/10/11、host E2E8+flow1、Linux native15画像が成功）。
- [x] 3.2 code spanのmodel semantics、draw/wrap/measurement/hit共有、本文raster不変、共通rounded selection/link-hit/decoration geometry、CR/LF/CRLF/U+2028/U+2029のcopy回帰を追加または更新する（`kuc044-final-gates.log`、完全checkで再成功）。
- [x] 3.3 5画面の既存スクリーンショット基準を変更せずmatrixを通す（`kuc044-full-just-check-libc189.log` exit0、root実hash照合`kuc044-fullcheck-root-accepted.json`）。
- [x] 3.4 標準host output操作修正後の`just VERSION=v0.1.0 release-check`が実child exit0。format、strict lint、AST19、core169/ACP42/host155/recorder2、unit/E2E、Storybook、coverage、package/publish dry-run、Linux matrixを確認した。source506不変、line coverage86.61%、native15/headless5の新規画像をrootが照合した（`pr4-review15-host-output-actions-final-root-accepted.json`）。正式固定HEAD受入・review/公開は4と親台帳で継続する。

- [x] 3.5 Storybookの自動smokeを既存Linux/aarch64 containerの専用Xvfb内へ移し、他作業の前面へ出ず実native frame検査が完了することを確認する。host DISPLAY/X socketは渡さず、通常のinteractive起動は維持する。macOSのeframe初回showは非active指定だけで抑制できないため、自動smokeではhostのnative windowを起動しない（完全checkで`kcu-storybook --smoke`終端と構造化exit0を回収）。

## 4. リリース

- [ ] 4.1 OpenSpec整合、自己レビュー、Draft PR、review指摘の解決、Ready、required checksを完了する。
- [ ] 4.2 merge後のrelease workflow、tag、GitHub Release、5 crates.io artifact、downstream取得、branch hygieneを確認する。

## Definition of Done

- [ ] 公開KUCのみで既存message幅・画像・操作契約・Markdown code spanのmodel/raster・bidi選択契約を維持する（v0.4.3で未達の#104/#105は公開修正版へ更新して満たす）。
- [ ] 依存更新によってbaseline、coverage、lint、AST、release基準を下げない。
- [ ] ユーザーが免除した初回リリースの物理手動確認を完了条件に戻さず、Storybook実画像とnative IME自動検証を維持する。
