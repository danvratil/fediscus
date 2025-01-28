// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use rsa::{RsaPrivateKey, pkcs8::{DecodePrivateKey, EncodePublicKey}};
use thiserror::Error;

mod uri;

pub use uri::Uri;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum DbError {
    #[error("SQL error: {0}")]
    Sqlx(sqlx::Error),
    #[error("URL error")]
    Url(url::ParseError),
}

/*
pub async fn get_counts(
    pool: &sqlx::SqlitePool,
    posts: &Vec<Post>,
) -> Result<(HashMap<Url, usize>, Vec<Post>), DbError> {
    let urls: Vec<_> = posts.iter().map(|post| post.url.to_string()).collect();

    let params = format!("?{}", ", ?".repeat(urls.len() - 1));
    let query_string = format!(
        "SELECT url, comment_count FROM posts WHERE url IN ({})",
        params
    );
    let mut query = sqlx::query(&query_string);
    for url in urls {
        query = query.bind(url);
    }
    let counts = query
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?
        .iter()
        .map(|row| -> Result<(Url, usize), DbError> {
            let url: String = row.get(0);
            let count: i64 = row.get(1);
            Ok((Url::parse(&url).map_err(DbError::Url)?, count as usize))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    let misses = posts
        .iter()
        .filter(|post| !counts.contains_key(&post.url))
        .cloned()
        .collect();

    Ok((counts, misses))
}iid
*/

fn derive_public_key_pem(private_key: &str) -> Result<String, anyhow::Error> {
    RsaPrivateKey::from_pkcs8_pem(private_key)?
        .to_public_key()
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .map_err(anyhow::Error::new)
}

pub async fn init_local_user(
    pool: &sqlx::SqlitePool,
    config: &crate::config::FediverseUser,
) -> Result<(), anyhow::Error> {
    let account = sqlx::query!(
        "SELECT id, local \
         FROM accounts \
         WHERE username = ? AND host = ?",
        config.username,
        config.host
    )
    .fetch_optional(pool)
    .await
    .map_err(DbError::Sqlx)?;

    let exists = match account {
        Some(account) => {
            if !account.local {
                sqlx::query!("DELETE FROM accounts WHERE id = ?", account.id)
                    .execute(pool)
                    .await
                    .map_err(DbError::Sqlx)?;
                false
            } else {
                true
            }
        }
        None => false,
    };

    if !exists {
        let inbox = format!(
            "https://{}/users/{}/inbox",
            config.host, config.username
        );
        let outbox = format!(
            "https://{}/users/{}/outbox",
            config.host, config.username
        );
        let uri = format!(
            "https://{}/users/{}",
            config.host, config.username
        );
        let public_key = derive_public_key_pem(&config.private_key)?;
        sqlx::query!(
            "INSERT INTO accounts \
             (uri, username, host, inbox, outbox, private_key, public_key, local) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            uri,
            config.username,
            config.host,
            inbox,
            outbox,
            config.private_key,
            public_key,
            true,
        )
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    }
    Ok(())
}
