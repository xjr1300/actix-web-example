use std::path::Path;

use actix_web::cookie::SameSite;
use config::{Config, FileFormat, FileSourceFile};
use enum_display::EnumDisplay;
use log::LevelFilter;
use secrecy::{ExposeSecret as _, SecretString};
use serde::{Deserialize as _, Deserializer};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::{ConnectOptions as _, PgPool};

use use_cases::settings::{AuthorizationSettings, PasswordSettings};

/// 設定ファイルディレクトリ・パス
pub const SETTINGS_DIR_NAME: &str = "settings";

/// 動作環境を表現する環境変数とそのデフォルト値
pub const ENV_APP_ENVIRONMENT: &str = "APP_ENVIRONMENT";
pub const ENV_APP_ENVIRONMENT_DEFAULT: &str = "development";

/// アプリの動作環境
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDisplay)]
#[enum_display(case = "Lower")]
pub enum AppEnvironment {
    /// 開発環境
    Development,
    /// 運用環境
    Production,
}

impl From<String> for AppEnvironment {
    /// アプリの動作環境を表現する文字列から、アプリの動作環境を判定する。
    ///
    /// アプリの動作環境を表現する文字列が`development`の場合は開発環境、
    /// `production`の場合は運用環境と判定する。
    ///
    /// 上記以外の場合、開発環境と判定する。
    /// なお、大文字と小文字は無視する。
    ///
    /// # 引数
    ///
    /// * `value` - アプリの動作環境を表現する文字列
    ///
    /// # 戻り値
    ///
    /// アプリの動作環境
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "production" => Self::Production,
            _ => Self::Development,
        }
    }
}

/// アプリケーション設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppSettings {
    /// HTTPサーバー設定
    pub http_server: HttpServerSettings,
    /// パスワード設定
    pub password: PasswordSettings,
    /// 人s表設定
    pub authorization: AuthorizationSettings,
    /// データベース設定
    pub database: DatabaseSettings,
    /// ロギング設定
    pub logging: LoggingSettings,
}

/// HTTPサーバー設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HttpServerSettings {
    /// リスニングポート番号
    pub port: u16,
    /// アクセス及びリフレッシュトークンを保存するクッキーに付与するSameSite属性
    #[serde(deserialize_with = "deserialize_same_site")]
    pub same_site: SameSite,
    /// アクセス及びリフレッシュトークンを保存するクッキーにSecure属性を付けるか示すフラグ
    pub secure: bool,
}

fn deserialize_same_site<'de, D>(deserializer: D) -> Result<SameSite, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?.to_lowercase();
    match value.as_str() {
        "strict" => Ok(SameSite::Strict),
        "lax" => Ok(SameSite::Lax),
        "none" => Ok(SameSite::None),
        _ => Err(serde::de::Error::unknown_variant(
            &value,
            &["strict", "lax", "none"],
        )),
    }
}

/// データベース設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DatabaseSettings {
    /// ユーザー名
    pub user: String,
    /// パスワード
    pub password: SecretString,
    /// ポート番号
    pub port: u16,
    /// ホスト
    pub host: String,
    /// データベース名
    pub name: String,
    /// SSL接続要求
    pub require_ssl: bool,
    /// 接続タイムアウト秒
    pub connection_timeout_seconds: u64,
    /// ログに記録するSQLステートメントの最小レベル
    pub log_statements: LevelFilter,
}

/// ロギング設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoggingSettings {
    /// ログレベル
    pub level: log::Level,
}

impl DatabaseSettings {
    /// データベースを指定しない接続オプションを取得する。
    ///
    /// # 戻り値
    ///
    /// データベース接続オプション
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = match self.require_ssl {
            true => PgSslMode::Require,
            false => PgSslMode::Prefer,
        };
        PgConnectOptions::new()
            .username(&self.user)
            .password(self.password.expose_secret())
            .port(self.port)
            .host(&self.host)
            .ssl_mode(ssl_mode)
    }

    /// データベース接続オプションを取得する。
    ///
    /// # 戻り値
    ///
    /// データベース接続オプション
    pub fn with_db(&self) -> PgConnectOptions {
        let options = self.without_db().database(&self.name);
        options.log_statements(self.log_statements)
    }

    /// データベース接続プールを取得する。
    ///
    /// # 戻り値
    ///
    /// データベース接続プール
    pub fn connection_pool(&self) -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(
                self.connection_timeout_seconds,
            ))
            .connect_lazy_with(self.with_db())
    }
}

/// アプリケーション設定を取得する。
///
/// # 引数
///
/// * `app_env` - アプリケーションの動作環境
/// * `settings_dir` - アプリケーション設定ファイルを格納しているディレクトリのパス
///
/// # 戻り値
///
/// アプリケーション設定
pub fn retrieve_app_settings<P: AsRef<Path>>(
    app_env: AppEnvironment,
    settings_dir: P,
) -> anyhow::Result<AppSettings> {
    // デフォルト及び動作環境別設定ファイルのパスを生成
    let settings_dir = settings_dir.as_ref();
    let default_settings_file = config_file_source(settings_dir, "default.yml");
    let env_settings_file = config_file_source(settings_dir, &format!("{app_env}.yml"));

    // アプリケーション設定のビルダーを構築
    let settings = Config::builder()
        // デフォルトの設定ファイルをロード
        .add_source(default_settings_file)
        // 環境別の設定ファイルをロード
        .add_source(env_settings_file)
        // 環境変数に記録された設定をロード
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .add_source(
            config::Environment::with_prefix("POSTGRES")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    // アプリケーション設定を読み込み
    settings
        .try_deserialize::<AppSettings>()
        .map_err(|e| e.into())
}

/// `Config`がロードする設定ファイルのパスを構築する。
///
/// # 引数
///
/// * `settings_dir` - 設定ファイルディレクトリ・パス
/// * `file_name` - 設定ファイルの名前
///
/// # 戻り値
///
/// 設定ファイルのパス
fn config_file_source(
    settings_dir: &Path,
    file_name: &str,
) -> config::File<FileSourceFile, FileFormat> {
    config::File::from(settings_dir.join(file_name))
}

#[cfg(test)]
pub mod tests {
    use std::path::Path;

    use log::LevelFilter;
    use secrecy::ExposeSecret;

    use crate::settings::{
        retrieve_app_settings, AppEnvironment, DatabaseSettings, SETTINGS_DIR_NAME,
    };

    /// 文字列からアプリの動作環境を正しく判定できることを確認
    #[test]
    fn can_retrieve_app_environment_from_strings() {
        let candidates = [
            (AppEnvironment::Development, "develop"),
            (AppEnvironment::Development, "DEVELOP"),
            (AppEnvironment::Production, "production"),
            (AppEnvironment::Production, "PRODUCTION"),
            (AppEnvironment::Development, ""),
            (AppEnvironment::Development, "foobar"),
        ];
        for (expected, candidate) in candidates {
            let environment: AppEnvironment = candidate.to_string().into();
            assert_eq!(expected, environment);
        }
    }

    /// 開発環境のアプリケーション設定を正しくロードできることを確認
    ///
    /// ワークスペースディレクトリ内の`.env`ファイルが存在することを想定している。
    #[test]
    fn can_retrieve_app_settings_for_development() -> anyhow::Result<()> {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let env_file = crate_dir.join("..").join(".env");
        dotenvx::from_path(env_file)?;

        let settings_dir = crate_dir.join("..").join(SETTINGS_DIR_NAME);
        let app_settings = retrieve_app_settings(AppEnvironment::Development, settings_dir)?;
        assert_eq!(8000, app_settings.http_server.port);
        assert_eq!(
            "very-long-and-complex-string",
            app_settings.password.pepper.expose_secret()
        );
        validate_database_settings(&app_settings.database);
        assert!(!app_settings.database.require_ssl); // SSL接続を要求しない
        assert_eq!(LevelFilter::Trace, app_settings.database.log_statements);
        assert_eq!(log::Level::Debug, app_settings.logging.level);

        Ok(())
    }

    /// 運用環境のアプリケーション設定を正しくロードできることを確認
    ///
    /// ワークスペースディレクトリ内の`.env`ファイルが存在することを想定している。
    #[test]
    fn can_retrieve_app_settings_for_production() -> anyhow::Result<()> {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let env_file = crate_dir.join("..").join(".env");
        println!("enf_file: {}", env_file.display());
        dotenvx::from_path(env_file)?;

        let settings_dir = crate_dir.join("..").join(SETTINGS_DIR_NAME);
        let app_settings = retrieve_app_settings(AppEnvironment::Production, settings_dir)?;
        assert_eq!(443, app_settings.http_server.port);
        assert_eq!(
            "very-long-and-complex-string",
            app_settings.password.pepper.expose_secret()
        );
        validate_database_settings(&app_settings.database);
        assert!(app_settings.database.require_ssl); // SSL接続を要求
        assert_eq!(LevelFilter::Error, app_settings.database.log_statements);
        assert_eq!(log::Level::Info, app_settings.logging.level);

        Ok(())
    }

    /// データベース設定を正しくロードできていることを確認
    fn validate_database_settings(settings: &DatabaseSettings) {
        assert_eq!("awe", settings.user);
        assert_eq!("awe-pass", settings.password.expose_secret());
        assert_eq!(5432, settings.port);
        assert_eq!("localhost", settings.host);
        assert_eq!("awe", settings.name);
        assert_eq!(3, settings.connection_timeout_seconds);
    }
}
