// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::FederationConfig;
use anyhow::Error;
use fediscus_activitypub::ActivityPubService;
use std::sync::Arc;

use fediscus_activitypub::db;
use fediscus_activitypub::Config;
use fediscus_activitypub::FederationData;
use fediscus_activitypub::HttpServer;
use fediscus_activitypub::Service;
use fediscus_activitypub::SqliteStorage;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = Config::load()?;
    {
        let db = sqlx::SqlitePool::connect(&config.database.url).await?;
        db::init_local_user(&db, &config.fediverse_user).await?;
    }

    let storage = SqliteStorage::new(&config.database).await?;
    let service = Arc::new(
        Box::new(Service::new(storage)) as Box<dyn ActivityPubService + Send + Sync + 'static>
    );

    let federation_data = FederationData {
        config: config.clone(),
        service: Arc::clone(&service),
    };

    let federation = FederationConfig::builder()
        .domain(config.fediverse_user.host.clone())
        .app_data(federation_data)
        .build()
        .await?;

    let http_server = HttpServer::new(&config, federation).await?;
    http_server.run().await
}
