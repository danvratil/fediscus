use std::sync::Arc;

use activitypub_federation::config::FederationConfig;
use anyhow::Error;
use fediscus_activitypub::testing::MemoryStorage;
use fediscus_activitypub::ActivityPubService;
use fediscus_activitypub::Config;
use fediscus_activitypub::FederationData;
use fediscus_activitypub::HttpServer;
use fediscus_activitypub::Service;
use tokio::task::AbortHandle;

pub struct FediscusServer {
    pub service: Arc<Box<dyn ActivityPubService + Send + Sync + 'static>>,
    task: AbortHandle,
}

impl Drop for FediscusServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl FediscusServer {
    pub async fn new() -> Result<Self, Error> {
        let config = Config::load_from("tests/config.yaml")?;
        let storage = MemoryStorage::new("localhost:8086");
        let service =
            Arc::new(Box::new(Service::new(storage))
                as Box<dyn ActivityPubService + Send + Sync + 'static>);

        let federation_data = FederationData {
            config: config.clone(),
            service: Arc::clone(&service),
        };

        let federation = FederationConfig::builder()
            .domain(config.fediverse_user.host.clone())
            .app_data(federation_data)
            .debug(true)
            .allow_http_urls(true)
            .domain("localhost:8086")
            .build()
            .await?;

        let http_server = HttpServer::new(&config, federation).await?;
        let task = tokio::spawn(async { http_server.run().await });

        Ok(Self {
            service,
            task: task.abort_handle(),
        })
    }
}
