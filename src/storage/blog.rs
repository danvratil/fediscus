use async_trait::async_trait;
use chrono::NaiveDateTime;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum BlogError {
    #[error("Blog already exists")]
    AlreadyExists,
    #[error("Blog not found")]
    NotFound,
    #[error("Sql Error: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("Url Parse Error: {0}")]
    UrlParseError(#[from] url::ParseError),
}


#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[sqlx(transparent)]
pub struct BlogId(i64);

impl Into<BlogId> for i64 {
    fn into(self) -> BlogId {
        BlogId(self)
    }
}

#[derive(Debug, Clone)]
pub struct Blog {
    pub id: BlogId,
    pub created_at: NaiveDateTime,
    pub url: Url,
}

#[async_trait]
pub trait BlogStorage {
    async fn new_blog(&self, url: &Url) -> Result<Blog, BlogError>;

    async fn blog_by_id(&self, id: BlogId) -> Result<Option<Blog>, BlogError>;

    async fn blog_by_url(&self, url: &Url) -> Result<Option<Blog>, BlogError>;

    async fn delete_blog_by_id(&self, id: BlogId) -> Result<(), BlogError>;
}