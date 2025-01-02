use async_trait::async_trait;
use sqlx::types::chrono::{DateTime, Utc};
use thiserror::Error;
use url::Url;

use super::account::AccountId;

#[derive(Debug, Error)]
pub enum FollowError {
    #[error("Follow already exists")]
    AlreadyExists,
    #[error("Follow not found")]
    NotFound,
    #[error("Sql Error: {0}")]
    SqlError(sqlx::Error),
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FollowId(i64);

#[derive(Debug, Clone)]
pub struct Follow {
    pub id: FollowId,
    pub created_at: DateTime<Utc>,
    pub account_id: AccountId,
    pub target_account_id: AccountId,
    pub uri: Url,
}

#[async_trait]
pub trait FollowStorage {
    async fn new_follow(
        &self,
        account_id: AccountId,
        target_account_id: AccountId,
        uri: &Url,
    ) -> Result<Follow, FollowError>;

    async fn follows_by_account_id(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Follow>, FollowError>;

    async fn follow_by_uri(&self, uri: &Url) -> Result<Option<Follow>, FollowError>;

    async fn delete_follow_by_uri(&self, uri: &Url) -> Result<(), FollowError>;
}
