// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

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

impl From<i64> for BlogId {
    fn from(id: i64) -> Self {
        BlogId(id)
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
