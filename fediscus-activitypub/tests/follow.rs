use std::sync::Arc;

use activitypub_federation::config::FederationConfig;
use anyhow::Error;
use fediscus_activitypub::testing::server::{listen, new_instance};
use fediscus_activitypub::testing::MemoryStorage;
use fediscus_activitypub::ActivityPubService;
use fediscus_activitypub::Config;
use fediscus_activitypub::FederationData;
use fediscus_activitypub::HttpServer;
use fediscus_activitypub::Service;
use serial_test::serial;
use tokio::task::AbortHandle;
use tracing::info;
use url::Url;

struct FediscusServer {
    service: Arc<Box<dyn ActivityPubService + Send + Sync + 'static>>,
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

#[tokio::test]
#[serial]
async fn test_handle_follow_request() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let fediscus = FediscusServer::new()
        .await
        .expect("Failed to start Fediscus server");
    info!("Fediscus server started");
    let fediscus_user = fediscus
        .service
        .storage()
        .get_local_account()
        .await
        .expect("Failed to retrieve local account");

    let test_server = new_instance("localhost:8087", "testuser".to_string())
        .await
        .expect("Failed to start test server");
    listen(&test_server).expect("Failed to start test server");
    info!("Test server listening");

    // The test users follows fediscus
    test_server
        .local_user()
        .follow("fediscus@localhost:8086", &test_server.to_request_data())
        .await
        .expect("Failed to follow Fediscus");

    // Result:
    // Fediscus is now aware of the test user
    let test_user = fediscus
        .service
        .storage()
        .account_by_uri(&Url::parse("http://localhost:8087/testuser").unwrap().into())
        .await
        .expect("Failed to retrieve users")
        .expect("Test user does not exist in local database");

    // Fediscus is aware of the follow relationship
    let follow = fediscus
        .service
        .storage()
        .follow_by_ids(test_user.id, fediscus_user.id)
        .await
        .expect("Failed to retrieve follow relation")
        .expect("Follow relation from test user to fediscus does not exist in local database");
    assert!(!follow.pending);

    // Fediscus followed the test user back - the relationship should be known to it
    let backfollow = fediscus
        .service
        .storage()
        .follow_by_ids(fediscus_user.id, test_user.id)
        .await
        .expect("Failed to retrieve back-follow relation")
        .expect("Back-follow relation from fediscus to test user does not exist in local database");
    // And it should have been accepted by the test server
    assert!(!backfollow.pending);

    // The test user should also be aware that its being followed by Fediscus
    assert!(test_server
        .local_user()
        .followers
        .iter()
        .any(|f| f == &Url::parse("http://localhost:8086/users/fediscus").unwrap()));
}

#[tokio::test]
#[serial]
async fn test_handle_unfollow() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let fediscus = FediscusServer::new()
        .await
        .expect("Failed to start Fediscus server");
    info!("Fediscus server started");
    let fediscus_user = fediscus
        .service
        .storage()
        .get_local_account()
        .await
        .expect("Failed to retrieve local account");

    let test_server = new_instance("localhost:8087", "testuser".to_string())
        .await
        .expect("Failed to start test server");
    listen(&test_server).expect("Failed to start test server");
    info!("Test server listening");

    // The test users follows fediscus
    let follow_id = test_server
        .local_user()
        .follow("fediscus@localhost:8086", &test_server.to_request_data())
        .await
        .expect("Failed to follow Fediscus");

    // Confirm that the follow relationship exists (in both directions)
    let test_user = fediscus
        .service
        .storage()
        .account_by_uri(&Url::parse("http://localhost:8087/testuser").unwrap().into())
        .await
        .expect("Failed to retrieve users")
        .expect("Test user does not exist in local database");
    fediscus
        .service
        .storage()
        .follow_by_ids(test_user.id, fediscus_user.id)
        .await
        .expect("Failed to retrieve follow relation")
        .expect("Follow relation from test user to fediscus does not exist in local database");

    // The test user unfollows fediscus
    test_server
        .local_user()
        .unfollow(
            "fediscus@localhost:8086",
            &follow_id,
            &test_server.to_request_data(),
        )
        .await
        .expect("Failed to unfollow Fediscus");

    // Result:
    let test_user = fediscus
        .service
        .storage()
        .account_by_uri(&Url::parse("http://localhost:8087/testuser").unwrap().into())
        .await
        .expect("Failed to retrieve users")
        .expect("Test user does not exist in local database");

    // Fediscus is no longer aware of the test user following us
    assert!(fediscus
        .service
        .storage()
        .follow_by_ids(test_user.id, fediscus_user.id)
        .await
        .expect("Failed to retrieve follow relation")
        .is_none());
    // Fediscus is no longer following the test user (backfollow was canceled)
    assert!(fediscus
        .service
        .storage()
        .follow_by_ids(fediscus_user.id, test_user.id)
        .await
        .expect("Failed to retrieve back-follow relation")
        .is_none());

    // The test user is no longer being followed by Fediscus
    assert!(test_server.local_user().followers.is_empty());
}
