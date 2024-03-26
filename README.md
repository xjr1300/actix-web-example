# actix-web-example

## 設定

### 環境変数

* 環境変数`APP_ENVIRONMENT`からアプリケーションの動作環境を取得
* 環境変数`APP_ENVIRONMENT`には、`development`、`production`を設定できそれぞれ開発環境と運用環境を表現

### アプリケーション設定

* `settings`ディレクトリの`default.yml`からアプリケーションの設定を読み込む
* 次に、アプリケーションの動作環境が開発環境であれば`settings`ディレクトリの`development.yml`を、
  運用環境であれば`production.yml`を読み込み、`default.yml`に定義された設定を上書き

## ログの記録

* `tracing`クレート及びそれに関連するクレートを利用してログを記録
  * `tracing`: スコープを持ち、構造化され、イベントに基づく診断情報を収集するフレームワーク
  * `tracing-actix-web`: `actix-web`のリクエスト/レスポンスのログを記録するミドルウェア
  * `tracing-bunyan-formatter`: Bunyanフォーマットでログを整形するフォーマッタ
  * `tracing-log`: `log`クレートが提供するロギング・ファサードと一緒に`tracing`を使用するための互換レイヤを提供
  * `tracing-subscriber`: `tracing`の購読者を実装または構成するユーティリティ
