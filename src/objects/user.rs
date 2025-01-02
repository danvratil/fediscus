use activitypub_federation::config::Data;
use activitypub_federation::protocol::verification;
use activitypub_federation::traits::Actor;
use activitypub_federation::{
    fetch::object_id::ObjectId, kinds::actor::PersonType, protocol::public_key::PublicKey,
    traits::Object,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error};
use url::Url;

use crate::FederationData;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum UserError {
    #[error("Couldn't parse URL")]
    UrlError(#[from] url::ParseError),
    #[error("SQL error")]
    SqlError(sqlx::Error),
    #[error("Verification error")]
    UrlVerificationError,
    #[error("Federation error")]
    FederationError(#[from] activitypub_federation::error::Error),
}

#[derive(Clone, Debug)]
/// Database type for the person/user object
pub struct DbPerson {
    pub id: ObjectId<DbPerson>,
    pub username: String,
    pub host: String,
    pub inbox: Url,
    pub outbox: Url,
    pub public_key: String,
    pub private_key: Option<String>,
    pub local: bool,
}

impl DbPerson {
    pub fn new(
        username: String,
        host: String,
        public_key: String,
        local: bool,
    ) -> Result<Self, UserError> {
        let id = Url::parse(&format!("https://{}/{}", host, username))?.into();
        let inbox = Url::parse(&format!("https://{}/{}/inbox", host, username))?;
        let outbox = Url::parse(&format!("https://{}/{}/outbox", host, username))?;
        Ok(Self {
            id,
            username,
            host,
            inbox,
            outbox,
            public_key,
            private_key: None,
            local,
        })
    }

    pub async fn read_from_name(
        name: &str,
        data: &Data<FederationData>,
    ) -> Result<Option<Self>, UserError> {
        let instance = &data.config.fediverse_user.host;
        let id_for_name = sqlx::query!(
            "SELECT uri FROM accounts WHERE username = ? AND host = ?",
            name,
            instance
        )
        .fetch_one(&data.db)
        .await
        .map_err(UserError::SqlError)?;

        DbPerson::read_from_id(Url::parse(&id_for_name.uri)?, data).await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// ActivityPub Person object representation - usually constructed from `DbPerson`.
pub struct Person {
    #[serde(rename = "type")]
    kind: PersonType,
    preferred_username: String,
    id: ObjectId<DbPerson>,
    inbox: Url,
    outbox: Url,
    public_key: PublicKey,
}

#[async_trait]
impl Object for DbPerson {
    type DataType = FederationData;
    type Kind = Person;
    type Error = UserError;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let db = &data.db;
        let id = object_id.as_str();
        let row = sqlx::query!(
            "SELECT username, host, inbox, outbox, private_key, public_key, local \
             FROM accounts \
             WHERE uri = ?",
            id
        )
        .fetch_one(db)
        .await;

        match row {
            Ok(row) => {
                // All local users must have a private key
                debug!(
                    "Found existing record for user {:?} in database",
                    object_id.as_str()
                );
                assert!(row.local == row.private_key.is_some());
                Ok(Some(DbPerson {
                    id: object_id.into(),
                    username: row.username,
                    host: row.host,
                    inbox: Url::parse(&row.inbox)?,
                    outbox: Url::parse(&row.outbox)?,
                    public_key: row.public_key,
                    private_key: row.private_key,
                    local: row.local,
                }))
            }
            Err(sqlx::Error::RowNotFound) => {
                debug!(
                    "No record found for user {:?} in database",
                    object_id.as_str()
                );
                Ok(None)
            }
            Err(e) => {
                error!("Error while reading user from database: {:?}", e);
                Err(UserError::SqlError(e))
            }
        }
    }

    async fn from_json(
        json: Self::Kind,
        _data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        Ok(DbPerson {
            id: json.id.clone(),
            username: json.preferred_username,
            host: json.id.inner().host_str().unwrap().to_string(),
            inbox: json.inbox,
            outbox: json.outbox,
            public_key: json.public_key.public_key_pem,
            private_key: None,
            local: false,
        })
    }

    async fn into_json(self, _: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(Person {
            kind: PersonType::Person,
            preferred_username: self.username.clone(),
            id: self.id.clone(),
            inbox: self.inbox(),
            outbox: self.outbox.clone(),
            public_key: PublicKey {
                id: format!("{}#main-key", self.id),
                owner: self.id.inner().clone(),
                public_key_pem: self.public_key,
            },
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verification::verify_domains_match(json.id.inner(), expected_domain)
            .map_err(|_| UserError::UrlVerificationError)
    }
}

impl Actor for DbPerson {
    fn id(&self) -> Url {
        self.id.inner().clone()
    }
    fn inbox(&self) -> Url {
        self.inbox.clone()
    }
    fn public_key_pem(&self) -> &str {
        &self.public_key
    }
    fn private_key_pem(&self) -> Option<String> {
        self.private_key.clone()
    }
}
