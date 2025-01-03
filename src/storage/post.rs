use async_trait::async_trait;
use chrono::NaiveDateTime;
use thiserror::Error;

use crate::db::Uri;

use super::{AccountId, BlogId};

#[derive(Debug, Error)]
pub enum PostError {
    #[error("Post already exists")]
    AlreadyExists,
    #[error("Post not found")]
    NotFound,
    #[error("Sql Error: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("Url Parse Error: {0}")]
    UrlParseError(#[from] url::ParseError),
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[sqlx(transparent)]
pub struct PostId(i64);

impl Into<PostId> for i64 {
    fn into(self) -> PostId {
        PostId(self)
    }
}

#[derive(Debug, Clone)]
pub struct Post {
    pub id: PostId,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub account_id: AccountId,
    pub uri: Uri,
    pub reply_to_id: Option<PostId>,
    pub root_id: Option<PostId>,
    pub blog_id: BlogId,
}

#[async_trait]
pub trait PostStorage {
    async fn new_post(
        &self,
        account_id: AccountId,
        uri: Uri,
        reply_to_id: Option<PostId>,
        root_id: Option<PostId>,
        blog_id: BlogId,
    ) -> Result<Post, PostError>;

    async fn post_by_id(&self, id: PostId) -> Result<Option<Post>, PostError>;

    async fn post_by_uri(&self, url: &Uri) -> Result<Option<Post>, PostError>;

    async fn delete_post_by_id(&self, id: PostId) -> Result<(), PostError>;
}
