// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::config::Data;
use activitypub_federation::traits::Object;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use thiserror::Error;
use url::Url;

use crate::{apub, db::Uri, FederationData};

use super::{account::AccountId, AccountError};

#[derive(Debug, Error)]
pub enum FollowError {
    #[error("Follow already exists")]
    AlreadyExists,
    #[error("Follow not found")]
    NotFound,
    #[error("Invalid account")]
    InvalidAccount(#[from] AccountError),
    #[error("Sql Error: {0}")]
    SqlError(sqlx::Error),
    #[error("Activity error {0}")]
    ActivityError(#[from] activitypub_federation::error::Error),
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[sqlx(transparent)]
pub struct FollowId(i64);

impl From<i64> for FollowId {
    fn from(id: i64) -> Self {
        FollowId(id)
    }
}

#[derive(Debug, Clone)]
pub struct Follow {
    pub id: FollowId,
    pub created_at: NaiveDateTime,
    pub account_id: AccountId,
    pub target_account_id: AccountId,
    pub uri: Uri,
    pub pending: bool,
}

#[async_trait]
pub trait FollowStorage {
    async fn new_follow(
        &self,
        account_id: AccountId,
        target_account_id: AccountId,
        uri: &Uri,
        pending: bool,
    ) -> Result<Follow, FollowError>;

    async fn follows_by_account_id(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Follow>, FollowError>;

    async fn follow_by_uri(&self, uri: &Uri) -> Result<Option<Follow>, FollowError>;

    async fn follow_by_ids(
        &self,
        account_id: AccountId,
        target_account_id: AccountId,
    ) -> Result<Option<Follow>, FollowError>;

    async fn delete_follow_by_uri(&self, uri: &Uri) -> Result<(), FollowError>;

    async fn delete_follow_by_id(&self, follow_id: FollowId) -> Result<(), FollowError>;

    async fn follow_accepted(&self, uri: &Uri) -> Result<(), FollowError>;
}

#[async_trait]
impl Object for Follow {
    type DataType = FederationData;
    type Kind = apub::Follow;
    type Error = FollowError;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        data.service
            .storage()
            .follow_by_uri(&object_id.into())
            .await
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let account = data
            .service
            .storage()
            .account_by_id(self.account_id)
            .await
            .map_err(FollowError::InvalidAccount)?
            .ok_or(FollowError::NotFound)?;
        let target_account = data
            .service
            .storage()
            .account_by_id(self.target_account_id)
            .await
            .map_err(FollowError::InvalidAccount)?
            .ok_or(FollowError::NotFound)?;
        Ok(apub::Follow::new(
            account.uri.into(),
            target_account.uri.into(),
            self.uri.into(),
        ))
    }

    /// Creates a new follow from the given JSON object. The state is set to pending.
    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        let actor = json
            .actor
            .dereference(data)
            .await
            .map_err(FollowError::InvalidAccount)?;
        let object = json
            .object
            .dereference(data)
            .await
            .map_err(FollowError::InvalidAccount)?;
        data.service
            .storage()
            .new_follow(actor.id, object.id, &json.id.into_inner().into(), true)
            .await
    }

    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        data.service.storage().delete_follow_by_uri(&self.uri).await
    }

    async fn verify(
        _json: &Self::Kind,
        _expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
