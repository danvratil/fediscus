// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::{fetch::object_id::ObjectId, kinds::object::NoteType};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

use crate::storage;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub r#type: String,
    pub href: Url,
    pub name: String
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub r#type: NoteType,
    pub id: ObjectId<storage::Note>,
    pub in_reply_to: Option<ObjectId<storage::Note>>,
    pub published: DateTime<Utc>,
    pub url: Url,
    pub attributed_to: ObjectId<storage::Account>,
    pub content: String,
    pub tag: Vec<Tag>,
}