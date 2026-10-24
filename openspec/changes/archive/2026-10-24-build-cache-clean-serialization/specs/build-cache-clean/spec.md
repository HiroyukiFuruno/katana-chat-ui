# Build cache cleanup serialization

## ADDED Requirements

### Requirement: shared repository lock
通常の Cargo 実行と cleanup は同じ KCU repository lock を使用 MUST。通常実行は shared lock、cleanup は non-blocking exclusive lock MUST。

#### Scenario: cleanup overlaps a running Cargo command
- **WHEN** 通常の guarded Cargo command が実行中に cleanup が開始される
- **THEN** cleanup は待機せず exit 3 で拒否される

#### Scenario: normal Cargo commands overlap
- **WHEN** 2つの通常 guarded Cargo command が同時に開始される
- **THEN** 両方が同じ shared lock を取得して実行を継続 MUST。

### Requirement: safe target boundary
Guard は実際の Cargo prefix で7 manifestそれぞれの metadata を実行し、target_directory、`CARGO_TARGET_DIR`、Cargo config を反映して全7つの targetを専用cleanup MUST。既存 target は Cargo `CACHEDIR.TAG` 所有署名を要求し、repository 外、source、`.git`、symlink、metadata失敗を拒否 MUST。`--manifest-path`、`--package`、`--profile`、`--target`、`--target-dir`など選択的cleanフィルターは非対応として拒否 MUST。

#### Scenario: unsafe target is selected
- **WHEN** target が repository 外または symlink を経由する
- **THEN** command は実行されず fail-closed で終了する

#### Scenario: metadata or ownership is unavailable
- **WHEN** metadata が失敗する、または既存 target の所有署名が欠落する
- **THEN** cleanup は実コマンドを実行せず fail-closed MUST。

### Requirement: open file safety
Cleanup は target 内の open file を検出した場合、または検査ツールが利用できない場合に拒否 MUST。

#### Scenario: unguarded process holds a target file
- **WHEN** cleanup 中に target の file descriptor が別 process に保持される
- **THEN** cleanup は target を変更せず exit 3 で終了する

### Requirement: normal command compatibility
Guard は通常 command の引数、parallelism、終了コードを変更してはならない MUST。

#### Scenario: wrapped command exits nonzero
- **WHEN** guarded command が exit code 7 で終了する
- **THEN** wrapper も exit code 7 を返す

### Requirement: repository-clean target contents
既存 target の内部に Git tracked または non-ignored untracked repository file が存在する場合、Guard は cleanup を実行せず fail-closed MUST。Git checkout root の確認、Git routing environment の除去、literal directory pathspec による inspection のいずれかが失敗した場合も fail-closed MUST。Gitignore で対象全体を無視した default/shared/nested target と、その内部の Cargo.toml/src 生成物は受け入れ MUST。

#### Scenario: repository-owned file is inside target
- **WHEN** owned target の配下に tracked または non-ignored untracked file がある
- **THEN** cleanup は実コマンドを実行せず fail-closed で終了する

#### Scenario: ignored generated package is inside target
- **WHEN** default/shared/nested target と Cargo 生成物が repository の ignore 規則で無視されている
- **THEN** target boundary は受け入れられ、通常の cleanup 検査へ進む
