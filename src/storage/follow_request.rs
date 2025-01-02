use async_trait::async_trait;
use sqlx::types::chrono::{DateTime, Utc};
use thiserror::Error;
use url::Url;

use super::account::AccountId;

#[derive(Debug, Error)]
pub enum FollowRequestError {
    #[error("Follow request already exists")]
    AlreadyExists,
    #[error("Follow request not found")]
    NotFound,
    #[error("Sql Error: {0}")]
    SqlError(sqlx::Error),
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FollowRequestId(i64);

#[derive(Debug, Clone)]
pub struct FollowRequest {
    pub id: FollowRequestId,
    pub created_at: DateTime<Utc>,
    pub account_id: AccountId,
    pub target_account_id: AccountId,
    pub uri: Url,
}

#[async_trait]
pub trait FollowRequestStorage {
    async fn new_follow_request(
        &self,
        account_id: AccountId,
        target_account_id: AccountId,
        uri: &Url,
    ) -> Result<FollowRequest, FollowRequestError>;

    async fn follow_requests_by_account_id(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<FollowRequest>, FollowRequestError>;

    async fn follow_request_by_uri(
        &self,
        uri: &Url,
    ) -> Result<Option<FollowRequest>, FollowRequestError>;

    async fn delete_follow_request_by_uri(&self, uri: &Url) -> Result<(), FollowRequestError>;
}
