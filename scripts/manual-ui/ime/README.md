# 実日本語IMEの隔離受入

`probe.py` は実IBus/Mozcの未確定・変換・確定・取消と、hostのdraft/action traceを照合する。単体の合成IMEイベントやUnicode貼付は使わない。最終文字列の検査と画像の目視は別々に確認する。

Linuxは既存のnative host buildを`kcu-v010-linux-target`へ準備した上で、Dockerfileのimageを`kcu-ime-linux-arm64:local`としてbuildし、repo直下から次を実行する。

```sh
rtk proxy python3 scripts/manual-ui/ime/run-background.py tmp/verification/ime-new-run
```

新しいartifact directoryだけに書込み、hostのDISPLAY/X socket・clipboard・networkを渡さない。専用uidのXvfb上で操作する。UTF-8 localeを必須とする。実行前後のJSONとnative bus log、各段階のapp画像・候補windowを含むscene画像を保存する。

macOSの`macos-probe.swift`はGitHub Actionsの専用runnerでだけ実行する。Release workflowの`verify_native_ime=true`は公開処理を無効にして診断だけを実行する。権限不足はexit77で未検証を記録する。ローカルではtypecheck/compileだけを行い、実行しない。

OCRは初期placeholderから本文行の位置を取得し、候補windowの別行を除いて本文を完全一致で検査する。純粋な行選択・重複拒否の単体検査は`macos-ocr.swift`と`test-macos-ocr.swift`を一緒にcompileして実行でき、アプリ・IME・入力ソースには触れない。native probeのtypecheck/compileには`macos-ocr.swift`と`macos-probe.swift`の両方を渡す。

`macos-reference.swift`は標準`NSTextView`を同じrunner・入力列で操作する比較器。実際の`setMarkedText`・`insertText`・`unmarkText`を記録し、runner/入力手順とKCU adapterの境界を切り分ける。比較器の成功をKCUの受入として数えない。これもローカルでは起動せず、CIで3種類のKCU hostの後に実行する。

比較器は`macos-ime-state.swift`と一緒にcompileする。実際の本文とmarked状態を受動観測し、同じ確定状態が連続して確認された時だけ記録する判定を`test-macos-ime-state.swift`で単体検査する。この単体検査はAppKitを起動しない。

macOSでは入力欄へfocusしてから日本語入力方式を選択する。追加の「かな」キーで制御文字が挿入された実測があるため、通常手順では送らず、`KCU_IME_ACTIVATION_KEY=kana`の標準AppKit比較対照だけに残す。`nihongo`の未確定表示、Spaceによる変換、Returnによる確定、続く`tesuto`のEscape 2回による取消を検査する。送信key codeと時刻、標準入力欄のmarked状態も記録する。日本語のUnicode文字列を直接入力してIME成功に見せる処理は使わない。

CopilotのローカルBYOK受入はCopilot Freeのcloud quotaの証明ではなく、Linuxの実IME受入はmacOS固有の受入の代替ではない。

macOS診断では、同じ標準NSTextViewについて瞬時のdown/upと50ms押下を比較する。押下時間は入力操作の条件であり、画面準備の固定待ちやtimeout延長ではない。各keyのdown/up時刻を保存し、本文完全一致・確定一回・未確定draft非反映・誤送信なしの判定は共通とする。瞬時tapの失敗も保持し、成功に置き換えない。

押下時間はThreadの待機で維持し、送出直前の単調時計から実測する。RunLoopの早期returnや、生成直後のCGEvent timestampが0のままになる場合を成功証拠に使わない。実測が指定押下時間を下回った場合は検証を失敗にする。

`ime_reference`は同じeframe依存の標準TextEdit比較器で、実ImeCommitとraw Textを区別して記録する。TextEditの内部bufferへpreeditが入ることをKCUの確定draftと混同しない。比較器の成功はKCU hostの合格ではない。通常の`cargo run`は`default-run`で従来のmanual-host-eguiを起動する。
