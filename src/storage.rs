mod account;
mod blog;
mod follow;
mod note;

use async_trait::async_trait;

pub use account::{Account, AccountError, AccountId, AccountStorage};
pub use blog::{Blog, BlogError, BlogId, BlogStorage};
pub use follow::{Follow, FollowError, FollowStorage};
pub use note::{Note, NoteError, NoteId, NoteStorage};

#[async_trait]
pub trait Storage: AccountStorage + FollowStorage + BlogStorage + NoteStorage {}
