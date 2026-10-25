# Floem native raster backend

## なぜ必要か

Native21では、control labelが見えていてもKCU raster内容が消える状態が発生した。取得されたraster viewportも1280x900から1024x642へ変化した。これはrenderer/backendの受入失敗であり、共通KUC frameや3つのvendor adapterが誤っている証拠ではない。

公開Vger packing結果は6 frameまでは再現し、その後`None`になる。測定usageは期待閾値未満（`0.61 < 0.7`）である。既存記録は診断結果に限られ、Native受入成功ではない。

## 変更内容

- production adapterのCargo featureで公式Floem Vello rendererをdefaultにする。既存TinySkia fallbackは明示的かつサポート対象として残す。
- 3つのvendor adapterと公開KUC frame/APIを維持し、rendererのforkやpath固定は行わない。
- control labelの可視性と有効なKCU raster frameを区別する、反復composition/image受入契約を追加する。
- 最新のNative text-input callback（recordIndex/uptime）を取得し、入力tokenをcapture前後で変えずにcanonical RGBA body sampleを2点取得する。

## 対象外

- KCU frame、adapter、公開API、model callbackの変更は行わない。
- 共通KUC仕様にVello固有動作を規定しない。
- Native22、Linux Floem、GPUI、public-consumer、app、Storybookの未取得証拠をwaiveしない。
