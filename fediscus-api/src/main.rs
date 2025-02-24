use anyhow::Error;

mod config;
mod http_server;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = Config::load()?;

    http_server::HttpServer::new(config.http_server)
        .run()
        .await?;
    Ok(())
}
