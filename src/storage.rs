mod account;
mod follow;
mod follow_request;

use axum::async_trait;
pub use account::{Account, AccountError, AccountId, AccountStorage};
pub use follow::{Follow, FollowError, FollowId, FollowStorage};
pub use follow_request::{
    FollowRequest, FollowRequestError, FollowRequestId, FollowRequestStorage,
};


#[async_trait]
pub trait Storage: AccountStorage + FollowStorage + FollowRequestStorage {}
