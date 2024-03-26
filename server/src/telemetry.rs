use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// ログを購読するサブスクライバを生成する。
///
/// # 引数
///
/// * `name` - ログを購読するサブスクライバの名前
/// * `default_level` - デフォルトのログレベル
///
/// # 戻り値
///
/// ログを購読するサブスクライバ
pub fn generate_log_subscriber(name: String, default_level: &str) -> impl Subscriber {
    // ログをフィルタする条件を環境変数から取得
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    // ログを購読するサブスクライバを構築
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// ログを購読するサブスクライバを初期化する。
///
/// # 引数
///
/// * `subscriber` - ログを購読するサブスクライバ
pub fn init_log_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // すべての`log`のイベントをサブスクライバにリダイレクト
    LogTracer::init().expect("failed to set log tracer");
    // 上記サブスクライバをデフォルトに設定
    set_global_default(subscriber).expect("failed to set subscriber");
}
