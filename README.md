# actix-web-example

## 設定

### 環境変数

* 環境変数は、`.env`ファイルで設定
* `.env`ファイルは、リポジトリに存在しないため作成
* 環境変数`APP_ENVIRONMENT`からアプリケーションの動作環境を取得
  * 環境変数`APP_ENVIRONMENT`には、`development`、`production`を設定できそれぞれ開発環境と運用環境を表現

* `APP_ENVIRONMENT`: アプリケーションの動作環境を`development`または`production`で指定
* `POSTGRES_CONTAINER_NAME`: PostgreSQLのコンテナ名
* `POSTGRES_USER_NAME`: PostgreSQLのスパー・ユーザー名
* `POSTGRES_USER_PASSWORD`: 上記ユーザーのパスワード
* `POSTGRES_DATABASE_NAME`: 本アプリが使用するデータベースの名前
* `POSTGRES_PORT`: PostgreSQLコンテナにアクセスするときに使用するホスト側のポートの番号
* `POSTGRES_HOST`: PostgreSQLコンテナのホスト

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

## コンテナの起動

次の通り、コンテナを起動する。

```sh
./scripts/run_containers.sh
```

## テスト

次の通り、単体テストを実行する。

```sh
cargo test
```

次の通り、統合テストを実行する。

```sh
# ログ出力なし
cargo test -- --ignored
# ログ出力あり
TEST_LOG=true cargo test -- --ignored | bunyan  # cargo install bunyan
TEST_LOG=true cargo test -- --ignored | jq  # apt -y install jq
```

統合テストは、`test_suite`クレートに実装して、統合テストを実行する関数に次の通り属性をつける。

```rust
#[tokio::test]
#[ignore]
async fn test_async() {
    // 非同期テスト・コード
}

#[ignore]
fn test_sync() {
    // 同期テスト・コード
}
```
