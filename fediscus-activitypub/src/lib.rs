use std::sync::Arc;

use activitypub_federation::config::FederationConfig;

mod activities;
mod apub;
mod config;
pub mod db;
mod http_server;
mod service;
mod sqlite;
mod storage;
pub mod testing;
mod types;

pub struct AppState {
    pub config: Config,
    pub federation: FederationConfig<FederationData>,
}

#[derive(Clone)]
pub struct FederationData {
    pub config: Config,
    pub service: Arc<Box<dyn ActivityPubService + Send + Sync + 'static>>,
    //pub storage: Arc<Box<dyn Storage + Send + Sync + 'static>>,
}

pub use config::Config;
pub use http_server::HttpServer;
pub use service::ActivityPubService;
pub use service::Service;
pub use sqlite::SqliteStorage; // FIXME: this leaks abstraction
