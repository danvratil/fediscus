// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::Data;
use activitypub_federation::protocol::public_key::PublicKey;
use activitypub_federation::protocol::verification;
use activitypub_federation::traits::Actor;
use activitypub_federation::{kinds::actor::PersonType, traits::Object};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use thiserror::Error;
use url::Url;

use crate::db::Uri;
use crate::{apub, FederationData};

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("Account already exists")]
    AlreadyExists,
    #[error("Account not found")]
    NotFound,
    #[error("Sql Error: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("URL verification failed: {0}")]
    UrlVerificationError(#[from] activitypub_federation::error::Error),
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[sqlx(transparent)]
pub struct AccountId(i64);

impl Into<AccountId> for i64 {
    fn into(self) -> AccountId {
        AccountId(self)
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: AccountId,
    pub uri: Uri,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub username: String,
    pub host: String,
    pub inbox: Uri,
    pub outbox: Option<Uri>,
    pub shared_inbox: Option<Uri>,
    pub public_key: String,
    pub private_key: Option<String>,
    pub local: bool,
}

#[async_trait]
impl Object for Account {
    type DataType = FederationData;
    type Kind = apub::Person;
    type Error = AccountError;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        data.storage.account_by_uri(&object_id.into()).await
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        data.storage.update_or_insert_account(&json).await
    }

    async fn into_json(self, _: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let shared_inbox = self.shared_inbox().map(Into::into);
        Ok(apub::Person {
            kind: PersonType::Person,
            preferred_username: self.username.clone(),
            id: self.uri.clone().into(),
            inbox: self.inbox(),
            outbox: self.outbox.map(Into::into),
            shared_inbox,
            public_key: PublicKey {
                id: format!("{}#main-key", self.uri),
                owner: self.uri.into(),
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
            .map_err(AccountError::UrlVerificationError)
    }

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        Some(self.updated_at.and_utc())
    }
}

impl Actor for Account {
    fn id(&self) -> Url {
        self.uri.clone().into()
    }

    fn inbox(&self) -> Url {
        self.inbox.clone().into()
    }

    fn public_key_pem(&self) -> &str {
        &self.public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        self.private_key.clone()
    }

    fn shared_inbox(&self) -> Option<Url> {
        self.shared_inbox.clone().map(Into::into)
    }
}

#[async_trait]
pub trait AccountStorage {
    async fn new_account(&self, person: &apub::Person) -> Result<Account, AccountError>;

    async fn account_by_id(&self, id: AccountId) -> Result<Option<Account>, AccountError>;

    async fn account_by_uri(&self, uri: &Uri) -> Result<Option<Account>, AccountError>;

    async fn get_local_account(&self) -> Result<Account, AccountError>;

    async fn update_or_insert_account(
        &self,
        person: &apub::Person,
    ) -> Result<Account, AccountError>;

    async fn delete_account_by_id(&self, id: AccountId) -> Result<(), AccountError>;

    async fn delete_account_by_uri(&self, uri: &Uri) -> Result<(), AccountError>;
}
