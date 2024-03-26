use std::path::Path;

use server::settings::SETTINGS_DIR_NAME;
use server::startup::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings_dir = Path::new(SETTINGS_DIR_NAME);

    run(settings_dir).await?.await.map_err(|e| e.into())
}
