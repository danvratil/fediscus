mod account;
mod follow;
use async_trait::async_trait;

pub use account::{Account, AccountError, AccountId, AccountStorage};
pub use follow::{Follow, FollowError, FollowId, FollowStorage};


#[async_trait]
pub trait Storage: AccountStorage + FollowStorage {}
