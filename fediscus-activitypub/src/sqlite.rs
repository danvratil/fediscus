// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use crate::{
    config::Database,
    storage::{
        Account, AccountError, AccountId, AccountStorage, Blog, BlogError, BlogId, BlogStorage,
        Follow, FollowError, FollowId, FollowStorage, Note, NoteError, NoteId, NoteStorage,
        Storage,
    },
};

use crate::db::Uri;
use async_trait::async_trait;
use thiserror::Error;
use tracing::error;
use url::Url;

use crate::apub;

pub struct SqliteStorage {
    db: sqlx::SqlitePool,
}

#[derive(Error, Debug)]
pub enum SqlError {
    #[error("Sqlx Error: {0}")]
    Sqlx(sqlx::Error),
}

impl SqliteStorage {
    pub async fn new(config: &Database) -> Result<Self, SqlError> {
        let db = sqlx::SqlitePool::connect(&config.url)
            .await
            .map_err(SqlError::Sqlx)?;
        Ok(Self { db })
    }
}

impl Storage for SqliteStorage {}

#[async_trait]
impl AccountStorage for SqliteStorage {
    async fn account_by_id(&self, id: AccountId) -> Result<Option<Account>, AccountError> {
        sqlx::query_as!(
            Account,
            r#"SELECT
                id AS "id: _",
                uri AS "uri: _",
                created_at,
                updated_at,
                username,
                host,
                inbox AS "inbox: _",
                outbox AS "outbox: _",
                shared_inbox AS "shared_inbox: _",
                public_key,
                private_key,
                local
            FROM accounts
            WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(AccountError::SqlError)
    }

    async fn account_by_uri(&self, uri: &Uri) -> Result<Option<Account>, AccountError> {
        sqlx::query_as!(
            Account,
            r#"SELECT
                id AS "id: _",
                uri AS "uri: _",
                created_at,
                updated_at,
                username,
                host,
                inbox AS "inbox: _",
                outbox AS "outbox: _",
                shared_inbox AS "shared_inbox: _",
                public_key,
                private_key,
                local
            FROM accounts
            WHERE uri = ?"#,
            uri
        )
        .fetch_optional(&self.db)
        .await
        .map_err(AccountError::SqlError)
    }

    async fn new_account(&self, person: &apub::Person) -> Result<Account, AccountError> {
        let uri: Uri = person.id.inner().clone().into();
        let host = person
            .inbox
            .host_str()
            .ok_or(AccountError::UrlVerificationError(
                activitypub_federation::error::Error::UrlParse(url::ParseError::EmptyHost),
            ))?;
        let inbox: Uri = person.inbox.clone().into();
        let outbox: Option<Uri> = person.outbox.clone().map(Into::into);
        let shared_inbox: Option<Uri> = person.shared_inbox.clone().map(Into::into);
        let id = sqlx::query_scalar!(
            r#"INSERT INTO accounts (
                uri,
                username,
                host,
                inbox,
                outbox,
                shared_inbox,
                public_key,
                local
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
            uri,
            person.preferred_username,
            host,
            inbox,
            outbox,
            shared_inbox,
            person.public_key.public_key_pem,
            false
        )
        .fetch_one(&self.db)
        .await
        .map_err(AccountError::SqlError)?
        .into();

        match self.account_by_id(id).await? {
            Some(account) => Ok(account),
            None => {
                #[cfg(debug_assertions)]
                {
                    panic!("Failed to retrieve just-created account");
                }

                #[cfg(not(debug_assertions))]
                {
                    error!("Failed to retrieve just-created account");
                    Err(AccountError::NotFound)
                }
            }
        }
    }

    async fn update_or_insert_account(
        &self,
        person: &apub::Person,
    ) -> Result<Account, AccountError> {
        let uri: Uri = person.id.inner().clone().into();
        let host = person
            .inbox
            .host_str()
            .ok_or(AccountError::UrlVerificationError(
                activitypub_federation::error::Error::UrlParse(url::ParseError::EmptyHost),
            ))?;
        let inbox: Uri = person.inbox.clone().into();
        let outbox: Option<Uri> = person.outbox.clone().map(Into::into);
        let shared_inbox: Option<Uri> = person.shared_inbox.clone().map(Into::into);
        let id = sqlx::query_scalar!(
            r#"INSERT INTO accounts (
                uri,
                username,
                host,
                inbox,
                outbox,
                shared_inbox,
                public_key,
                local
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT DO UPDATE SET
                inbox = excluded.inbox,
                outbox = excluded.outbox,
                shared_inbox = excluded.shared_inbox,
                public_key = excluded.public_key,
                private_key = excluded.private_key,
                updated_at = DATETIME('now')
            RETURNING id
            "#,
            uri,
            person.preferred_username,
            host,
            inbox,
            outbox,
            shared_inbox,
            person.public_key.public_key_pem,
            false
        )
        .fetch_one(&self.db)
        .await
        .map_err(AccountError::SqlError)?
        .into();

        match self.account_by_id(id).await? {
            Some(account) => Ok(account),
            None => {
                #[cfg(debug_assertions)]
                {
                    panic!("Failed to retrieve just-upserted account");
                }
                #[cfg(not(debug_assertions))]
                {
                    error!("Failed to retrieve just-upserted account");
                    Err(AccountError::NotFound)
                }
            }
        }
    }

    async fn delete_account_by_id(&self, id: AccountId) -> Result<(), AccountError> {
        sqlx::query!(r#"DELETE FROM accounts WHERE id = ?"#, id)
            .execute(&self.db)
            .await
            .map_err(AccountError::SqlError)?;
        Ok(())
    }

    async fn delete_account_by_uri(&self, uri: &Uri) -> Result<(), AccountError> {
        let uri = uri.as_str();
        sqlx::query!(r#"DELETE FROM accounts WHERE uri = ?"#, uri)
            .execute(&self.db)
            .await
            .map_err(AccountError::SqlError)?;
        Ok(())
    }

    async fn get_local_account(&self) -> Result<Account, AccountError> {
        sqlx::query_as!(
            Account,
            r#"SELECT
                id AS "id: _",
                uri AS "uri: _",
                created_at,
                updated_at,
                username,
                host,
                inbox AS "inbox: _",
                outbox AS "outbox: _",
                shared_inbox AS "shared_inbox: _",
                public_key,
                private_key,
                local
            FROM accounts
            WHERE local = 1
            "#,
        )
        .fetch_one(&self.db)
        .await
        .map_err(AccountError::SqlError)
    }
}

#[async_trait]
impl FollowStorage for SqliteStorage {
    async fn new_follow(
        &self,
        account_id: AccountId,
        target_account_id: AccountId,
        uri: &Uri,
        pending: bool,
    ) -> Result<Follow, FollowError> {
        sqlx::query!(
            r#"INSERT INTO follows (
                account_id,
                target_account_id,
                uri,
                pending
            )
            VALUES (?, ?, ?, ?)
            "#,
            account_id,
            target_account_id,
            uri,
            pending
        )
        .execute(&self.db)
        .await
        .map_err(FollowError::SqlError)?;

        match self.follow_by_uri(uri).await? {
            Some(follow) => Ok(follow),
            None => {
                #[cfg(debug_assertions)]
                {
                    panic!("Failed to retrieve just-created follow");
                }

                #[cfg(not(debug_assertions))]
                {
                    error!("Failed to retrieve just-created follow");
                    Err(FollowError::NotFound)
                }
            }
        }
    }

    async fn follows_by_account_id(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Follow>, FollowError> {
        sqlx::query_as!(
            Follow,
            r#"SELECT
                id,
                created_at,
                account_id,
                target_account_id,
                uri AS "uri: _",
                pending
            FROM follows
            WHERE account_id = ?"#,
            account_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(FollowError::SqlError)
    }

    async fn follow_by_uri(&self, uri: &Uri) -> Result<Option<Follow>, FollowError> {
        sqlx::query_as!(
            Follow,
            r#"SELECT
                id,
                created_at,
                account_id,
                target_account_id,
                uri AS "uri: _",
                pending
            FROM follows
            WHERE uri = ?
            "#,
            uri
        )
        .fetch_optional(&self.db)
        .await
        .map_err(FollowError::SqlError)
    }

    async fn follow_by_ids(
        &self,
        account_id: AccountId,
        target_account_id: AccountId,
    ) -> Result<Option<Follow>, FollowError> {
        sqlx::query_as!(
            Follow,
            r#"SELECT
                id,
                created_at,
                account_id,
                target_account_id,
                uri AS "uri: _",
                pending
            FROM follows
            WHERE account_id = ? AND target_account_id = ?
            "#,
            account_id,
            target_account_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(FollowError::SqlError)
    }

    async fn delete_follow_by_uri(&self, uri: &Uri) -> Result<(), FollowError> {
        let uri = uri.as_str();
        sqlx::query!(r#"DELETE FROM follows WHERE uri = ?"#, uri)
            .execute(&self.db)
            .await
            .map_err(FollowError::SqlError)?;
        Ok(())
    }

    async fn delete_follow_by_id(&self, follow_id: FollowId) -> Result<(), FollowError> {
        sqlx::query!(r#"DELETE FROM follows WHERE id = ?"#, follow_id)
            .execute(&self.db)
            .await
            .map_err(FollowError::SqlError)?;
        Ok(())
    }

    async fn follow_accepted(&self, uri: &Uri) -> Result<(), FollowError> {
        let uri = uri.as_str();
        sqlx::query!(r#"UPDATE follows SET pending = FALSE WHERE uri = ?"#, uri)
            .execute(&self.db)
            .await
            .map_err(FollowError::SqlError)?;
        Ok(())
    }
}

#[async_trait]
impl BlogStorage for SqliteStorage {
    async fn new_blog(&self, url: &Url) -> Result<Blog, BlogError> {
        let url = url.as_str();
        let id = sqlx::query_scalar!(r#"INSERT INTO blogs (url) VALUES (?) RETURNING id"#, url)
            .fetch_one(&self.db)
            .await
            .map_err(BlogError::SqlError)?
            .into();

        match self.blog_by_id(id).await? {
            Some(blog) => Ok(blog),
            None => {
                #[cfg(debug_assertions)]
                {
                    panic!("Failed to retrieve just-created blog");
                }

                #[cfg(not(debug_assertions))]
                {
                    error!("Failed to retrieve just-created blog");
                    Err(BlogError::NotFound)
                }
            }
        }
    }

    async fn blog_by_id(&self, id: BlogId) -> Result<Option<Blog>, BlogError> {
        let rcord = sqlx::query!(
            r#"SELECT
                id,
                created_at,
                url
            FROM blogs
            WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(BlogError::SqlError)?;

        match rcord {
            Some(record) => Ok(Some(Blog {
                id: record.id.into(),
                created_at: record.created_at,
                url: record.url.parse().map_err(|e| {
                    error!("Failed to parse blog URL: {}", e);
                    BlogError::UrlParseError(e)
                })?,
            })),
            None => Ok(None),
        }
    }

    async fn blog_by_url(&self, url: &Url) -> Result<Option<Blog>, BlogError> {
        let url = url.as_str();
        let record = sqlx::query!(
            r#"SELECT
                id,
                created_at,
                url
            FROM blogs
            WHERE url = ?"#,
            url
        )
        .fetch_optional(&self.db)
        .await
        .map_err(BlogError::SqlError)?;

        match record {
            Some(record) => Ok(Some(Blog {
                id: record.id.into(),
                created_at: record.created_at,
                url: record.url.parse().map_err(|e| {
                    error!("Failed to parse blog URL: {}", e);
                    BlogError::UrlParseError(e)
                })?,
            })),
            None => Ok(None),
        }
    }

    async fn delete_blog_by_id(&self, id: BlogId) -> Result<(), BlogError> {
        sqlx::query!(r#"DELETE FROM blogs WHERE id = ?"#, id)
            .execute(&self.db)
            .await
            .map_err(BlogError::SqlError)?;
        Ok(())
    }
}

#[async_trait]
impl NoteStorage for SqliteStorage {
    async fn new_post(
        &self,
        account_id: AccountId,
        uri: Uri,
        reply_to_id: Option<NoteId>,
        root_id: Option<NoteId>,
        blog_id: BlogId,
    ) -> Result<Note, NoteError> {
        let id = sqlx::query_scalar!(
            r#"INSERT INTO notes (
                account_id,
                uri,
                reply_to_id,
                root_id,
                blog_id
            )
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "#,
            account_id,
            uri,
            reply_to_id,
            root_id,
            blog_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(NoteError::SqlError)?
        .into();

        match self.post_by_id(id).await? {
            Some(post) => Ok(post),
            None => {
                #[cfg(debug_assertions)]
                {
                    panic!("Failed to retrieve just-created post");
                }

                #[cfg(not(debug_assertions))]
                {
                    error!("Failed to retrieve just-created post");
                    Err(NoteError::NotFound)
                }
            }
        }
    }

    async fn post_by_id(&self, id: NoteId) -> Result<Option<Note>, NoteError> {
        sqlx::query_as!(
            Note,
            r#"SELECT
                id AS "id: _",
                created_at AS "created_at: _",
                updated_at AS "updated_at: _",
                account_id AS "account_id: _",
                uri AS "uri: _",
                reply_to_id AS "reply_to_id: _",
                root_id AS "root_id: _",
                blog_id AS "blog_id: _",
                likes,
                reposts
            FROM notes
            WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(NoteError::SqlError)
    }

    async fn post_by_uri(&self, uri: &Uri) -> Result<Option<Note>, NoteError> {
        sqlx::query_as!(
            Note,
            r#"SELECT
                id AS "id: _",
                created_at AS "created_at: _",
                updated_at AS "updated_at: _",
                account_id AS "account_id: _",
                uri AS "uri: _",
                reply_to_id AS "reply_to_id: _",
                root_id AS "root_id: _",
                blog_id AS "blog_id: _",
                likes,
                reposts
            FROM notes
            WHERE uri = ?"#,
            uri
        )
        .fetch_optional(&self.db)
        .await
        .map_err(NoteError::SqlError)
    }

    async fn delete_post_by_id(&self, id: NoteId) -> Result<(), NoteError> {
        sqlx::query!(r#"DELETE FROM notes WHERE id = ?"#, id)
            .execute(&self.db)
            .await
            .map_err(NoteError::SqlError)?;
        Ok(())
    }

    async fn post_count(&self) -> Result<usize, NoteError> {
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM notes"#)
            .fetch_one(&self.db)
            .await
            .map_err(NoteError::SqlError)?;
        Ok(count as usize)
    }

    async fn like_post(&self, post_uri: &Uri) -> Result<(), NoteError> {
        sqlx::query!(
            r#"UPDATE notes SET likes = likes + 1 WHERE uri = ?"#,
            post_uri
        )
        .execute(&self.db)
        .await
        .map_err(NoteError::SqlError)?;
        Ok(())
    }

    async fn unlike_post(&self, post_uri: &Uri) -> Result<(), NoteError> {
        sqlx::query!(
            r#"UPDATE notes SET likes = likes - 1 WHERE uri = ?"#,
            post_uri
        )
        .execute(&self.db)
        .await
        .map_err(NoteError::SqlError)?;
        Ok(())
    }
}
