mod account;
mod blog;
mod follow;
mod post;

use async_trait::async_trait;

pub use account::{Account, AccountError, AccountId, AccountStorage};
pub use blog::{Blog, BlogError, BlogId, BlogStorage};
pub use follow::{Follow, FollowError, FollowStorage};
pub use post::{Post, PostError, PostId, PostStorage};

#[async_trait]
pub trait Storage: AccountStorage + FollowStorage + BlogStorage + PostStorage {}
