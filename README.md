# actix-web-example

## 参考文献

* [Zero To Production In Rust](https://www.zero2prod.com/)
* [proc-macro-workshop/builderをやってみる](https://blog.ymgyt.io/entry/proc-macro-workshop-builder/)
* [RustでClean Architectureを実装してみる](https://zenn.dev/htlsne/articles/rust-clean-architecture)

## 設定

### 環境変数

* 環境変数は、`.env`ファイルで設定
* `.env`ファイルは、リポジトリに存在しないため作成
* 環境変数`APP_ENVIRONMENT`からアプリケーションの動作環境を取得
  * 環境変数`APP_ENVIRONMENT`には、`development`、`production`を設定できそれぞれ開発環境と運用環境を表現

#### アプリケーション設定

* `APP_ENVIRONMENT`: アプリケーションの動作環境を`development`または`production`で指定
* `APP_AUTHORIZATION__JWT_TOKEN_SECRET`: JWTトークンを生成するときの秘密鍵
* `APP_PASSWORD__PEPPER`: パスワードをハッシュ化する前に、パスワードに追加する文字列

#### データベース設定

* `POSTGRES_CONTAINER`: PostgreSQLのコンテナ名
* `POSTGRES_DATABASE__USER`: PostgreSQLのスーパーユーザーの名前
* `POSTGRES_DATABASE__PASSWORD`: 上記ユーザーのパスワード
* `POSTGRES_DATABASE__PORT`: PostgreSQLコンテナに接続するホスト側のポートの番号
* `POSTGRES_DATABASE__HOST`: PostgreSQLコンテナに接続するホストの名前
* `DATABASE_URL`: PostgreSQLの接続URL

#### Redis設定

* `REDIS_CONTAINER`: Redisのコンテナ名

### 設定ファイル

* `settings`ディレクトリの`default.yml`からアプリケーションの設定を読み込む
* 次に、アプリケーションの動作環境が開発環境であれば`settings`ディレクトリの`development.yml`を、
  運用環境であれば`production.yml`を読み込み、`default.yml`に定義された設定を上書き

* `http_server`: Httpサーバー設定
  * `port`: HTTPサーバーがリッスンするポートの番号
  * `sign_in_attempting_seconds`: ユーザーがサインインを試行する期間（秒）
  * `number_of_sign_in_failures`: ユーザーのアカウントをロックするまでの失敗回数
  * `access_token_seconds`: アクセストークンの有効期限（秒）
  * `refresh_token_seconds`: リフレッシュトークンの有効期限（秒）
  * `same_site`: クッキーの`SameSite`属性（`strict`または`lax`）
  * `secure`: クッキーの`Secure`属性（`true`または`false`）
* `password`: パスワード設定
  * `hash_memory`: パスワードをハッシュ化するときのメモリサイズ
  * `hash_iterations`: パスワードをハッシュ化するときの反復回数
  * `hash_parallelism`: パスワードをハッシュ化するときの並列度
* `authorization`: 認証設定
  * `attempting_seconds`: ユーザーがサインインを試行する期間（秒）
  * `number_of_failures`: ユーザーのアカウントをロックするまでの失敗回数
  * `access_token_seconds`: アクセストークンの有効期限（秒）
  * `refresh_token_seconds`: リフレッシュトークンの有効期限（秒）
* `database`: データベース設定
  * `require_ssl`: SSL接続を要求するかどうか(`true`, `false`)
  * `log_statements`: ログに記録するSQLステートメントの最小レベル(`debug`, `info`, `warn`, `error`)
* `logging`: ロギング設定
  * `level`: ロギングレベル（`trace`, `debug`, `info`, `warn`, `error`）

## 認証

* ユーザーをユーザーのEメールアドレスとパスワードで認証
* パスワードは、環境変数に設定されたペッパーと、ユーザーごとのソルトを付与したユーザーが設定したパスワードを、ハッシュ化して保存
  * パスワードをハッシュ化するアルゴリズムに`Argon2id`を使用
  * パスワードをハッシュ化するときのメモリサイズ、反復回数、並列度は設定ファイルから取得
  * 生成されるハッシュ値の長さはデフォルトの32byte
  * パスワードをハッシュ化するときの推奨値は[OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id)を参照
* ユーザーが認証に成功した場合、アクセストークンとリフレッシュ・トークンを返す
* また同時に、アクセストークンとリフレッシュ・トークンを、名前をそれぞれ`access`と`refresh`としてクッキーに保存する`Set-Cookie`ヘッダを返す
  * `SameSite`属性に設定ファイルの値を設定（`Strict`または`Lax`）
  * `Secure`属性を設定ファイルに従って設定
  * `HttpOnly`属性を設定
* ユーザーが`authorization`の`attempting_seconds`時間内に`number_of_failures`回以上認証に失敗した場合、アカウントをロック

## ログの記録

* `tracing`クレート及びそれに関連するクレートを利用してログを記録
  * `tracing`: スコープを持ち、構造化され、イベントに基づく診断情報を収集するフレームワーク
  * `tracing-actix-web`: `actix-web`のリクエスト/レスポンスのログを記録するミドルウェア
  * `tracing-bunyan-formatter`: Bunyanフォーマットでログを整形するフォーマッタ
  * `tracing-log`: `log`クレートが提供するロギングファサードと一緒に`tracing`を使用するための互換レイヤを提供
  * `tracing-subscriber`: `tracing`の購読者を実装または構成するユーティリティ

## リクエストとレスポンスの処理

### ユースケース層でデータを加工する必要がない場合

* ドメイン層: FooInput、FooOutput
* ユースケース層: FooUseCaseInput、FooUseCaseOutput
* インフラストラクチャ層: FooReqBody、FooResBody

* インフラストラクチャ層でリクエストボディとして受け取るデータの型(`FooReqBody`)と、レスポンス・ボディとして返すデータの型(`FooResBody`)を定義
* ドメイン層でリポジトリが受け取るデータの型(`FooInput`)と、返すデータの型(`FooOutput`)を定義
* インフラストラクチャ層は、クライアントから受け取ったリクエストボディを、`FooReqBody`に変換
  * 変換時に検証に失敗した場合は、適切なエラーを返す
* インフラストラクチャ層は、`FooReqBody`をリポジトリが扱う`FooInput`に変換して、ユースケース層に渡す
  * 変換時に検証に失敗した場合は、適切なエラーを返す
* ユースケース層は、`FooInput`でリポジトリを操作して、リポジトリから操作した結果を`FooOutput`として受け取る
  * リポジトリの操作に失敗した場合は、適切なエラーを返す
* ユースケース層は、`FooOutput`をインフラ・ストラクチャ層に返す
* インフラストラクチャ層は、`FooOutput`を`FooResBody`に変換して、クライアントに返す

### ユースケース層でデータを加工する必要がある場合

* インフラストラクチャ層でリクエストボディとして受け取るデータの型(`FooReqBody`)と、レスポンス・ボディとして返すデータの型(`FooResBody`)を定義
* ユースケース層でインフラストラクチャ層から受け取るデータの型(`FooUseCaseInput`)と、インフラストラクチャ層に返すデータの型(`FooUseCaseOutput`)を定義
* ドメイン層でリポジトリが受け取るデータの型(`FooInput`)と、返すデータの型(`FooOutput`)を定義
* インフラストラクチャ層は、クライアントから受け取ったリクエストボディを、`FooReqBody`に変換
  * 変換時に検証に失敗した場合は、適切なエラーを返す
* インフラストラクチャ層は、`FooReqBody`を、ユースケース層が扱う`FooUseCaseInput`に変換して、ユース・ケース層に渡す
  * 変換時に検証に失敗した場合は、適切なエラーを返す
* ユースケース層は、`FooUseCaseInput`を操作した後、リポジトリに渡す`FooInput`を生成してリポジトリを操作して、リポジトリから操作した結果を`FooOutput`として受け取る
  * `FooUseCaseInput`の操作に失敗した場合は、適切なエラーを返す
  * リポジトリの操作に失敗した場合は、適切なエラーを返す
* ユースケース層は、`FooOutput`を操作した後、`FooUseCaseOutput`を生成してインフラストラクチャ層に返す
* インフラストラクチャ層は、 `FooUseCaseOutput`を`FooResBody`に変換してクライアントに返す

## コンテナの起動

次の通り、コンテナを起動する。

```sh
./scripts/run_containers.sh
```

## テスト

### 単体テスト

* ワークスペースディレクトリの`.env`ファイルが必要な場合はバックアップ
* ワークスペースディレクトリの`.env.test`ファイルを、`.env`ファイルに名前を変更

次の通り、単体テストを実行する。

```sh
cargo test
```

#### 統合テスト

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
    // 非同期テストコード
}

#[ignore]
fn test_sync() {
    // 同期テストコード
}
```

## クレートの脆弱性調査

```sh
# インストール
cargo install cargo-audit --features=fix
# 脆弱性調査
cargo audit
# [experimental] 脆弱性のあるクレートのアップデート
cargo audit fix
cargo audit fix --dry-run
```

## テーブルの制約名の形式

| 制約の種類               | 制約名の形式                          | 備考          |
| ------------------------ | ------------------------------------- | ------------- |
| 主キー制約               | `pk_<table-name>`                     | Primary key   |
| ユニークインデックス制約 | `ak_<table-name>-<field>[_<field>..]` | Alternate key |
| インデックス制約         | `ix_<table-name>-<field>[_<field>..]` | Index         |
| 外部キー制約             | `fk_<table-name>-<relationship>`      | Foreign key   |
| チェック制約             | `ck_<table-name>-<content>`           | Check         |

* `relationship`には、関連の説明を記述
* `content`には、チェック制約の内容を記述
