use crate::testing::server::{
    error::Error,
    http,
    objects::{DbUser, DbPost},
};
use activitypub_federation::config::{FederationConfig, UrlVerifier};
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use url::Url;

// RSA keys for testing purposes - generating new random keypair for each test run
// is fairly slow, so we use hardcoded static keys here. DO NOT USE THEM ANYWHERE!
static SYSTEM_USER_RSA_PRIVATE_KEY: &str = include_str!("data/system_private_key.pem");
static SYSTEM_USER_RSA_PUBLIC_KEY: &str = include_str!("data/system_public_key.pem");
static LOCAL_USER_RSA_PRIVATE_KEY: &str = include_str!("data/local_private_key.pem");
static LOCAL_USER_RSA_PUBLIC_KEY: &str = include_str!("data/local_public_key.pem");

pub async fn new_instance(
    hostname: &str,
    name: String,
) -> Result<FederationConfig<DatabaseHandle>, Error> {
    let mut system_user: DbUser = DbUser::new(
        hostname,
        "system".into(),
        SYSTEM_USER_RSA_PRIVATE_KEY,
        SYSTEM_USER_RSA_PUBLIC_KEY,
    )?;
    system_user.ap_id = Url::parse(&format!("http://{}/", hostname))?.into();

    let local_user = DbUser::new(
        hostname,
        name,
        LOCAL_USER_RSA_PRIVATE_KEY,
        LOCAL_USER_RSA_PUBLIC_KEY,
    )?;
    let database = Arc::new(Database {
        system_user: system_user.clone(),
        users: Mutex::new(vec![local_user]),
        posts: Mutex::new(vec![]),
    });
    let config = FederationConfig::builder()
        .domain(hostname)
        .signed_fetch_actor(&system_user)
        .app_data(database)
        .url_verifier(Box::new(MyUrlVerifier()))
        .debug(true)
        .build()
        .await?;
    Ok(config)
}

pub type DatabaseHandle = Arc<Database>;

/// Our "database" which contains all known posts and users (local and federated)
pub struct Database {
    pub system_user: DbUser,
    pub users: Mutex<Vec<DbUser>>,
    pub posts: Mutex<Vec<DbPost>>,
}

/// Use this to store your federation blocklist, or a database connection needed to retrieve it.
#[derive(Clone)]
struct MyUrlVerifier();

#[async_trait]
impl UrlVerifier for MyUrlVerifier {
    async fn verify(&self, url: &Url) -> Result<(), activitypub_federation::error::Error> {
        if url.domain() == Some("malicious.com") {
            Err(activitypub_federation::error::Error::Other(
                "malicious domain".into(),
            ))
        } else {
            Ok(())
        }
    }
}

pub fn listen(config: &FederationConfig<DatabaseHandle>) -> Result<(), Error> {
    http::listen(config)
}

impl Database {
    pub fn local_user(&self) -> DbUser {
        let lock = self.users.lock().unwrap();
        lock.first().unwrap().clone()
    }

    pub fn read_user(&self, name: &str) -> Result<DbUser, Error> {
        let db_user = self.local_user();
        if name == db_user.name {
            Ok(db_user)
        } else {
            Err(anyhow!("Invalid user {name}").into())
        }
    }
}
