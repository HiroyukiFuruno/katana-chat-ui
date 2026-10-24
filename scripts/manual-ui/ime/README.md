# 実日本語IMEの隔離受入

`probe.py` は実IBus/Mozcの未確定・変換・確定・取消と、hostのdraft/action traceを照合する。単体の合成IMEイベントやUnicode貼付は使わない。最終文字列の検査と画像の目視は別々に確認する。

Linuxは既存のnative host buildを`kcu-v010-linux-target`へ準備した上で、Dockerfileのimageを`kcu-ime-linux-arm64:local`としてbuildし、repo直下から次を実行する。

```sh
rtk proxy python3 scripts/manual-ui/ime/run-background.py tmp/verification/ime-new-run
```

新しいartifact directoryだけに書込み、hostのDISPLAY/X socket・clipboard・networkを渡さない。専用uidのXvfb上で操作する。UTF-8 localeを必須とする。実行前後のJSONとnative bus log、各段階のapp画像・候補windowを含むscene画像を保存する。

macOSの`macos-probe.swift`はGitHub Actionsの専用runnerでだけ実行する。Release workflowの`verify_native_ime=true`は公開処理を無効にして診断だけを実行する。権限不足はexit77で未検証を記録する。ローカルではtypecheck/compileだけを行い、実行しない。

OCRは初期placeholderから本文行の位置を取得し、候補windowの別行を除いて本文を完全一致で検査する。純粋な行選択・重複拒否の単体検査は`macos-ocr.swift`と`test-macos-ocr.swift`を一緒にcompileして実行でき、アプリ・IME・入力ソースには触れない。native probeのtypecheck/compileには`macos-ocr.swift`と`macos-probe.swift`の両方を渡す。

CopilotのローカルBYOK受入はCopilot Freeのcloud quotaの証明ではなく、Linuxの実IME受入はmacOS固有の受入の代替ではない。
