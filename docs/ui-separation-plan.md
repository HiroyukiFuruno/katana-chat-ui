# katana-chat-ui — UI 分離計画 抜粋

作成日: 2026-05-17  
canonical: [`katana/docs/architecture/ui-separation/detailed-design-and-tasks.md`](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md)

## このファイルの位置付け

本ファイルは KatanA ecosystem の **UI 分離構想 master** から `katana-chat-ui` (KCU) 担当部分を抜粋したもの。task ID は master と同一。**master が単一情報源**。

## Repository の役割

`katana-chat-ui` (KCU) は **framework-neutral AI chat UI state + lightweight render model** を所有する。

- host applications がそのモデルを自前の UI framework で render するという既存方針を維持。
- 今回の UI 分離構想とは方向性が一致しており、KUC (`katana-ui-core`) と親和性が高い。
- 接続は Phase 4 で `katana-ui::ChatPane` の adapter target として実装される。本 repo は接続側で受け入れ可能な render model の輸出責任を持つ。

詳細: master [`1.8 katana-chat-ui`](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md#18-katana-chat-ui)

## 担当 Phase

- **Phase 4-C / 4-D (受け側のみ)**: `katana-ui::ChatPane` が KCU render model を adapter で消費する。本 repo は render model export を保つだけでよい。
- **後続**: 必要に応じて KUC Component model に揃える tuning。今回の Phase に直接の重い作業は無い。

## Task list (master 抜粋)

### Phase 4 接続側 (本 repo は受け入れ可能であればよい)

- [ ] P4-C-003: `ChatPane` を定義する。(katana-ui repo 側 task / 本 repo は API 確認のみ)
- [ ] P4-D-003: `ChatPane` が KCU render model adapter を呼ぶ boundary を作る。(katana-ui repo 側 / 本 repo は render model API を保つ責任)

### 本 repo 側の方針確認

- [ ] KCU-MIGRATE-001 (本 repo 個別 task): chat render model が framework-neutral であることを再確認する。framework 型を露出させない。
- [ ] KCU-MIGRATE-002: render model が KUC `UiTree` / KUC Component model に対応する DTO シェイプを持つことを確認する。必要なら adapter trait を本 repo 側で定義する。
- [ ] KCU-MIGRATE-003: KCU README の責務説明が「framework-neutral chat render model」と「host application が UI framework で render」になっていることを確認する。

> 注: `KCU-MIGRATE-*` は master 側にまだ task ID として登録されていない。master に追加するか本 repo 個別 task として扱うかは P8-B 検討時に決定する。それまでは本ファイル単独の暫定 task として進める。

## 前提 (depends on) / 出力 (provides)

- **前提**:
  - P1 で KUC render model (`UiTree` / `UiNodeKind` / `UiProps`) が確定していること
  - P4-0 で primary adapter が確定していること

- **出力**:
  - framework-neutral chat render model API (現状維持)
  - KUC Component model に対応する DTO 構造の確認 (必要なら adapter trait)

## Done criteria

本 repo に関する master 9 章 Done criteria のうち、該当項目:

- [ ] `katana-ui::ChatPane` から KCU render model adapter 経由で chat 表示が動く (Phase 5 段階で実機検証)
- [ ] KCU が framework-specific UI 型を export していない

## 重要な非該当事項

- 今回の UI 分離構想で本 repo は **大きな変更を行わない**。既存の framework-neutral 方針が活きるため、KUC / katana-ui との接続時に微調整が発生する程度。
- chat の詳細仕様は別系統 (`docs/chat-ui-spec.ja.md`) に既存。本ファイルは UI 分離構想との接続のみを扱う。

## drift 検出

- 本ファイルの task ID は master と一致する範囲 (P4-C / P4-D) と、`KCU-MIGRATE-*` の暫定 task を持つ。後者は master に統合された段階で本ファイルから消す。
- P8-A-001 の CI script では `KCU-MIGRATE-*` を例外扱いするか、master に正式登録するかを後で判断する。

## 参照リンク

- [master detailed-design-and-tasks.md](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md)
- [master principles.md](../../katana/docs/architecture/ui-separation/principles.md)
- [overview README](../../katana/docs/architecture/ui-separation/README.md)
- [既存 docs/chat-ui-spec.ja.md](chat-ui-spec.ja.md)
- [既存 docs/release.md](release.md)
