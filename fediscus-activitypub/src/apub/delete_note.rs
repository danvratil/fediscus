// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::{fetch::object_id::ObjectId, kinds::activity::DeleteType};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{apub, storage};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNote {
    pub actor: ObjectId<storage::Account>,
    pub object: apub::Tombstone,
    r#type: DeleteType,
    pub id: Url,
}

impl DeleteNote {
    pub fn new(actor: ObjectId<storage::Account>, object: apub::Tombstone, id: Url) -> Self {
        Self {
            actor,
            object,
            r#type: DeleteType::Delete,
            id,
        }
    }
}
