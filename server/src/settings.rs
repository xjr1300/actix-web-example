use std::path::Path;

use config::{self, Config, FileFormat, FileSourceFile};
use enum_display::EnumDisplay;

/// 設定ファイル・ディレクトリ・パス
pub const SETTINGS_DIR_NAME: &str = "settings";

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
    /// ロギング設定
    pub logging: LoggingSettings,
}

/// HTTPサーバー設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HttpServerSettings {
    /// リスニング・ポート番号
    pub port: u16,
}

/// ロギング設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoggingSettings {
    /// ログ・レベル
    pub level: log::Level,
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
/// * `settings_dir` - 設定ファイル・ディレクトリ・パス
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

    use crate::settings::{retrieve_app_settings, AppEnvironment, SETTINGS_DIR_NAME};

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
    #[test]
    fn can_retrieve_app_settings_for_development() -> anyhow::Result<()> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let settings_dir = dir.join("..").join(SETTINGS_DIR_NAME);
        let app_settings = retrieve_app_settings(AppEnvironment::Development, settings_dir)?;
        assert_eq!(8000, app_settings.http_server.port);
        assert_eq!(log::Level::Debug, app_settings.logging.level);

        Ok(())
    }

    /// 運用環境のアプリケーション設定を正しくロードできることを確認
    #[test]
    fn can_retrieve_app_settings_for_production() -> anyhow::Result<()> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let settings_dir = dir.join("..").join(SETTINGS_DIR_NAME);
        let app_settings = retrieve_app_settings(AppEnvironment::Production, settings_dir)?;
        assert_eq!(443, app_settings.http_server.port);
        assert_eq!(log::Level::Info, app_settings.logging.level);

        Ok(())
    }
}
