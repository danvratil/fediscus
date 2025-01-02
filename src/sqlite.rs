use crate::{
    config::Database,
    storage::{Account, AccountError, AccountId, AccountStorage, Storage},
};

use async_trait::async_trait;
use thiserror::Error;
use tracing::error;
use crate::db::Uri;

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
        sqlx::query_as!(Account, r#"SELECT * FROM accounts WHERE uri = ?"#, uri)
            .fetch_optional(&self.db)
            .await
            .map_err(AccountError::SqlError)
    }

    async fn new_account(&self, person: &apub::Person) -> Result<Account, AccountError> {
        let uri: Uri = person.id.inner().into();
        let host = person.inbox.host_str().ok_or(AccountError::UrlVerificationError(
            activitypub_federation::error::Error::UrlParse(url::ParseError::EmptyHost)
        ))?;
        let inbox: Uri = person.inbox.into();
        let outbox: Uri = person.outbox.into();
        let shared_inbox: Uri = person.shared_inbox.map(Into::into);
        let id = sqlx::query_scalar!(
            r#"INSERT INTO accounts (
                uri,
                username,
                host,
                inbox,
                outbox,
                shared_inbox,
                public_key,
                private_key,
                local
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
            uri,
            person.preferred_username,
            host,
            inbox,
            outbox,
            shared_inbox,
            person.public_key.public_key_pem,
            None,
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
        let uri: Uri = person.id.inner().into();
        let host = person.inbox.host_str().ok_or(AccountError::UrlVerificationError(
            activitypub_federation::error::Error::UrlParse(url::ParseError::EmptyHost)
        ))?;
        let inbox: Uri = person.inbox.into();
        let outbox: Uri = person.outbox.into();
        let shared_inbox: Uri = person.shared_inbox.map(Into::into);
        let id = sqlx::query_scalar!(
            r#"INSERT INTO accounts (
                uri,
                username,
                host,
                inbox,
                outbox,
                shared_inbox,
                public_key,
                private_key,
                local
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            None,
            false
        )
        .fetch_one(&self.db)
        .await
        .map_err(AccountError::SqlError)?;

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
}
