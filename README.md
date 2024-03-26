# actix-web-example

## ログの記録

* `tracing`クレート及びそれに関連するクレートを利用してログを記録
  * `tracing`: スコープを持ち、構造化され、イベントに基づく診断情報を収集するフレームワーク
  * `tracing-actix-web`: `actix-web`のリクエスト/レスポンスのログを記録するミドルウェア
  * `tracing-bunyan-formatter`: Bunyanフォーマットでログを整形するフォーマッタ
  * `tracing-log`: `log`クレートが提供するロギング・ファサードと一緒に`tracing`を使用するための互換レイヤを提供
  * `tracing-subscriber`: `tracing`の購読者を実装または構成するユーティリティ
