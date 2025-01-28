// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::{config::Data, traits::Object};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use thiserror::Error;
use url::Url;

use crate::{apub, db::Uri, FederationData};

use super::{AccountId, BlogId};

#[derive(Debug, Error)]
pub enum NoteError {
    #[error("Post already exists")]
    AlreadyExists,
    #[error("Post not found")]
    NotFound,
    #[error("Sql Error: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("Url Parse Error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Activity error {0}")]
    ActivityError(#[from] activitypub_federation::error::Error),
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[sqlx(transparent)]
pub struct NoteId(i64);

impl From<i64> for NoteId {
    fn from(id: i64) -> Self {
        NoteId(id)
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub id: NoteId,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub account_id: AccountId,
    pub uri: Uri,
    pub reply_to_id: Option<NoteId>,
    pub root_id: Option<NoteId>,
    pub blog_id: BlogId,
}

#[async_trait]
impl Object for Note {
    type DataType = FederationData;
    type Kind = apub::Note;
    type Error = NoteError;

    async fn read_from_id(_id: Url, _data: &Data<Self::DataType>) ->  Result<Option<Self> ,Self::Error> {
        todo!()
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        todo!()
    }

    async fn from_json(_json: Self::Kind, _data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        todo!()
    }

    async fn verify(_json: &Self::Kind, _expected_domain: &Url, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[async_trait]
pub trait NoteStorage {
    async fn new_post(
        &self,
        account_id: AccountId,
        uri: Uri,
        reply_to_id: Option<NoteId>,
        root_id: Option<NoteId>,
        blog_id: BlogId,
    ) -> Result<Note, NoteError>;

    async fn post_by_id(&self, id: NoteId) -> Result<Option<Note>, NoteError>;

    async fn post_by_uri(&self, url: &Uri) -> Result<Option<Note>, NoteError>;

    async fn delete_post_by_id(&self, id: NoteId) -> Result<(), NoteError>;
}
