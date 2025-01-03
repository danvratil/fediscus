// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use anyhow::{Error, anyhow};
use axum::routing::{get, post};
use storage::Storage;
use tower_http::trace::TraceLayer;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{self, info};

mod config;
mod db;
mod handlers;
mod types;
mod activities;
mod storage;
mod apub;
mod sqlite;

use crate::config::Config;
use crate::sqlite::SqliteStorage;

pub struct AppState {
    pub config: Config,
    pub storage: Arc<Box<dyn Storage + Send + Sync + 'static>>,
    pub federation: FederationConfig<FederationData>,
}

#[derive(Clone)]
pub struct FederationData {
    pub config: Config,
    pub storage: Arc<Box<dyn Storage + Send + Sync + 'static>>,
}

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

    let storage = Arc::new(Box::new(SqliteStorage::new(&config.database).await?) as Box<dyn Storage + Send + Sync>);

    let federation_data = FederationData {
        config: config.clone(),
        storage: Arc::clone(&storage),
    };

    let federation = FederationConfig::builder()
        .domain(config.fediverse_user.host.clone())
        .app_data(federation_data)
        .build()
        .await?;


    let listener = TcpListener::bind(config.http_server.listen.clone()).await.unwrap();
    info!("Listening on: {}", listener.local_addr().unwrap());

    let app_state = Arc::new(AppState {
        config,
        storage: Arc::clone(&storage),
        federation: federation.clone(),
    });
    let service = axum::Router::new()
        .route("/users/:name", get(handlers::federation::get_user))
        .route("/users/:name/inbox", post(handlers::federation::post_inbox))
        .route("/users/:name/outbox", get(handlers::federation::get_outbox))
        .route("/.well-known/webfinger", get(handlers::federation::get_webfinger))
        .route_layer(FederationMiddleware::new(federation))
        .route("/api/v1/comments", post(handlers::api::post_comments))
        .route("/api/v1/count", post(handlers::api::get_count))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    axum::serve(listener, service).await.map_err(|e| anyhow!(e))
}
