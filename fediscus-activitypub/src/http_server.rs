use anyhow::Error;
use axum::routing::{get, post};
use tower_http::trace::TraceLayer;
use std::sync::Arc;

use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use tokio::net::TcpListener;
use tracing::info;

use crate::{config::Config, AppState, FederationData};

mod handlers;

pub struct HttpServer {
    app_state: Arc<AppState>,
}

impl HttpServer {
    pub async fn new(config: &Config, federation: FederationConfig<FederationData>) -> Result<Self, Error> {
        Ok(Self {
            app_state: Arc::new(AppState {
                config: config.clone(),
                federation,
            })
        })
    }

    pub async fn run(self) -> Result<(), Error> {
        let listener = TcpListener::bind(self.app_state.config.http_server.listen.clone()).await.unwrap();
        info!("Listening on: {}", listener.local_addr().unwrap());

        let service = axum::Router::new()
            .route("/users/:name", get(handlers::get_user))
            .route("/users/:name/inbox", post(handlers::post_inbox))
            .route("/users/:name/outbox", get(handlers::get_outbox))
            .route("/.well-known/webfinger", get(handlers::get_webfinger))
            .route_layer(FederationMiddleware::new(self.app_state.federation.clone()))
            .layer(TraceLayer::new_for_http())
            .with_state(self.app_state);

        axum::serve(listener, service).await?;
        Ok(())
    }
}
