## ADDED Requirements

### Requirement: kcu core は UI framework と host application に依存してはならない

システムは、`katana-chat-ui` core crate の public API から UI framework 型、host application 固有型、vendor SDK 型を排除しなければならない（MUST）。egui app は host 側 adapter で kcu の render model を描画する。

#### Scenario: core dependency tree を確認する

- **WHEN** `cargo tree -p katana-chat-ui` を実行する
- **THEN** `egui` / `eframe` / `floem` / `vello` は含まれない
- **THEN** host application 固有 crate は含まれない

#### Scenario: egui app から利用する

- **WHEN** egui を使う host application が kcu を利用する
- **THEN** host は `ChatRenderModel` と callback contract を egui widget へ写像する
- **THEN** kcu core は `egui::Ui` を受け取らない

### Requirement: 標準的な AI chat 入力を扱えなければならない

システムは、multiline text、file attachment、image attachment、path drop を `ChatInputDraft` と `Attachment` で扱えなければならない（MUST）。

#### Scenario: file path が drop される

- **WHEN** ユーザーが UI に file path を drag and drop する
- **THEN** kcu は host callback に file 読み取りを依頼する
- **THEN** host が返した内容だけを `FileResource` attachment として保持する
- **THEN** kcu core は filesystem を直接読まない

#### Scenario: agent が image input を support しない

- **WHEN** image capability のない provider に画像を添付する
- **THEN** システムは送信前に unsupported attachment state を返す
- **THEN** 画像を黙って text に変換しない

### Requirement: Markdown subset を安全に描画しなければならない

システムは、assistant message と user draft preview に code block、inline code、blockquote、ordered list、unordered list、link を描画できなければならない（MUST）。

#### Scenario: code block を表示する

- **WHEN** assistant message に fenced code block が含まれる
- **THEN** render model は code block として区別できる block を返す
- **THEN** language label は存在する場合のみ metadata として保持する

#### Scenario: raw HTML が含まれる

- **WHEN** message に raw HTML、script、style が含まれる
- **THEN** システムは実行可能 HTML として描画しない
- **THEN** text として安全に表示する

### Requirement: message role ごとの視覚差分を表現しなければならない

システムは、user、assistant、tool、system の message role ごとに visual intent を返さなければならない（MUST）。

#### Scenario: user message を表示する

- **WHEN** message role が user である
- **THEN** render model は assistant message と異なる bubble / alignment / highlight intent を返す

#### Scenario: agent message を表示する

- **WHEN** message role が assistant である
- **THEN** render model は streaming / complete / error status を表示できる

### Requirement: theme token と SVG icon override を提供しなければならない

システムは、host から theme token と SVG icon override を受け取り、button icon をすべて SVG asset id として扱わなければならない（MUST）。

#### Scenario: host が theme を渡す

- **WHEN** host が `ThemeTokens` を渡す
- **THEN** chat surface、user bubble、assistant bubble、border、accent、danger、warning、success の色が render model に反映される

#### Scenario: host が icon を上書きする

- **WHEN** host が `send` icon id に別 SVG を登録する
- **THEN** render model は default SVG ではなく override された SVG を参照する

### Requirement: context token と account usage の表示枠を持たなければならない

システムは、context token 使用量と account usage を表示できる render model を持たなければならない（MUST）。

#### Scenario: context usage が取得できる

- **WHEN** provider から used tokens と max tokens が渡される
- **THEN** UI は percentage と status を計算して表示できる

#### Scenario: account usage が取得できない

- **WHEN** provider が account usage を提供しない
- **THEN** UI は空白ではなく `Unavailable(reason)` として扱う
