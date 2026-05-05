---
name: lint-and-ast-lint
description: katana-chat-ui の静的検査（lint）と抽象構文木検査（AST lint）を厳格に扱い、警告ゼロ、抜け道禁止、設計で直す方針を徹底するスキル。
---

# Lint and AST Lint

静的検査（lint）と抽象構文木検査（AST lint）は、通過させる作業ではなく設計の品質 gate として扱う。
失敗したら、設定を緩めずにコードか設計を直す。

## 現在の入口

`Justfile` の入口を使う。

```bash
just check
just ast-lint
```

現在の `just check` は、format、Clippy、workspace test をまとめて確認する。
現在の `just ast-lint` は、少なくとも次の危険を検出する。

- production code の `unwrap` / `expect`
- `unwrap_or_default`
- `reqwest` の `.send().json()` で HTTP status を確認しない流れ
- request context を無視する実装

## 厳格化方針

- warning は失敗扱いにする。
- `allow`、exclude、検査条件の緩和を自己判断で追加しない。
- lint 失敗は、局所修正ではなく設計上の意図から直す。
- `unwrap`、`expect`、`unwrap_or_default` は、仕様化された境界以外で使わない。
- HTTP response は body parse 前に status を確認する。
- context、timeout、cancel、trace に相当する値を受け取った処理は、無視しない。
- 型を曖昧にして設計問題を隠さない。

## 失敗時の進め方

1. まず該当 command をそのまま実行し、失敗箇所を確認する。
2. 同じ repository 内の既存実装で、どう直しているかを探す。
3. ルールに準拠する実装へ修正する。
4. 準拠できない場合は、理由と代替案をユーザーへ相談する。
5. 修正後に同じ command を再実行する。

## 禁止

- `#[allow(...)]` の安易な追加。
- clippy.toml、Justfile、検査 script の緩和。
- grep 条件や対象 path を狭めて失敗を隠すこと。
- test や lint を通すためだけの fallback。
- `--no-verify` による品質 gate 回避。

## 検査不足を見つけた場合

現在の検査で拾えない危険な pattern を見つけた場合は、すぐに雑な手作業で済ませない。
どの危険を防ぎたいか、どの command に昇格するか、既存差分と分けるかを整理してユーザーに報告する。
